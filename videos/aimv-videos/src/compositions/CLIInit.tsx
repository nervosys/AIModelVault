// CLI Init — `aim init` + `aim unlock` demo
import React from "react";
import {
  AbsoluteFill,
  interpolate,
  useCurrentFrame,
  useVideoConfig,
  Easing,
} from "remotion";
import { COLORS, FONTS } from "../theme";
import { GridBackground } from "../components/shared";

// ─── Inline terminal typing (no Sequence needed) ─────────────────────────────

interface Line {
  type: "cmd" | "out";
  text: string;
  /** Absolute frame this line starts at */
  at: number;
}

const LINES: Line[] = [
  { type: "cmd", text: "aim init --encryption aes-256-gcm", at: 20 },
  { type: "out", text: "🔐 Creating new vault...", at: 70 },
  { type: "out", text: "   Location: ~/.local/share/aim/vault", at: 78 },
  { type: "out", text: "   Cipher:   AES-256-GCM", at: 86 },
  { type: "out", text: "   KDF:      Argon2id (19 MiB, 2 iterations)", at: 94 },
  { type: "out", text: "   Salt:     32-byte CSPRNG", at: 102 },
  { type: "out", text: "✅ Vault initialized successfully!", at: 114 },
  { type: "cmd", text: "aim unlock", at: 150 },
  { type: "out", text: "🔑 Enter passphrase: ••••••••••••", at: 180 },
  { type: "out", text: "🔓 Vault unlocked (session: 30 min)", at: 195 },
  { type: "cmd", text: "aim status", at: 225 },
  { type: "out", text: "Vault:       unlocked", at: 258 },
  { type: "out", text: "Models:      0", at: 265 },
  { type: "out", text: "Encryption:  AES-256-GCM", at: 272 },
  { type: "out", text: "Compliance:  FIPS 140-3 ✓", at: 279 },
];

const TYPE_SPEED = 1.5; // frames per char for commands

export const CLIInit: React.FC = () => {
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
          Initialize &amp; Unlock
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
          Set up a military-grade encrypted vault in seconds
        </div>

        {/* Terminal */}
        <div
          style={{
            width: "85%",
            borderRadius: 14,
            overflow: "hidden",
            boxShadow: `0 25px 60px rgba(0,0,0,0.6), 0 0 40px ${COLORS.glow}`,
            border: `1px solid ${COLORS.border}`,
          }}
        >
          {/* Title bar */}
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
              Terminal — aim init
            </span>
          </div>

          {/* Body */}
          <div
            style={{
              background: COLORS.terminalBg,
              padding: "24px 28px",
              fontFamily: FONTS.mono,
              fontSize: 17,
              lineHeight: 1.8,
              color: COLORS.text,
              minHeight: 340,
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
                    {!done && (
                      <span style={{ color: COLORS.text }}>▌</span>
                    )}
                  </div>
                );
              }
              // output line
              const show = frame >= line.at;
              if (!show) return null;
              const opacity = interpolate(
                frame - line.at,
                [0, 5],
                [0, 1],
                { extrapolateRight: "clamp" }
              );
              return (
                <div key={i} style={{ color: COLORS.text, opacity }}>
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
