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
  { type: "out", text: "📦 Format detected: SafeTensors", at: 78 },
  { type: "out", text: "📐 Size: 140.3 GB", at: 86 },
  { type: "out", text: "🗜️  Compressing (LZMA)...  140.3 GB → 62.1 GB  (55.7%)", at: 96 },
  { type: "out", text: "🔒 Encrypting (AES-256-GCM)...", at: 110 },
  { type: "out", text: "🔑 KDF: Argon2id (19 MiB, 2 iterations)", at: 120 },
  { type: "out", text: "✅ Stored successfully!", at: 136, color: COLORS.terminalGreen },
  { type: "out", text: "   Version:    1", at: 144 },
  { type: "out", text: "   Checkpoint: 3fa8c2e1", at: 150 },
  { type: "out", text: "   SHA-256:    a7c3…f912", at: 156 },
  { type: "cmd", text: "aim store mistral-7b ./mistral.gguf", at: 185 },
  { type: "out", text: "📦 Format detected: GGUF", at: 230 },
  { type: "out", text: "📐 Size: 4.1 GB", at: 238 },
  { type: "out", text: "🗜️  Compressing...  4.1 GB → 3.8 GB  (7.3%)", at: 248 },
  { type: "out", text: "🔒 Encrypting...", at: 260 },
  { type: "out", text: "✅ Stored successfully!  v1 · c1e9ab5f", at: 272, color: COLORS.terminalGreen },
  { type: "cmd", text: "aim list", at: 300 },
  { type: "out", text: "  Model             Format       Versions  Size", at: 335 },
  { type: "out", text: "  ─────             ──────       ────────  ────", at: 340 },
  { type: "out", text: "  llama-3.1-70b     SafeTensors  1         62.1 GB", at: 346 },
  { type: "out", text: "  mistral-7b        GGUF         1          3.8 GB", at: 352 },
  { type: "out", text: "", at: 358 },
  { type: "out", text: "  Total: 2 models · 65.9 GB stored", at: 362 },
];

const TYPE_SPEED = 1.2;

export const CLIStore: React.FC = () => {
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
          padding: 80,
        }}
      >
        {/* Title */}
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
          Store &amp; List Models
        </div>
        <div
          style={{
            fontFamily: FONTS.sans,
            fontSize: 20,
            color: COLORS.textMuted,
            marginBottom: 36,
            opacity: interpolate(frame, [5, 18], [0, 1], {
              extrapolateRight: "clamp",
            }),
          }}
        >
          Auto-detect format, compress, encrypt — one command
        </div>

        {/* Terminal */}
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
              Terminal — aim store
            </span>
          </div>

          <div
            style={{
              background: COLORS.terminalBg,
              padding: "24px 28px",
              fontFamily: FONTS.mono,
              fontSize: 16,
              lineHeight: 1.75,
              color: COLORS.text,
              minHeight: 380,
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
