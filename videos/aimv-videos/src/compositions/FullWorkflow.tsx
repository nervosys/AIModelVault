// Full Workflow composition — end-to-end AIMV demo in one scene
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
  Badge,
  CheckItem,
} from "../components/shared";

// ─── Step Indicator ───────────────────────────────────────────────────────────

const StepIndicator: React.FC<{
  steps: string[];
  activeStep: number;
}> = ({ steps, activeStep }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  return (
    <div
      style={{
        display: "flex",
        gap: 6,
        alignItems: "center",
        justifyContent: "center",
        marginBottom: 20,
      }}
    >
      {steps.map((step, i) => {
        const isActive = i === activeStep;
        const isDone = i < activeStep;
        return (
          <React.Fragment key={i}>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
              }}
            >
              <div
                style={{
                  width: 28,
                  height: 28,
                  borderRadius: "50%",
                  background: isDone
                    ? COLORS.success
                    : isActive
                      ? COLORS.primary
                      : COLORS.bgLight,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontFamily: FONTS.mono,
                  fontSize: 13,
                  fontWeight: 700,
                  color: isDone || isActive ? COLORS.bg : COLORS.textMuted,
                  border: `2px solid ${isDone ? COLORS.success : isActive ? COLORS.primary : COLORS.textMuted}40`,
                  boxShadow: isActive
                    ? `0 0 12px ${COLORS.glow}`
                    : "none",
                }}
              >
                {isDone ? "✓" : i + 1}
              </div>
              <span
                style={{
                  fontFamily: FONTS.sans,
                  fontSize: 13,
                  color: isActive
                    ? COLORS.text
                    : isDone
                      ? COLORS.success
                      : COLORS.textMuted,
                  fontWeight: isActive ? 600 : 400,
                }}
              >
                {step}
              </span>
            </div>
            {i < steps.length - 1 && (
              <div
                style={{
                  width: 30,
                  height: 2,
                  background: isDone
                    ? COLORS.success
                    : `${COLORS.textMuted}40`,
                }}
              />
            )}
          </React.Fragment>
        );
      })}
    </div>
  );
};

// ─── FullWorkflow Composition ─────────────────────────────────────────────────

export const FullWorkflow: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const steps = ["Init", "Store", "Analyze", "Convert", "Comply"];
  const activeStep =
    frame < 60
      ? 0
      : frame < 130
        ? 1
        : frame < 200
          ? 2
          : frame < 270
            ? 3
            : 4;

  // Final badge
  const finalScale = spring({
    fps,
    frame: frame - 330,
    config: { damping: 10, mass: 0.8 },
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
          padding: "40px 80px",
          gap: 16,
        }}
      >
        <SlideTitle
          title="Full Workflow"
          subtitle="From zero to production-ready in 60 seconds"
        />

        <StepIndicator steps={steps} activeStep={activeStep} />

        <div
          style={{
            display: "flex",
            gap: 40,
            width: "100%",
            alignItems: "flex-start",
          }}
        >
          {/* Terminal */}
          <div style={{ flex: 1.2 }}>
            <Terminal title="Terminal — Full Workflow" width="100%">
              {/* Step 1: Init */}
              <Sequence from={10} layout="none">
                <TerminalLine
                  command="aim init --encryption aes-256-gcm"
                  output={["🔐 Vault initialized at ~/.local/share/aim/vault"]}
                  delay={0}
                  typeSpeed={1.5}
                />
              </Sequence>

              {/* Step 2: Store */}
              <Sequence from={60} layout="none">
                <div style={{ marginTop: 12 }}>
                  <TerminalLine
                    command="aim store llama-3.1 ./model.safetensors"
                    output={[
                      "📦 Stored v1 (3fa8c2e1) · 140GB → 62GB",
                    ]}
                    delay={0}
                    typeSpeed={1.5}
                  />
                </div>
              </Sequence>

              {/* Step 3: Analyze */}
              <Sequence from={130} layout="none">
                <div style={{ marginTop: 12 }}>
                  <TerminalLine
                    command="aim analyze llama-3.1"
                    output={[
                      "📊 Params: 70B · Layers: 80 · Format: SafeTensors",
                      "   Entropy: 0.89 · Quantizable: yes (INT8/INT4)",
                    ]}
                    delay={0}
                    typeSpeed={1.5}
                  />
                </div>
              </Sequence>

              {/* Step 4: Convert */}
              <Sequence from={200} layout="none">
                <div style={{ marginTop: 12 }}>
                  <TerminalLine
                    command="aim convert llama-3.1 --to gguf --quantize q4_k_m"
                    output={[
                      "🔄 Converting SafeTensors → GGUF (Q4_K_M)...",
                      "✅ Stored v2 · 62GB → 11.2GB",
                    ]}
                    delay={0}
                    typeSpeed={1.5}
                  />
                </div>
              </Sequence>

              {/* Step 5: Compliance */}
              <Sequence from={270} layout="none">
                <div style={{ marginTop: 12 }}>
                  <TerminalLine
                    command="aim compliance llama-3.1"
                    output={[
                      "🛡️  FIPS 140-3 ✓  CVE Scan ✓  MITRE ✓",
                      "✅ Score: 100/100 · Production Ready",
                    ]}
                    delay={0}
                    typeSpeed={1.5}
                  />
                </div>
              </Sequence>
            </Terminal>
          </div>

          {/* Right sidebar: Live stats */}
          <div style={{ flex: 0.7, display: "flex", flexDirection: "column", gap: 16 }}>
            <Sequence from={30} layout="none">
              <GlowBox glowColor={COLORS.glow}>
                <h4
                  style={{
                    fontFamily: FONTS.sans,
                    fontSize: 18,
                    color: COLORS.primaryLight,
                    margin: "0 0 12px 0",
                    fontWeight: 700,
                  }}
                >
                  Pipeline Progress
                </h4>
                <CheckItem label="Vault initialized" delay={30} />
                <CheckItem label="Model stored & encrypted" delay={100} />
                <CheckItem label="Analysis complete" delay={170} />
                <CheckItem label="Format converted" delay={240} />
                <CheckItem label="Compliance verified" delay={310} />
              </GlowBox>
            </Sequence>

            <Sequence from={330} layout="none">
              <div style={{ transform: `scale(${finalScale})` }}>
                <GlowBox glowColor={COLORS.glowGreen}>
                  <div style={{ textAlign: "center", padding: "8px 0" }}>
                    <div style={{ fontSize: 36 }}>🚀</div>
                    <div
                      style={{
                        fontFamily: FONTS.sans,
                        fontSize: 22,
                        fontWeight: 700,
                        color: COLORS.success,
                        marginTop: 6,
                      }}
                    >
                      Production Ready
                    </div>
                    <div
                      style={{
                        fontFamily: FONTS.mono,
                        fontSize: 13,
                        color: COLORS.textMuted,
                        marginTop: 6,
                      }}
                    >
                      cargo install ai-model-vault
                    </div>
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
