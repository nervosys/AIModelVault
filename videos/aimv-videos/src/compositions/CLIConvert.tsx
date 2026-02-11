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
  { type: "out", text: "🔄 Converting SafeTensors → GGUF (Q4_K_M)...", at: 72 },
  { type: "out", text: "   Source:  v1 (3fa8c2e1) — 140.3 GB SafeTensors", at: 82 },
  { type: "out", text: "   Target:  GGUF with Q4_K_M quantization", at: 90 },
  { type: "out", text: "   ████████████████████████████████████████  100%", at: 105 },
  { type: "out", text: "✅ Converted! Stored as v2 (e4b2a91c)", at: 120, color: COLORS.terminalGreen },
  { type: "out", text: "   Size:    140.3 GB → 11.2 GB  (92.0% reduction)", at: 128 },
  { type: "out", text: "   SHA-256: b2f4…8a31 ✓", at: 136, color: COLORS.terminalGreen },

  { type: "cmd", text: "aim convert mistral-7b --to onnx", at: 170 },
  { type: "out", text: "🔄 Converting GGUF → ONNX...", at: 215 },
  { type: "out", text: "   ████████████████████████████████████████  100%", at: 228 },
  { type: "out", text: "✅ Converted! Stored as v2 (7d1f3c8e)", at: 240, color: COLORS.terminalGreen },
  { type: "out", text: "   Size:    3.8 GB → 3.9 GB", at: 248 },

  { type: "cmd", text: "aim list --format", at: 275 },
  { type: "out", text: "  Model            Versions  Formats", at: 305 },
  { type: "out", text: "  ─────            ────────  ───────", at: 310 },
  { type: "out", text: "  llama-3.1-70b    2         SafeTensors, GGUF", at: 316 },
  { type: "out", text: "  mistral-7b       2         GGUF, ONNX", at: 322 },
  { type: "out", text: "", at: 328 },
  { type: "out", text: "  Supported formats: SafeTensors, PyTorch, ONNX, GGUF,", at: 332 },
  { type: "out", text: "    GGML, TFLite, Keras, HDF5, CoreML, MLX, JAX,", at: 338 },
  { type: "out", text: "    NumPy, Parquet, Arrow, MsgPack, CBOR, Protobuf,", at: 344 },
  { type: "out", text: "    Bincode, BSON, FlexBuffers, JSON  (23+ total)", at: 350 },
];

const TYPE_SPEED = 1.2;

export const CLIConvert: React.FC = () => {
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
          padding: 24,
        }}
      >
        <div
          style={{
            width: "100%",
            height: "100%",
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
              padding: "14px 20px",
              display: "flex",
              alignItems: "center",
              gap: 10,
            }}
          >
            <div style={{ display: "flex", gap: 8 }}>
              {["#ff5f57", "#febc2e", "#28c840"].map((c) => (
                <div
                  key={c}
                  style={{
                    width: 14,
                    height: 14,
                    borderRadius: "50%",
                    background: c,
                  }}
                />
              ))}
            </div>
            <span
              style={{
                color: COLORS.textMuted,
                fontSize: 16,
                fontFamily: FONTS.mono,
                marginLeft: 8,
              }}
            >
              Terminal — aim convert
            </span>
          </div>

          <div
            style={{
              background: COLORS.terminalBg,
              padding: "32px 40px",
              fontFamily: FONTS.mono,
              fontSize: 24,
              lineHeight: 1.55,
              color: COLORS.text,
              flex: 1,
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
