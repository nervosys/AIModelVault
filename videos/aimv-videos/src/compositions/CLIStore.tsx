// CLI Store — `aim store` + `aim list` demo
import React from "react";
import {
  AbsoluteFill,
  interpolate,
  useCurrentFrame,
  Easing,
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
  { type: "cmd", text: "aim store llama-3.1-70b ./model.safetensors", at: 20 },
  { type: "out", text: "📦 Format: SafeTensors  📐 140.3 GB", at: 78 },
  { type: "out", text: "🗜️  Compressing (LZMA)...  → 62.1 GB (55.7%)", at: 90 },
  { type: "out", text: "🔒 Encrypting (AES-256-GCM)...", at: 104 },
  { type: "out", text: "✅ Stored! v1 · 3fa8c2e1 · SHA ✓", at: 118, color: COLORS.terminalGreen },
  { type: "cmd", text: "aim store mistral-7b ./mistral.gguf", at: 155 },
  { type: "out", text: "📦 Format: GGUF  📐 4.1 GB", at: 200 },
  { type: "out", text: "🗜️  Compressing...  → 3.8 GB (7.3%)", at: 214 },
  { type: "out", text: "🔒 Encrypting...", at: 228 },
  { type: "out", text: "✅ Stored! v1 · c1e9ab5f", at: 242, color: COLORS.terminalGreen },
  { type: "cmd", text: "aim list", at: 275 },
  { type: "out", text: "  Model           Format       Size", at: 308 },
  { type: "out", text: "  ─────           ──────       ────", at: 313 },
  { type: "out", text: "  llama-3.1-70b   SafeTensors  62.1 GB", at: 320 },
  { type: "out", text: "  mistral-7b      GGUF          3.8 GB", at: 328 },
  { type: "out", text: "  Total: 2 models · 65.9 GB", at: 340 },
];

const TYPE_SPEED = 1.2;
const FONT_SIZE = 38;
const LINE_HEIGHT = 1.5;
const LINE_PX = FONT_SIZE * LINE_HEIGHT;
const MAX_VISIBLE = 13;

export const CLIStore: React.FC = () => {
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
              Terminal — aim store
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
