// Intro composition — Logo reveal + feature taglines
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
import { GridBackground, Badge } from "../components/shared";

// ─── Lock Icon (SVG) ──────────────────────────────────────────────────────────

const LockIcon: React.FC<{ scale: number; glowOpacity: number }> = ({
  scale,
  glowOpacity,
}) => (
  <div
    style={{
      transform: `scale(${scale})`,
      filter: `drop-shadow(0 0 30px rgba(59,130,246,${glowOpacity}))`,
    }}
  >
    <svg width="120" height="120" viewBox="0 0 24 24" fill="none">
      <rect
        x="3"
        y="11"
        width="18"
        height="11"
        rx="2"
        fill={COLORS.primary}
        opacity={0.9}
      />
      <path
        d="M7 11V7a5 5 0 0 1 10 0v4"
        stroke={COLORS.primaryLight}
        strokeWidth="2"
        strokeLinecap="round"
        fill="none"
      />
      <circle cx="12" cy="16" r="1.5" fill="#fff" />
    </svg>
  </div>
);

// ─── Intro Composition ────────────────────────────────────────────────────────

export const Intro: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Phase 1: Lock icon appears (0-30)
  const lockScale = spring({
    fps,
    frame,
    config: { damping: 8, mass: 0.8, stiffness: 120 },
  });
  const lockGlow = interpolate(frame, [15, 40], [0, 0.6], {
    extrapolateRight: "clamp",
  });

  // Phase 2: Title text (20-50)
  const titleOpacity = interpolate(frame, [20, 40], [0, 1], {
    extrapolateRight: "clamp",
  });
  const titleY = interpolate(frame, [20, 45], [30, 0], {
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });

  // Phase 3: Subtitle (40-60)
  const subOpacity = interpolate(frame, [40, 55], [0, 1], {
    extrapolateRight: "clamp",
  });

  // Phase 4: Version badge (55-70)
  const versionScale = spring({
    fps,
    frame: frame - 55,
    config: { damping: 12 },
  });

  // Phase 5: Feature badges appear staggered (70+)
  const features = [
    "FIPS 140-3",
    "23+ Formats",
    "Version Control",
    "GraphQL API",
    "GPU Encryption",
    "Federation",
    "Blockchain Audit",
  ];

  // Phase 6: Tagline (110+)
  const taglineOpacity = interpolate(frame, [110, 130], [0, 1], {
    extrapolateRight: "clamp",
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
          gap: 20,
        }}
      >
        {/* Lock Icon */}
        <LockIcon scale={lockScale} glowOpacity={lockGlow} />

        {/* Title */}
        <h1
          style={{
            fontFamily: FONTS.display,
            fontSize: 80,
            fontWeight: 900,
            color: COLORS.text,
            margin: 0,
            transform: `translateY(${titleY}px)`,
            opacity: titleOpacity,
            letterSpacing: -2,
          }}
        >
          AI Model{" "}
          <span style={{ color: COLORS.primary }}>Vault</span>
        </h1>

        {/* Subtitle */}
        <p
          style={{
            fontFamily: FONTS.sans,
            fontSize: 28,
            color: COLORS.textMuted,
            margin: 0,
            opacity: subOpacity,
            letterSpacing: 2,
            textTransform: "uppercase",
          }}
        >
          Military-Grade Security for AI Models
        </p>

        {/* Version badge */}
        <div style={{ transform: `scale(${versionScale})`, marginTop: 8 }}>
          <span
            style={{
              display: "inline-block",
              padding: "8px 24px",
              borderRadius: 24,
              background: `linear-gradient(135deg, ${COLORS.primaryDark}, ${COLORS.primary})`,
              color: "#fff",
              fontSize: 18,
              fontFamily: FONTS.mono,
              fontWeight: 700,
              boxShadow: `0 4px 20px ${COLORS.glow}`,
            }}
          >
            v1.1.0
          </span>
        </div>

        {/* Feature badges */}
        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            justifyContent: "center",
            gap: 8,
            maxWidth: 800,
            marginTop: 24,
          }}
        >
          {features.map((feat, i) => (
            <Badge
              key={feat}
              label={feat}
              delay={70 + i * 6}
              bgColor={i < 3 ? COLORS.bgCard : "#1a2744"}
              color={COLORS.primaryLight}
            />
          ))}
        </div>

        {/* Tagline */}
        <p
          style={{
            fontFamily: FONTS.mono,
            fontSize: 18,
            color: COLORS.textDim,
            margin: 0,
            marginTop: 30,
            opacity: taglineOpacity,
          }}
        >
          cargo install ai-model-vault &nbsp;·&nbsp; pip install neuralvault
        </p>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
