import json

with open("target/tarpaulin-report.json") as f:
    data = json.load(f)

# Files to analyze - ones with most uncovered lines
targets = [
    "model_card.rs", "utils.rs", "formats.rs", "version.rs", "traits.rs",
    "rag/vector.rs", "rag/cache.rs", "rag/documents.rs", "rag/rules.rs",
    "rag/knowledge.rs", "vault.rs", "conversion.rs", "storage.rs",
    "crypto/mod.rs", "audit.rs", "blockchain.rs", "error.rs",
    "config.rs", "storage/local.rs", "crypto/streaming.rs"
]

for finfo in data["files"]:
    path = "/".join(finfo["path"])
    if "src/" not in path:
        continue
    short = path.split("src/")[1] if "src/" in path else path
    if short not in targets:
        continue
    uncovered = [t["line"] for t in finfo["traces"] if t["stats"]["Line"] == 0]
    if not uncovered:
        continue
    total = len(finfo["traces"])
    covered = total - len(uncovered)
    print(f"\n=== {short} ({covered}/{total}, {len(uncovered)} uncovered) ===")
    # Group consecutive lines into ranges
    ranges = []
    for line in sorted(uncovered):
        if ranges and line == ranges[-1][1] + 1:
            ranges[-1] = (ranges[-1][0], line)
        else:
            ranges.append((line, line))
    for start, end in ranges:
        if start == end:
            print(f"  L{start}")
        else:
            print(f"  L{start}-L{end}")
