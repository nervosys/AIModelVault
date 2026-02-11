//! Blockchain-based immutable audit trail
//!
//! Provides cryptographic proof of audit log integrity using:
//! - Merkle tree structure for efficient verification
//! - Hash chain linking blocks together
//! - Digital signatures for non-repudiation
//!
//! ## Architecture
//!
//! ```text
//! Block N-1          Block N            Block N+1
//! ┌────────────┐    ┌────────────┐    ┌────────────┐
//! │ prev_hash  │◄───│ prev_hash  │◄───│ prev_hash  │
//! │ merkle_root│    │ merkle_root│    │ merkle_root│
//! │ timestamp  │    │ timestamp  │    │ timestamp  │
//! │ nonce      │    │ nonce      │    │ nonce      │
//! └────────────┘    └────────────┘    └────────────┘
//!       │                │                │
//!    entries          entries          entries
//! ```
//!
//! ## Security Properties
//!
//! - **Immutability**: Hash chains prevent modification of past entries
//! - **Tamper Evidence**: Any modification breaks the chain
//! - **Non-repudiation**: Optional signing proves authorship
//! - **Efficient Verification**: Merkle proofs for individual entries

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::audit::{AuditEntry, AuditEventType};
use crate::error::{Result, VaultError};

/// SHA-256 hash as hex string
pub type Hash = String;

/// Block index
pub type BlockIndex = u64;

/// Compute SHA-256 hash of data
fn sha256(data: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Merkle tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    /// Hash of this node
    pub hash: Hash,
    /// Left child hash (if internal node)
    pub left: Option<Hash>,
    /// Right child hash (if internal node)
    pub right: Option<Hash>,
}

/// Merkle tree for a set of entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    /// Root hash
    pub root: Hash,
    /// All nodes indexed by hash
    pub nodes: HashMap<Hash, MerkleNode>,
    /// Leaf hashes (in order)
    pub leaves: Vec<Hash>,
}

impl MerkleTree {
    /// Build a Merkle tree from data
    pub fn build(data: &[Vec<u8>]) -> Self {
        if data.is_empty() {
            return Self {
                root: sha256(b""),
                nodes: HashMap::new(),
                leaves: Vec::new(),
            };
        }

        // Compute leaf hashes
        let leaves: Vec<Hash> = data.iter().map(|d| sha256(d)).collect();
        let mut nodes = HashMap::new();

        // Add leaf nodes
        for hash in &leaves {
            nodes.insert(
                hash.clone(),
                MerkleNode {
                    hash: hash.clone(),
                    left: None,
                    right: None,
                },
            );
        }

        // Build tree bottom-up
        let mut current_level = leaves.clone();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                let left = &chunk[0];
                let right = if chunk.len() > 1 { &chunk[1] } else { left };

                let combined = format!("{}{}", left, right);
                let parent_hash = sha256(combined.as_bytes());

                nodes.insert(
                    parent_hash.clone(),
                    MerkleNode {
                        hash: parent_hash.clone(),
                        left: Some(left.clone()),
                        right: Some(right.clone()),
                    },
                );

                next_level.push(parent_hash);
            }

            current_level = next_level;
        }

        Self {
            root: current_level.into_iter().next().unwrap_or_default(),
            nodes,
            leaves,
        }
    }

    /// Generate proof for a leaf at given index
    pub fn generate_proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() {
            return None;
        }

        let mut proof = Vec::new();
        let mut current_idx = index;
        let mut current_level = self.leaves.clone();

        while current_level.len() > 1 {
            let sibling_idx = if current_idx % 2 == 0 {
                current_idx + 1
            } else {
                current_idx - 1
            };

            let sibling = current_level
                .get(sibling_idx.min(current_level.len() - 1))
                .cloned();
            let is_left = current_idx % 2 == 1;

            if let Some(s) = sibling {
                proof.push(ProofElement { hash: s, is_left });
            }

            // Build next level
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                let left = &chunk[0];
                let right = if chunk.len() > 1 { &chunk[1] } else { left };
                let combined = format!("{}{}", left, right);
                next_level.push(sha256(combined.as_bytes()));
            }

            current_idx /= 2;
            current_level = next_level;
        }

        Some(MerkleProof {
            leaf_hash: self.leaves[index].clone(),
            proof,
            root: self.root.clone(),
        })
    }

    /// Verify a proof
    pub fn verify_proof(proof: &MerkleProof) -> bool {
        let mut current = proof.leaf_hash.clone();

        for element in &proof.proof {
            let combined = if element.is_left {
                format!("{}{}", element.hash, current)
            } else {
                format!("{}{}", current, element.hash)
            };
            current = sha256(combined.as_bytes());
        }

        current == proof.root
    }
}

/// Merkle proof for a single entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Hash of the leaf being proven
    pub leaf_hash: Hash,
    /// Proof path (hashes of siblings from leaf to root)
    pub proof: Vec<ProofElement>,
    /// Expected root hash
    pub root: Hash,
}

/// Single element in a Merkle proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofElement {
    /// Hash of the sibling node
    pub hash: Hash,
    /// Whether this sibling is on the left
    pub is_left: bool,
}

/// Audit block in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditBlock {
    /// Block index (height)
    pub index: BlockIndex,
    /// Timestamp of block creation
    pub timestamp: DateTime<Utc>,
    /// Hash of the previous block
    pub prev_hash: Hash,
    /// Merkle root of entries in this block
    pub merkle_root: Hash,
    /// Entries in this block
    pub entries: Vec<BlockEntry>,
    /// Optional digital signature (base64-encoded)
    pub signature: Option<String>,
    /// Nonce (for proof-of-work, if enabled)
    pub nonce: u64,
    /// Block hash
    pub hash: Hash,
}

impl AuditBlock {
    /// Compute block hash
    pub fn compute_hash(&self) -> Hash {
        let header = format!(
            "{}:{}:{}:{}:{}",
            self.index, self.timestamp, self.prev_hash, self.merkle_root, self.nonce
        );
        sha256(header.as_bytes())
    }

    /// Verify block integrity
    pub fn verify(&self, prev_block: Option<&AuditBlock>) -> BlockVerification {
        let mut issues = Vec::new();

        // Check hash
        if self.hash != self.compute_hash() {
            issues.push("Block hash mismatch".into());
        }

        // Check previous hash
        if let Some(prev) = prev_block {
            if self.prev_hash != prev.hash {
                issues.push("Previous hash mismatch".into());
            }
            if self.index != prev.index + 1 {
                issues.push("Non-sequential index".into());
            }
            if self.timestamp < prev.timestamp {
                issues.push("Timestamp before previous block".into());
            }
        } else if self.index != 0 {
            issues.push("Non-genesis block without predecessor".into());
        }

        // Verify Merkle root
        let entry_data: Vec<Vec<u8>> = self
            .entries
            .iter()
            .map(|e| serde_json::to_vec(e).unwrap_or_default())
            .collect();
        let tree = MerkleTree::build(&entry_data);
        if tree.root != self.merkle_root {
            issues.push("Merkle root mismatch".into());
        }

        BlockVerification {
            valid: issues.is_empty(),
            issues,
        }
    }
}

/// Block verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockVerification {
    /// Whether the block is valid
    pub valid: bool,
    /// List of issues found
    pub issues: Vec<String>,
}

/// Entry within a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEntry {
    /// Original audit entry
    pub audit: AuditEntry,
    /// Entry hash
    pub hash: Hash,
    /// Index within block
    pub index_in_block: usize,
}

/// Blockchain audit trail manager
pub struct BlockchainAudit {
    /// Chain storage directory
    chain_dir: PathBuf,
    /// Current block being built
    pending_entries: Vec<BlockEntry>,
    /// Block size threshold (entries per block)
    block_size: usize,
    /// Latest block (cached)
    latest_block: Option<AuditBlock>,
    /// Genesis block hash
    genesis_hash: Hash,
}

impl BlockchainAudit {
    /// Create new blockchain audit manager
    pub fn new(chain_dir: &Path, block_size: usize) -> Result<Self> {
        fs::create_dir_all(chain_dir)?;

        let mut manager = Self {
            chain_dir: chain_dir.to_path_buf(),
            pending_entries: Vec::new(),
            block_size,
            latest_block: None,
            genesis_hash: String::new(),
        };

        // Load latest block
        manager.load_latest_block()?;

        // Create genesis block if chain is empty
        if manager.latest_block.is_none() {
            manager.create_genesis_block()?;
        }

        Ok(manager)
    }

    /// Create genesis block
    fn create_genesis_block(&mut self) -> Result<()> {
        let genesis_entry = BlockEntry {
            audit: AuditEntry {
                timestamp: Utc::now(),
                event_type: AuditEventType::VaultCreated,
                description: "Blockchain audit trail initialized".into(),
                model_name: None,
                version: None,
                success: true,
                metadata: None,
            },
            hash: sha256(b"genesis_entry"),
            index_in_block: 0,
        };

        // Compute Merkle root from entries
        let entry_data: Vec<Vec<u8>> = vec![serde_json::to_vec(&genesis_entry).unwrap_or_default()];
        let tree = MerkleTree::build(&entry_data);

        let genesis = AuditBlock {
            index: 0,
            timestamp: Utc::now(),
            prev_hash: "0".repeat(64), // Genesis has no predecessor
            merkle_root: tree.root,
            entries: vec![genesis_entry],
            signature: None,
            nonce: 0,
            hash: String::new(),
        };

        let mut genesis = genesis;
        genesis.hash = genesis.compute_hash();
        self.genesis_hash = genesis.hash.clone();

        self.save_block(&genesis)?;
        self.latest_block = Some(genesis);

        Ok(())
    }

    /// Load the latest block from disk
    fn load_latest_block(&mut self) -> Result<()> {
        let index_path = self.chain_dir.join("latest_index");
        if !index_path.exists() {
            return Ok(());
        }

        let latest_idx: BlockIndex = fs::read_to_string(&index_path)?
            .trim()
            .parse()
            .map_err(|e| VaultError::IoError(std::io::Error::other(format!("Parse error: {e}"))))?;

        let block_path = self.chain_dir.join(format!("block_{:08}.json", latest_idx));
        if block_path.exists() {
            let contents = fs::read_to_string(&block_path)?;
            let block: AuditBlock = serde_json::from_str(&contents)?;
            self.genesis_hash = if block.index == 0 {
                block.hash.clone()
            } else {
                // Load genesis to get its hash
                let genesis_path = self.chain_dir.join("block_00000000.json");
                if genesis_path.exists() {
                    let genesis_contents = fs::read_to_string(&genesis_path)?;
                    let genesis: AuditBlock = serde_json::from_str(&genesis_contents)?;
                    genesis.hash
                } else {
                    String::new()
                }
            };
            self.latest_block = Some(block);
        }

        Ok(())
    }

    /// Save a block to disk
    fn save_block(&self, block: &AuditBlock) -> Result<()> {
        let block_path = self
            .chain_dir
            .join(format!("block_{:08}.json", block.index));
        let json = serde_json::to_string_pretty(block)?;
        fs::write(&block_path, json)?;

        // Update latest index
        let index_path = self.chain_dir.join("latest_index");
        fs::write(&index_path, block.index.to_string())?;

        Ok(())
    }

    /// Add an entry to the pending block
    pub fn add_entry(&mut self, entry: AuditEntry) -> Result<Hash> {
        let entry_json = serde_json::to_vec(&entry)?;
        let entry_hash = sha256(&entry_json);

        let block_entry = BlockEntry {
            audit: entry,
            hash: entry_hash.clone(),
            index_in_block: self.pending_entries.len(),
        };

        self.pending_entries.push(block_entry);

        // Create new block if threshold reached
        if self.pending_entries.len() >= self.block_size {
            self.finalize_block()?;
        }

        Ok(entry_hash)
    }

    /// Finalize current pending entries into a new block
    pub fn finalize_block(&mut self) -> Result<Option<BlockIndex>> {
        if self.pending_entries.is_empty() {
            return Ok(None);
        }

        let prev = self
            .latest_block
            .as_ref()
            .ok_or_else(|| VaultError::IoError(std::io::Error::other("No previous block")))?;

        // Build Merkle tree
        let entry_data: Vec<Vec<u8>> = self
            .pending_entries
            .iter()
            .map(|e| serde_json::to_vec(e).unwrap_or_default())
            .collect();
        let tree = MerkleTree::build(&entry_data);

        let mut block = AuditBlock {
            index: prev.index + 1,
            timestamp: Utc::now(),
            prev_hash: prev.hash.clone(),
            merkle_root: tree.root,
            entries: std::mem::take(&mut self.pending_entries),
            signature: None,
            nonce: 0,
            hash: String::new(),
        };

        block.hash = block.compute_hash();

        self.save_block(&block)?;
        let idx = block.index;
        self.latest_block = Some(block);

        Ok(Some(idx))
    }

    /// Get a specific block
    pub fn get_block(&self, index: BlockIndex) -> Result<Option<AuditBlock>> {
        let block_path = self.chain_dir.join(format!("block_{:08}.json", index));
        if !block_path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&block_path)?;
        let block: AuditBlock = serde_json::from_str(&contents)?;
        Ok(Some(block))
    }

    /// Get the latest block
    pub fn latest(&self) -> Option<&AuditBlock> {
        self.latest_block.as_ref()
    }

    /// Get chain height (number of blocks)
    pub fn height(&self) -> BlockIndex {
        self.latest_block.as_ref().map(|b| b.index + 1).unwrap_or(0)
    }

    /// Verify entire chain integrity
    pub fn verify_chain(&self) -> ChainVerification {
        let mut result = ChainVerification {
            valid: true,
            blocks_verified: 0,
            blocks_total: self.height(),
            issues: Vec::new(),
        };

        if result.blocks_total == 0 {
            return result;
        }

        let mut prev_block: Option<AuditBlock> = None;

        for idx in 0..result.blocks_total {
            match self.get_block(idx) {
                Ok(Some(block)) => {
                    let verification = block.verify(prev_block.as_ref());
                    if !verification.valid {
                        result.valid = false;
                        for issue in verification.issues {
                            result.issues.push(format!("Block {}: {}", idx, issue));
                        }
                    }
                    result.blocks_verified += 1;
                    prev_block = Some(block);
                }
                Ok(None) => {
                    result.valid = false;
                    result.issues.push(format!("Block {} missing", idx));
                    break;
                }
                Err(e) => {
                    result.valid = false;
                    result.issues.push(format!("Block {} error: {}", idx, e));
                    break;
                }
            }
        }

        result
    }

    /// Generate proof for an entry
    pub fn generate_proof(&self, block_idx: BlockIndex, entry_idx: usize) -> Result<AuditProof> {
        let block = self
            .get_block(block_idx)?
            .ok_or_else(|| VaultError::IoError(std::io::Error::other("Block not found")))?;

        if entry_idx >= block.entries.len() {
            return Err(VaultError::IoError(std::io::Error::other(
                "Entry not found",
            )));
        }

        // Build Merkle tree and generate proof
        let entry_data: Vec<Vec<u8>> = block
            .entries
            .iter()
            .map(|e| serde_json::to_vec(e).unwrap_or_default())
            .collect();
        let tree = MerkleTree::build(&entry_data);

        let merkle_proof = tree.generate_proof(entry_idx).ok_or_else(|| {
            VaultError::IoError(std::io::Error::other("Failed to generate Merkle proof"))
        })?;

        // Build chain of block hashes to genesis
        let mut block_chain = Vec::new();
        let mut current_idx = block_idx;
        while current_idx > 0 {
            if let Some(b) = self.get_block(current_idx)? {
                block_chain.push(BlockHashLink {
                    index: b.index,
                    hash: b.hash,
                    prev_hash: b.prev_hash,
                });
            }
            current_idx -= 1;
        }
        // Add genesis
        if let Some(genesis) = self.get_block(0)? {
            block_chain.push(BlockHashLink {
                index: 0,
                hash: genesis.hash,
                prev_hash: genesis.prev_hash,
            });
        }

        Ok(AuditProof {
            entry: block.entries[entry_idx].clone(),
            block_index: block_idx,
            merkle_proof,
            block_chain,
            genesis_hash: self.genesis_hash.clone(),
        })
    }

    /// Verify a proof
    pub fn verify_proof(proof: &AuditProof) -> ProofVerification {
        let mut issues = Vec::new();

        // Verify Merkle proof
        if !MerkleTree::verify_proof(&proof.merkle_proof) {
            issues.push("Merkle proof invalid".into());
        }

        // Verify block chain to genesis
        for i in 0..proof.block_chain.len() - 1 {
            let current = &proof.block_chain[i];
            let prev = &proof.block_chain[i + 1];

            if current.prev_hash != prev.hash {
                issues.push(format!("Block chain broken at index {}", current.index));
            }
        }

        // Verify ends at genesis
        if let Some(last) = proof.block_chain.last() {
            if last.index != 0 || last.hash != proof.genesis_hash {
                issues.push("Chain doesn't end at genesis".into());
            }
        }

        ProofVerification {
            valid: issues.is_empty(),
            issues,
        }
    }

    /// Search entries
    pub fn search(
        &self,
        model_name: Option<&str>,
        event_type: Option<AuditEventType>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<Vec<(BlockIndex, usize, BlockEntry)>> {
        let mut results = Vec::new();
        let mut idx = self.height().saturating_sub(1);

        loop {
            if let Some(block) = self.get_block(idx)? {
                // Check time bounds
                if let Some(from_ts) = from {
                    if block.timestamp < from_ts {
                        break;
                    }
                }
                if let Some(to_ts) = to {
                    if block.timestamp > to_ts {
                        if idx == 0 {
                            break;
                        }
                        idx -= 1;
                        continue;
                    }
                }

                for (entry_idx, entry) in block.entries.iter().enumerate() {
                    let matches = model_name
                        .map(|m| entry.audit.model_name.as_deref() == Some(m))
                        .unwrap_or(true)
                        && event_type
                            .as_ref()
                            .map(|t| {
                                std::mem::discriminant(&entry.audit.event_type)
                                    == std::mem::discriminant(t)
                            })
                            .unwrap_or(true);

                    if matches {
                        results.push((idx, entry_idx, entry.clone()));
                        if results.len() >= limit {
                            return Ok(results);
                        }
                    }
                }
            }

            if idx == 0 {
                break;
            }
            idx -= 1;
        }

        Ok(results)
    }
}

/// Complete audit proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditProof {
    /// The entry being proven
    pub entry: BlockEntry,
    /// Block index containing the entry
    pub block_index: BlockIndex,
    /// Merkle proof within the block
    pub merkle_proof: MerkleProof,
    /// Chain of block hashes from entry's block to genesis
    pub block_chain: Vec<BlockHashLink>,
    /// Genesis block hash
    pub genesis_hash: Hash,
}

/// Link in the block hash chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHashLink {
    /// Block index
    pub index: BlockIndex,
    /// Block hash
    pub hash: Hash,
    /// Previous block hash
    pub prev_hash: Hash,
}

/// Chain verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainVerification {
    /// Whether the entire chain is valid
    pub valid: bool,
    /// Number of blocks verified
    pub blocks_verified: u64,
    /// Total blocks in chain
    pub blocks_total: u64,
    /// Issues found
    pub issues: Vec<String>,
}

/// Proof verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerification {
    /// Whether the proof is valid
    pub valid: bool,
    /// Issues found
    pub issues: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash() {
        let hash = sha256(b"hello");
        assert_eq!(hash.len(), 64); // 256 bits = 32 bytes = 64 hex chars
    }

    #[test]
    fn test_merkle_tree_single() {
        let data = vec![b"hello".to_vec()];
        let tree = MerkleTree::build(&data);

        assert_eq!(tree.leaves.len(), 1);
        assert!(!tree.root.is_empty());
    }

    #[test]
    fn test_merkle_tree_multiple() {
        let data = vec![
            b"entry1".to_vec(),
            b"entry2".to_vec(),
            b"entry3".to_vec(),
            b"entry4".to_vec(),
        ];
        let tree = MerkleTree::build(&data);

        assert_eq!(tree.leaves.len(), 4);

        // Generate and verify proofs
        for i in 0..4 {
            let proof = tree.generate_proof(i).unwrap();
            assert!(MerkleTree::verify_proof(&proof));
        }
    }

    #[test]
    fn test_merkle_proof_verification() {
        let data = vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()];
        let tree = MerkleTree::build(&data);

        let proof = tree.generate_proof(1).unwrap();
        assert!(MerkleTree::verify_proof(&proof));

        // Tamper with proof
        let mut tampered = proof.clone();
        tampered.leaf_hash = sha256(b"tampered");
        assert!(!MerkleTree::verify_proof(&tampered));
    }

    #[test]
    fn test_block_verification() {
        let block = AuditBlock {
            index: 1,
            timestamp: Utc::now(),
            prev_hash: "0".repeat(64),
            merkle_root: sha256(b"root"),
            entries: vec![],
            signature: None,
            nonce: 0,
            hash: String::new(),
        };

        let mut block = block;
        block.hash = block.compute_hash();

        // Valid block (as genesis-like)
        let result = block.verify(None);
        // Will fail because index != 0 and no predecessor
        assert!(!result.valid);
    }

    #[test]
    fn test_blockchain_audit_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let audit = BlockchainAudit::new(temp_dir.path(), 10).unwrap();

        assert!(audit.latest().is_some());
        assert_eq!(audit.height(), 1); // Genesis block
    }

    #[test]
    fn test_blockchain_add_entry() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut audit = BlockchainAudit::new(temp_dir.path(), 2).unwrap();

        let entry = AuditEntry {
            timestamp: Utc::now(),
            event_type: AuditEventType::ModelStored,
            description: "Test entry".into(),
            model_name: Some("test_model".into()),
            version: Some(1),
            success: true,
            metadata: None,
        };

        let hash = audit.add_entry(entry).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_blockchain_verify_chain() {
        let temp_dir = tempfile::tempdir().unwrap();
        let audit = BlockchainAudit::new(temp_dir.path(), 10).unwrap();

        let result = audit.verify_chain();
        assert!(result.valid);
        assert_eq!(result.blocks_verified, 1);
    }
}
