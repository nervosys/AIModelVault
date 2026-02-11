// CLI Versions — `aim versions` + `aim get --version` rollback demo
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
  { type: "cmd", text: "aim store llama-3.1 ./v2.gguf", at: 15 },
  { type: "out", text: "📦 GGUF · 11.2 GB", at: 55 },
  { type: "out", text: "✅ Stored v2 (c1e9ab5f)", at: 65, color: COLORS.terminalGreen },
  { type: "cmd", text: "aim store llama-3.1 ./v3.safetensors", at: 95 },
  { type: "out", text: "📦 SafeTensors · 140.3 GB", at: 140 },
  { type: "out", text: "✅ Stored v3 (a7f3d21b)", at: 150, color: COLORS.terminalGreen },
  { type: "cmd", text: "aim versions llama-3.1", at: 180 },
  { type: "out", text: "  Ver  Checkpoint  Format       Size", at: 210 },
  { type: "out", text: "  ───  ──────────  ──────       ────", at: 215 },
  { type: "out", text: "▸ v3   a7f3d21b    SafeTensors  140.3 GB", at: 222 },
  { type: "out", text: "  v2   c1e9ab5f    GGUF          11.2 GB", at: 229 },
  { type: "out", text: "  v1   3fa8c2e1    SafeTensors  140.3 GB", at: 236 },
  { type: "cmd", text: "aim get llama-3.1 --version 1 -o ./rollback/", at: 265 },
  { type: "out", text: "🔓 Decrypting v1 (3fa8c2e1)...", at: 315 },
  { type: "out", text: "✅ Restored → ./rollback/llama-3.1.safetensors", at: 330, color: COLORS.terminalGreen },
  { type: "out", text: "   SHA-256 verified ✓", at: 340, color: COLORS.terminalGreen },
  { type: "cmd", text: "aim lineage llama-3.1", at: 370 },
  { type: "out", text: "  v1 ─→ v2 ─→ v3  (current)", at: 400 },
  { type: "out", text: "  └── parent: (none — initial)", at: 410 },
];

const TYPE_SPEED = 1.3;
const FONT_SIZE = 38;
const LINE_HEIGHT = 1.5;
const LINE_PX = FONT_SIZE * LINE_HEIGHT;
const MAX_VISIBLE = 13;

export const CLIVersions: React.FC = () => {
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
              Terminal — aim versions
            </span>
          </div>

          <div
            style={{
              background: COLORS.terminalBg,
              padding: "36px 48px",
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
                    <div key={i} style={{ marginTop: i > 0 ? 12 : 0 }}>
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
