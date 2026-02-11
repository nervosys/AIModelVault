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

interface Line {
  type: "cmd" | "out";
  text: string;
  at: number;
}

const LINES: Line[] = [
  { type: "cmd", text: "aim init --encryption aes-256-gcm", at: 20 },
  { type: "out", text: "🔐 Creating new vault...", at: 70 },
  { type: "out", text: "   Location: ~/.local/share/aim/vault", at: 78 },
  { type: "out", text: "   Cipher:   AES-256-GCM", at: 86 },
  { type: "out", text: "   KDF:      Argon2id (19 MiB, 2 iter)", at: 94 },
  { type: "out", text: "   Salt:     32-byte CSPRNG", at: 102 },
  { type: "out", text: "✅ Vault initialized!", at: 114 },
  { type: "cmd", text: "aim unlock", at: 150 },
  { type: "out", text: "🔑 Passphrase: ••••••••••••", at: 180 },
  { type: "out", text: "🔓 Vault unlocked (30 min session)", at: 195 },
  { type: "cmd", text: "aim status", at: 225 },
  { type: "out", text: "Vault:       unlocked", at: 258 },
  { type: "out", text: "Models:      0", at: 265 },
  { type: "out", text: "Encryption:  AES-256-GCM", at: 272 },
  { type: "out", text: "Compliance:  FIPS 140-3 ✓", at: 279 },
];

const TYPE_SPEED = 1.5;
const FONT_SIZE = 40;
const LINE_HEIGHT = 1.5;
const LINE_PX = FONT_SIZE * LINE_HEIGHT;
const MAX_VISIBLE = 13;

export const CLIInit: React.FC = () => {
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
              Terminal — aim init
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
                  <div key={i} style={{ color: COLORS.text, opacity }}>
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
