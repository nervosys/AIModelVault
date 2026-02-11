// Store & Encrypt composition — shows aim store workflow with encryption animation
import React from "react";
import {
  AbsoluteFill,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
  Sequence,
  Easing,
} from "remotion";
import { COLORS, FONTS } from "../theme";
import {
  GridBackground,
  Terminal,
  TerminalLine,
  SlideTitle,
  GlowBox,
  AnimatedCounter,
} from "../components/shared";

// ─── Encryption Bar ───────────────────────────────────────────────────────────

const EncryptionBar: React.FC<{ progress: number; label: string }> = ({
  progress,
  label,
}) => {
  return (
    <div style={{ marginBottom: 14 }}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          marginBottom: 4,
        }}
      >
        <span
          style={{
            fontFamily: FONTS.mono,
            fontSize: 14,
            color: COLORS.textMuted,
          }}
        >
          {label}
        </span>
        <span
          style={{
            fontFamily: FONTS.mono,
            fontSize: 14,
            color: COLORS.success,
          }}
        >
          {Math.round(progress * 100)}%
        </span>
      </div>
      <div
        style={{
          height: 6,
          borderRadius: 3,
          background: COLORS.bgLight,
          overflow: "hidden",
        }}
      >
        <div
          style={{
            height: "100%",
            width: `${progress * 100}%`,
            borderRadius: 3,
            background: `linear-gradient(90deg, ${COLORS.primary}, ${COLORS.success})`,
            boxShadow:
              progress > 0 ? `0 0 10px ${COLORS.glowGreen}` : "none",
            transition: "width 0.1s",
          }}
        />
      </div>
    </div>
  );
};

// ─── StoreAndEncrypt Composition ──────────────────────────────────────────────

export const StoreAndEncrypt: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Encryption progress bars
  const compressProgress = interpolate(frame, [120, 160], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const encryptProgress = interpolate(frame, [140, 185], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const checksumProgress = interpolate(frame, [165, 195], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });

  // Success card
  const successScale = spring({
    fps,
    frame: frame - 210,
    config: { damping: 12, mass: 0.8 },
  });

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
          gap: 30,
        }}
      >
        <SlideTitle
          title="Store & Encrypt"
          subtitle="Military-grade encryption with a single command"
        />

        <div
          style={{
            display: "flex",
            gap: 40,
            width: "100%",
            alignItems: "flex-start",
          }}
        >
          {/* Left: Terminal */}
          <div style={{ flex: 1 }}>
            <Terminal title="Terminal — aim store" width="100%">
              <Sequence from={10} layout="none">
                <TerminalLine
                  command="aim unlock"
                  output={["🔓 Vault unlocked successfully"]}
                  delay={0}
                  typeSpeed={2}
                />
              </Sequence>
              <Sequence from={50} layout="none">
                <div style={{ marginTop: 16 }}>
                  <TerminalLine
                    command="aim store llama-3.1-70b ./model.safetensors"
                    output={[
                      "📦 Format detected: SafeTensors",
                      "📐 Original size: 140.3 GB",
                    ]}
                    delay={0}
                    typeSpeed={1.2}
                  />
                </div>
              </Sequence>
            </Terminal>
          </div>

          {/* Right: Progress + stats */}
          <div style={{ flex: 1, paddingTop: 20 }}>
            <Sequence from={100} layout="none">
              <GlowBox glowColor={COLORS.glow} delay={0}>
                <h3
                  style={{
                    fontFamily: FONTS.sans,
                    fontSize: 20,
                    color: COLORS.primaryLight,
                    margin: "0 0 18px 0",
                    fontWeight: 700,
                  }}
                >
                  Encryption Pipeline
                </h3>
                <EncryptionBar
                  progress={compressProgress}
                  label="LZMA Compression"
                />
                <EncryptionBar
                  progress={encryptProgress}
                  label="AES-256-GCM Encryption"
                />
                <EncryptionBar
                  progress={checksumProgress}
                  label="SHA-256 Checksum"
                />
              </GlowBox>
            </Sequence>

            {/* Success result */}
            <Sequence from={210} layout="none">
              <div
                style={{
                  marginTop: 20,
                  transform: `scale(${successScale})`,
                }}
              >
                <GlowBox glowColor={COLORS.glowGreen}>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 12,
                      marginBottom: 10,
                    }}
                  >
                    <span style={{ fontSize: 28 }}>✅</span>
                    <span
                      style={{
                        fontFamily: FONTS.sans,
                        fontSize: 22,
                        color: COLORS.success,
                        fontWeight: 700,
                      }}
                    >
                      Stored Successfully
                    </span>
                  </div>
                  <div
                    style={{
                      fontFamily: FONTS.mono,
                      fontSize: 14,
                      color: COLORS.textMuted,
                      lineHeight: 1.8,
                    }}
                  >
                    <div>Version: 1 &nbsp;·&nbsp; Checkpoint: 3fa8c2e1</div>
                    <div>Compressed: 140.3 GB → 62.1 GB (55.7%)</div>
                    <div>Encryption: AES-256-GCM + Argon2id</div>
                  </div>
                </GlowBox>
              </div>
            </Sequence>
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
