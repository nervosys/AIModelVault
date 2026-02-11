// CLI Convert — `aim convert` format conversion demo
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
  { type: "cmd", text: "aim convert llama-3.1 --to gguf --quantize q4_k_m", at: 15 },
  { type: "out", text: "🔄 SafeTensors → GGUF (Q4_K_M)...", at: 72 },
  { type: "out", text: "   Source: v1 — 140.3 GB SafeTensors", at: 82 },
  { type: "out", text: "   ████████████████████████████ 100%", at: 100 },
  { type: "out", text: "✅ Converted! v2 (e4b2a91c)", at: 115, color: COLORS.terminalGreen },
  { type: "out", text: "   140.3 GB → 11.2 GB  (92% reduction)", at: 123 },
  { type: "cmd", text: "aim convert mistral-7b --to onnx", at: 160 },
  { type: "out", text: "🔄 GGUF → ONNX...", at: 200 },
  { type: "out", text: "   ████████████████████████████ 100%", at: 215 },
  { type: "out", text: "✅ Converted! v2 (7d1f3c8e)", at: 228, color: COLORS.terminalGreen },
  { type: "out", text: "   3.8 GB → 3.9 GB", at: 236 },
  { type: "cmd", text: "aim list --format", at: 265 },
  { type: "out", text: "  Model          Formats", at: 295 },
  { type: "out", text: "  ─────          ───────", at: 300 },
  { type: "out", text: "  llama-3.1-70b  SafeTensors, GGUF", at: 307 },
  { type: "out", text: "  mistral-7b     GGUF, ONNX", at: 314 },
  { type: "out", text: "  23+ formats supported", at: 326 },
];

const TYPE_SPEED = 1.2;
const FONT_SIZE = 38;
const LINE_HEIGHT = 1.5;
const LINE_PX = FONT_SIZE * LINE_HEIGHT;
const MAX_VISIBLE = 13;

export const CLIConvert: React.FC = () => {
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
              Terminal — aim convert
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
