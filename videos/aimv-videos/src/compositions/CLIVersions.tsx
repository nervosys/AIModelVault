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
  { type: "cmd", text: "aim store llama-3.1 ./v2-quantized.gguf", at: 15 },
  { type: "out", text: "📦 Format: GGUF · 11.2 GB", at: 60 },
  { type: "out", text: "✅ Stored v2 (c1e9ab5f)", at: 70, color: COLORS.terminalGreen },

  { type: "cmd", text: "aim store llama-3.1 ./v3-finetuned.safetensors", at: 100 },
  { type: "out", text: "📦 Format: SafeTensors · 140.3 GB", at: 155 },
  { type: "out", text: "✅ Stored v3 (a7f3d21b)", at: 165, color: COLORS.terminalGreen },

  { type: "cmd", text: "aim versions llama-3.1", at: 195 },
  { type: "out", text: "  Version  Checkpoint  Format       Size      Date", at: 228 },
  { type: "out", text: "  ───────  ──────────  ──────       ────      ────────────────", at: 233 },
  { type: "out", text: "▸ v3       a7f3d21b    SafeTensors  140.3 GB  2025-01-15 14:32", at: 239 },
  { type: "out", text: "  v2       c1e9ab5f    GGUF          11.2 GB  2025-01-14 09:18", at: 245 },
  { type: "out", text: "  v1       3fa8c2e1    SafeTensors  140.3 GB  2025-01-13 15:42", at: 251 },

  { type: "cmd", text: "aim get llama-3.1 --version 1 -o ./rollback/", at: 280 },
  { type: "out", text: "🔓 Decrypting v1 (3fa8c2e1)...", at: 325 },
  { type: "out", text: "🗜️  Decompressing...", at: 335 },
  { type: "out", text: "✅ Restored → ./rollback/llama-3.1.safetensors", at: 348, color: COLORS.terminalGreen },
  { type: "out", text: "   SHA-256 verified ✓", at: 358, color: COLORS.terminalGreen },

  { type: "cmd", text: "aim lineage llama-3.1", at: 385 },
  { type: "out", text: "  v1 ─→ v2 ─→ v3  (current)", at: 415 },
  { type: "out", text: "  │", at: 421 },
  { type: "out", text: "  └── parent: (none — initial upload)", at: 427 },
];

const TYPE_SPEED = 1.3;

export const CLIVersions: React.FC = () => {
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
          Versions &amp; Rollback
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
          Git-like version control with instant time-travel
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
              Terminal — aim versions
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
