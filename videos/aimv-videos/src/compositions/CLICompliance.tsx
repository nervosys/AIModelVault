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
  { type: "out", text: "  Encryption", at: 60 },
  { type: "out", text: "  ✓ FIPS 140-3 cipher (AES-256-GCM)", at: 67, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ Authenticated encryption (GHASH)", at: 74, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ 96-bit random nonces (CSPRNG)", at: 81, color: COLORS.terminalGreen },
  { type: "out", text: "  Key Derivation", at: 92 },
  { type: "out", text: "  ✓ Argon2id (OWASP recommended)", at: 99, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ Memory: 19 MiB · Iterations: 2", at: 106, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ 32-byte salt (CSPRNG)", at: 113, color: COLORS.terminalGreen },
  { type: "out", text: "  Code Safety", at: 124 },
  { type: "out", text: "  ✓ Zero unsafe code blocks", at: 131, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ Supply chain audit passed", at: 138, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ No known CVEs in deps", at: 145, color: COLORS.terminalGreen },
  { type: "out", text: "  Threat Coverage", at: 156 },
  { type: "out", text: "  ✓ MITRE ATT&CK mitigated", at: 163, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ At-rest encryption for all models", at: 170, color: COLORS.terminalGreen },
  { type: "out", text: "  ✓ Tamper detection (SHA-256)", at: 177, color: COLORS.terminalGreen },
  { type: "out", text: "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", at: 190 },
  { type: "out", text: "  Score: 100/100  PASS ✅  12/12 checks", at: 197, color: COLORS.terminalGreen },
  { type: "cmd", text: "aim audit-log --last 3", at: 230 },
  { type: "out", text: "  Timestamp          Action       Model", at: 260 },
  { type: "out", text: "  ─────────────────  ──────       ─────", at: 265 },
  { type: "out", text: "  2025-01-15 14:35   compliance   llama-3.1-70b", at: 272 },
  { type: "out", text: "  2025-01-15 14:32   store (v3)   llama-3.1-70b", at: 279 },
  { type: "out", text: "  2025-01-14 09:18   store (v2)   llama-3.1-70b", at: 286 },
];

const TYPE_SPEED = 1.3;
const FONT_SIZE = 32;
const LINE_HEIGHT = 1.45;
const LINE_PX = FONT_SIZE * LINE_HEIGHT;
const MAX_VISIBLE = 16;

export const CLICompliance: React.FC = () => {
  const frame = useCurrentFrame();
  const visibleCount = LINES.filter((l) => frame >= l.at).length;
  const scrollY = Math.max(0, visibleCount - MAX_VISIBLE) * LINE_PX;

  return (
    <AbsoluteFill>
      <GridBackground />
      <AbsoluteFill
        style={{ display: "flex", flexDirection: "column", padding: 20 }}
      >
        <div
          style={{
            width: "100%",
            flex: 1,
            borderRadius: 14,
            overflow: "hidden",
            boxShadow: `0 25px 60px rgba(0,0,0,0.6), 0 0 40px ${COLORS.glow}`,
            border: `1px solid ${COLORS.border}`,
            display: "flex",
            flexDirection: "column",
          }}
        >
          <div
            style={{
              background: "#1c2333",
              padding: "14px 24px",
              display: "flex",
              alignItems: "center",
              gap: 10,
            }}
          >
            <div style={{ display: "flex", gap: 8 }}>
              {["#ff5f57", "#febc2e", "#28c840"].map((c) => (
                <div key={c} style={{ width: 14, height: 14, borderRadius: "50%", background: c }} />
              ))}
            </div>
            <span style={{ color: COLORS.textMuted, fontSize: 18, fontFamily: FONTS.mono, marginLeft: 8 }}>
              Terminal — aim compliance
            </span>
          </div>

          <div
            style={{
              background: COLORS.terminalBg,
              padding: "32px 44px",
              fontFamily: FONTS.mono,
              fontSize: FONT_SIZE,
              lineHeight: LINE_HEIGHT,
              color: COLORS.text,
              flex: 1,
              overflow: "hidden",
            }}
          >
            <div style={{ transform: `translateY(-${scrollY}px)` }}>
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
                      <span style={{ color: COLORS.terminalGreen }}>{visible}</span>
                      {!done && <span style={{ color: COLORS.text }}>▌</span>}
                    </div>
                  );
                }
                if (frame < line.at) return null;
                const opacity = interpolate(frame - line.at, [0, 5], [0, 1], { extrapolateRight: "clamp" });
                return (
                  <div key={i} style={{ color: line.color ?? COLORS.text, opacity }}>
                    {line.text}
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
