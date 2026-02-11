// Version Timeline composition — visual version history with time-travel
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
} from "../components/shared";

// ─── Version Node ─────────────────────────────────────────────────────────────

interface VersionNodeProps {
  version: number;
  hash: string;
  label: string;
  date: string;
  active: boolean;
  delay: number;
}

const VersionNode: React.FC<VersionNodeProps> = ({
  version,
  hash,
  label,
  date,
  active,
  delay,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const nodeScale = spring({
    fps,
    frame: frame - delay,
    config: { damping: 12, mass: 0.6 },
  });

  const glow = active
    ? interpolate(Math.sin(frame * 0.08), [-1, 1], [0.4, 1])
    : 0;

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 20 }}>
      {/* Dot + line */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          width: 30,
        }}
      >
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: "50%",
            background: active
              ? COLORS.primary
              : `${COLORS.textMuted}66`,
            boxShadow: active
              ? `0 0 ${12 + glow * 10}px ${COLORS.glow}`
              : "none",
            transform: `scale(${nodeScale})`,
            border: `2px solid ${active ? COLORS.primaryLight : COLORS.textMuted}`,
          }}
        />
        <div
          style={{
            width: 2,
            height: 50,
            background: `linear-gradient(${COLORS.bgLight}, transparent)`,
          }}
        />
      </div>

      {/* Info */}
      <div
        style={{
          transform: `scale(${nodeScale}) translateX(${(1 - nodeScale) * 20}px)`,
          opacity: nodeScale,
        }}
      >
        <div style={{ display: "flex", alignItems: "baseline", gap: 12 }}>
          <span
            style={{
              fontFamily: FONTS.sans,
              fontSize: 22,
              fontWeight: 700,
              color: active ? COLORS.text : COLORS.textMuted,
            }}
          >
            v{version}
          </span>
          <code
            style={{
              fontFamily: FONTS.mono,
              fontSize: 14,
              color: COLORS.primary,
              background: `${COLORS.primary}15`,
              padding: "2px 8px",
              borderRadius: 4,
            }}
          >
            {hash}
          </code>
        </div>
        <div
          style={{
            fontFamily: FONTS.sans,
            fontSize: 16,
            color: COLORS.text,
            marginTop: 4,
          }}
        >
          {label}
        </div>
        <div
          style={{
            fontFamily: FONTS.mono,
            fontSize: 12,
            color: COLORS.textMuted,
            marginTop: 2,
          }}
        >
          {date}
        </div>
      </div>
    </div>
  );
};

// ─── VersionTimeline Composition ──────────────────────────────────────────────

export const VersionTimeline: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const versions = [
    {
      version: 3,
      hash: "a7f3d21b",
      label: "Fine-tuned for medical imaging",
      date: "2025-01-15 14:32 UTC",
    },
    {
      version: 2,
      hash: "c1e9ab5f",
      label: "Quantized INT8 (4x smaller)",
      date: "2025-01-14 09:18 UTC",
    },
    {
      version: 1,
      hash: "3fa8c2e1",
      label: "Initial SafeTensors upload",
      date: "2025-01-13 15:42 UTC",
    },
  ];

  // Highlight animation: the "active" version shifts over time
  const activeVersion =
    frame < 180 ? 3 : frame < 220 ? 2 : 1;

  // Rollback success
  const rollbackScale = spring({
    fps,
    frame: frame - 230,
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
          padding: "50px 80px",
          gap: 20,
        }}
      >
        <SlideTitle
          title="Version Control"
          subtitle="Git-like versioning with instant rollback for every model"
        />

        <div
          style={{
            display: "flex",
            gap: 50,
            width: "100%",
            alignItems: "flex-start",
          }}
        >
          {/* Left: Terminal */}
          <div style={{ flex: 1 }}>
            <Terminal title="Terminal — aim versions" width="100%">
              <Sequence from={10} layout="none">
                <TerminalLine
                  command="aim versions llama-3.1-70b"
                  output={[
                    "   Version  Checkpoint   Date",
                    "   ───────  ──────────   ────────────────",
                    " ▸ v3       a7f3d21b     2025-01-15 14:32",
                    "   v2       c1e9ab5f     2025-01-14 09:18",
                    "   v1       3fa8c2e1     2025-01-13 15:42",
                  ]}
                  delay={0}
                  typeSpeed={1.5}
                />
              </Sequence>

              <Sequence from={155} layout="none">
                <div style={{ marginTop: 16 }}>
                  <TerminalLine
                    command="aim get llama-3.1-70b --version 1"
                    output={[
                      "🔄 Rolling back to v1 (3fa8c2e1)...",
                      "✅ Restored: ./llama-3.1-70b.safetensors",
                    ]}
                    delay={0}
                    typeSpeed={1.5}
                  />
                </div>
              </Sequence>
            </Terminal>
          </div>

          {/* Right: Visual timeline */}
          <div style={{ flex: 0.8, paddingTop: 10 }}>
            <Sequence from={60} layout="none">
              <div>
                {versions.map((v, i) => (
                  <VersionNode
                    key={v.version}
                    {...v}
                    active={v.version === activeVersion}
                    delay={60 + i * 20}
                  />
                ))}
              </div>
            </Sequence>

            {/* Rollback success */}
            <Sequence from={230} layout="none">
              <div style={{ transform: `scale(${rollbackScale})`, marginTop: 10 }}>
                <GlowBox glowColor={COLORS.glowGreen}>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 10,
                    }}
                  >
                    <span style={{ fontSize: 24 }}>⏪</span>
                    <span
                      style={{
                        fontFamily: FONTS.sans,
                        fontSize: 18,
                        color: COLORS.success,
                        fontWeight: 700,
                      }}
                    >
                      Rolled back to v1
                    </span>
                  </div>
                  <div
                    style={{
                      fontFamily: FONTS.mono,
                      fontSize: 13,
                      color: COLORS.textMuted,
                      marginTop: 6,
                    }}
                  >
                    All versions preserved · Lineage intact
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
