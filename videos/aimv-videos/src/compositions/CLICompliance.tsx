// CLI Compliance — `aim compliance` security audit demo
import React from "react";
import {
  AbsoluteFill,
  interpolate,
  useCurrentFrame,
} from "remotion";
import { COLORS, FONTS } from "../theme";
import { GridBackground } from "../components/shared";

interface Line {
  type: "cmd" | "out";
  text: string;
  at: number;
  color?: string;
}

const LINES: Line[] = [
  { type: "cmd", text: "aim compliance --verbose", at: 15 },
  { type: "out", text: "🛡️  Running security audit...", at: 50 },
  { type: "out", text: "", at: 58 },
  { type: "out", text: "  Encryption", at: 62 },
  { type: "out", text: "  ✓ FIPS 140-3 compliant cipher (AES-256-GCM)", at: 68, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ Authenticated encryption with GHASH MAC", at: 75, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ 96-bit random nonces (CSPRNG)", at: 82, color: COLORS.terminalGreen },
  { type: "out", text: "", at: 90 },
  { type: "out", text: "  Key Derivation", at: 94 },
  { type: "out", text: "  ✓ Argon2id (OWASP recommended)", at: 100, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ Memory cost: 19 MiB  ·  Iterations: 2", at: 107, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ 32-byte salt (CSPRNG)", at: 114, color: COLORS.terminalGreen },
  { type: "out", text: "", at: 122 },
  { type: "out", text: "  Code Safety", at: 126 },
  { type: "out", text: "  ✓ Zero unsafe code blocks", at: 132, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ Supply chain audit passed (cargo-deny)", at: 139, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ No known CVEs in dependency tree", at: 146, color: COLORS.terminalGreen },
  { type: "out", text: "", at: 154 },
  { type: "out", text: "  Threat Coverage", at: 158 },
  { type: "out", text: "  ✓ MITRE ATT&CK: T1005, T1027, T1486 mitigated", at: 164, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ At-rest encryption for all stored models", at: 171, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ Tamper detection via SHA-256 checksums", at: 178, color: COLORS.terminalGreen },
  { type: "out", text: "", at: 188 },
  { type: "out", text: "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", at: 192 },
  { type: "out", text: "  Score: 100 / 100                  PASS ✅", at: 198, color: COLORS.terminalGreen },
  { type: "out", text: "  12/12 checks passed · 0 vulnerabilities", at: 206, color: COLORS.terminalGreen },

  { type: "cmd", text: "aim audit-log --last 3", at: 235 },
  { type: "out", text: "  Timestamp            Action       Model           User", at: 268 },
  { type: "out", text: "  ──────────────────   ──────       ─────           ────", at: 273 },
  { type: "out", text: "  2025-01-15 14:35     compliance   llama-3.1-70b   adam", at: 279 },
  { type: "out", text: "  2025-01-15 14:32     store (v3)   llama-3.1-70b   adam", at: 285 },
  { type: "out", text: "  2025-01-14 09:18     store (v2)   llama-3.1-70b   adam", at: 291 },
];

const TYPE_SPEED = 1.3;

export const CLICompliance: React.FC = () => {
  const frame = useCurrentFrame();

  return (
    <AbsoluteFill>
      <GridBackground />
      <AbsoluteFill
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          padding: "60px 80px",
        }}
      >
        <div
          style={{
            fontFamily: FONTS.display,
            fontSize: 40,
            fontWeight: 700,
            color: COLORS.text,
            marginBottom: 8,
            opacity: interpolate(frame, [0, 12], [0, 1], {
              extrapolateRight: "clamp",
            }),
          }}
        >
          Security &amp; Compliance
        </div>
        <div
          style={{
            fontFamily: FONTS.sans,
            fontSize: 20,
            color: COLORS.textMuted,
            marginBottom: 30,
            opacity: interpolate(frame, [5, 18], [0, 1], {
              extrapolateRight: "clamp",
            }),
          }}
        >
          FIPS 140-3 · MITRE ATT&amp;CK · CVE scanning — built in
        </div>

        <div
          style={{
            width: "88%",
            borderRadius: 14,
            overflow: "hidden",
            boxShadow: `0 25px 60px rgba(0,0,0,0.6), 0 0 40px ${COLORS.glow}`,
            border: `1px solid ${COLORS.border}`,
          }}
        >
          <div
            style={{
              background: "#1c2333",
              padding: "10px 16px",
              display: "flex",
              alignItems: "center",
              gap: 8,
            }}
          >
            <div style={{ display: "flex", gap: 6 }}>
              {["#ff5f57", "#febc2e", "#28c840"].map((c) => (
                <div
                  key={c}
                  style={{
                    width: 12,
                    height: 12,
                    borderRadius: "50%",
                    background: c,
                  }}
                />
              ))}
            </div>
            <span
              style={{
                color: COLORS.textMuted,
                fontSize: 13,
                fontFamily: FONTS.mono,
                marginLeft: 8,
              }}
            >
              Terminal — aim compliance
            </span>
          </div>

          <div
            style={{
              background: COLORS.terminalBg,
              padding: "22px 28px",
              fontFamily: FONTS.mono,
              fontSize: 15,
              lineHeight: 1.7,
              color: COLORS.text,
              minHeight: 440,
            }}
          >
            {LINES.map((line, i) => {
              if (line.type === "cmd") {
                const elapsed = frame - line.at;
                if (elapsed < 0) return null;
                const chars = Math.floor(elapsed / TYPE_SPEED);
                const visible = line.text.slice(0, chars);
                const done = chars >= line.text.length;
                return (
                  <div key={i} style={{ marginTop: i > 0 ? 10 : 0 }}>
                    <span style={{ color: COLORS.terminalPrompt }}>$ </span>
                    <span style={{ color: COLORS.terminalGreen }}>
                      {visible}
                    </span>
                    {!done && <span style={{ color: COLORS.text }}>▌</span>}
                  </div>
                );
              }
              if (frame < line.at) return null;
              const opacity = interpolate(frame - line.at, [0, 5], [0, 1], {
                extrapolateRight: "clamp",
              });
              return (
                <div
                  key={i}
                  style={{ color: line.color ?? COLORS.text, opacity }}
                >
                  {line.text}
                </div>
              );
            })}
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
