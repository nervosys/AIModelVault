// Security Audit composition — FIPS / CVE / MITRE compliance checklist
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
  SlideTitle,
  GlowBox,
  CheckItem,
  AnimatedCounter,
} from "../components/shared";

// ─── Shield Icon ──────────────────────────────────────────────────────────────

const ShieldIcon: React.FC<{ progress: number }> = ({ progress }) => {
  const dashOffset = interpolate(progress, [0, 1], [200, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <svg width="80" height="80" viewBox="0 0 24 24" fill="none">
      <path
        d="M12 2L3 7v5c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V7l-9-5z"
        stroke={COLORS.success}
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill={`${COLORS.success}15`}
        strokeDasharray="200"
        strokeDashoffset={dashOffset}
      />
      {progress > 0.7 && (
        <path
          d="M9 12l2 2 4-4"
          stroke={COLORS.success}
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          opacity={interpolate(progress, [0.7, 1], [0, 1], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
          })}
        />
      )}
    </svg>
  );
};

// ─── CryptoDetail ─────────────────────────────────────────────────────────────

const CryptoDetail: React.FC<{
  label: string;
  value: string;
  delay: number;
}> = ({ label, value, delay }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const s = spring({ fps, frame: frame - delay, config: { damping: 12 } });

  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        padding: "8px 0",
        borderBottom: `1px solid ${COLORS.bgLight}`,
        opacity: s,
        transform: `translateX(${(1 - s) * 30}px)`,
      }}
    >
      <span
        style={{
          fontFamily: FONTS.sans,
          fontSize: 15,
          color: COLORS.textMuted,
        }}
      >
        {label}
      </span>
      <span
        style={{
          fontFamily: FONTS.mono,
          fontSize: 15,
          color: COLORS.primaryLight,
          fontWeight: 600,
        }}
      >
        {value}
      </span>
    </div>
  );
};

// ─── SecurityAudit Composition ────────────────────────────────────────────────

export const SecurityAudit: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const shieldProgress = interpolate(frame, [30, 90], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });

  // Score counter
  const scoreFrame = Math.max(0, frame - 200);

  const checks = [
    { label: "FIPS 140-3 Compliant Encryption", delay: 60 },
    { label: "AES-256-GCM Authenticated Encryption", delay: 80 },
    { label: "Argon2id Key Derivation (OWASP)", delay: 100 },
    { label: "Zero Unsafe Code Blocks", delay: 120 },
    { label: "Supply Chain Audit (cargo-deny)", delay: 140 },
    { label: "CVE Database Scan — 0 Vulnerabilities", delay: 160 },
    { label: "MITRE ATT&CK Coverage", delay: 180 },
  ];

  const cryptoDetails = [
    { label: "Cipher", value: "AES-256-GCM", delay: 60 },
    { label: "KDF", value: "Argon2id (19 MiB)", delay: 75 },
    { label: "Salt", value: "32-byte CSPRNG", delay: 90 },
    { label: "Nonce", value: "96-bit random", delay: 105 },
    { label: "MAC", value: "GHASH (128-bit)", delay: 120 },
    { label: "Audit", value: "Zero unsafe", delay: 135 },
  ];

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
          title="Security Audit"
          subtitle="Military-grade cryptography with full compliance"
        />

        <div
          style={{
            display: "flex",
            gap: 40,
            width: "100%",
            alignItems: "flex-start",
          }}
        >
          {/* Left: Compliance checklist */}
          <div style={{ flex: 1 }}>
            <GlowBox glowColor={COLORS.glowGreen}>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 16,
                  marginBottom: 20,
                }}
              >
                <ShieldIcon progress={shieldProgress} />
                <div>
                  <h3
                    style={{
                      fontFamily: FONTS.sans,
                      fontSize: 22,
                      color: COLORS.success,
                      margin: 0,
                      fontWeight: 700,
                    }}
                  >
                    Compliance Check
                  </h3>
                  <div
                    style={{
                      fontFamily: FONTS.mono,
                      fontSize: 13,
                      color: COLORS.textMuted,
                      marginTop: 4,
                    }}
                  >
                    aim compliance --verbose
                  </div>
                </div>
              </div>
              {checks.map((c, i) => (
                <CheckItem key={i} label={c.label} delay={c.delay} />
              ))}
            </GlowBox>
          </div>

          {/* Right: Crypto details + score */}
          <div style={{ flex: 0.85, display: "flex", flexDirection: "column", gap: 20 }}>
            <GlowBox glowColor={COLORS.glow}>
              <h3
                style={{
                  fontFamily: FONTS.sans,
                  fontSize: 20,
                  color: COLORS.primaryLight,
                  margin: "0 0 12px 0",
                  fontWeight: 700,
                }}
              >
                Cryptographic Details
              </h3>
              {cryptoDetails.map((d, i) => (
                <CryptoDetail key={i} {...d} />
              ))}
            </GlowBox>

            <Sequence from={200} layout="none">
              <GlowBox glowColor={COLORS.glowGreen}>
                <div
                  style={{
                    textAlign: "center",
                    padding: "10px 0",
                  }}
                >
                  <div
                    style={{
                      fontFamily: FONTS.sans,
                      fontSize: 14,
                      color: COLORS.textMuted,
                      textTransform: "uppercase",
                      letterSpacing: 2,
                      marginBottom: 6,
                    }}
                  >
                    Security Score
                  </div>
                  <div
                    style={{
                      fontFamily: FONTS.display,
                      fontSize: 64,
                      fontWeight: 800,
                      color: COLORS.success,
                    }}
                  >
                    <AnimatedCounter from={0} to={100} startFrame={200} duration={45} />
                  </div>
                  <div
                    style={{
                      fontFamily: FONTS.mono,
                      fontSize: 13,
                      color: COLORS.success,
                      marginTop: 4,
                    }}
                  >
                    7 / 7 checks passed · 0 vulnerabilities
                  </div>
                </div>
              </GlowBox>
            </Sequence>
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
