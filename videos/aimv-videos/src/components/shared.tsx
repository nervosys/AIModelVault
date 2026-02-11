// Shared reusable components for AIMV videos

import React from "react";
import {
  AbsoluteFill,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
  Easing,
} from "remotion";
import { COLORS, FONTS } from "../theme";

// ─── Terminal Window ──────────────────────────────────────────────────────────

type TerminalProps = {
  children: React.ReactNode;
  title?: string;
  width?: string | number;
};

export const Terminal: React.FC<TerminalProps> = ({
  children,
  title = "Terminal — aim",
  width = "80%",
}) => {
  return (
    <div
      style={{
        width,
        borderRadius: 12,
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
          <div
            style={{
              width: 12,
              height: 12,
              borderRadius: "50%",
              background: "#ff5f57",
            }}
          />
          <div
            style={{
              width: 12,
              height: 12,
              borderRadius: "50%",
              background: "#febc2e",
            }}
          />
          <div
            style={{
              width: 12,
              height: 12,
              borderRadius: "50%",
              background: "#28c840",
            }}
          />
        </div>
        <span
          style={{
            color: COLORS.textMuted,
            fontSize: 13,
            fontFamily: FONTS.mono,
            marginLeft: 8,
          }}
        >
          {title}
        </span>
      </div>
      {/* Terminal body */}
      <div
        style={{
          background: COLORS.terminalBg,
          padding: "20px 24px",
          fontFamily: FONTS.mono,
          fontSize: 18,
          lineHeight: 1.7,
          color: COLORS.text,
          minHeight: 120,
        }}
      >
        {children}
      </div>
    </div>
  );
};

// ─── Terminal Line (typing animation) ─────────────────────────────────────────

type TerminalLineProps = {
  command: string;
  output?: string[];
  delay?: number; // frames before this line starts appearing
  typeSpeed?: number; // frames per character
  prompt?: string;
  outputColor?: string;
};

export const TerminalLine: React.FC<TerminalLineProps> = ({
  command,
  output = [],
  delay = 0,
  typeSpeed = 1.5,
  prompt = "$ ",
  outputColor = COLORS.text,
}) => {
  const frame = useCurrentFrame();
  const adjustedFrame = Math.max(0, frame - delay);
  const charsTyped = Math.floor(adjustedFrame / typeSpeed);
  const visibleCommand = command.slice(0, charsTyped);
  const commandDone = charsTyped >= command.length;
  const framesAfterCommand = commandDone
    ? adjustedFrame - command.length * typeSpeed
    : 0;

  return (
    <div>
      <div>
        <span style={{ color: COLORS.terminalPrompt }}>{prompt}</span>
        <span style={{ color: COLORS.terminalGreen }}>{visibleCommand}</span>
        {!commandDone && adjustedFrame > 0 && (
          <span
            style={{
              color: COLORS.text,
              animation: "blink 1s step-end infinite",
            }}
          >
            ▌
          </span>
        )}
      </div>
      {commandDone &&
        output.map((line, i) => {
          const lineDelay = (i + 1) * 4;
          const show = framesAfterCommand > lineDelay;
          return show ? (
            <div
              key={i}
              style={{
                color: outputColor,
                opacity: interpolate(
                  framesAfterCommand - lineDelay,
                  [0, 6],
                  [0, 1],
                  { extrapolateRight: "clamp" }
                ),
              }}
            >
              {line}
            </div>
          ) : null;
        })}
    </div>
  );
};

// ─── Badge ────────────────────────────────────────────────────────────────────

type BadgeProps = {
  label: string;
  color?: string;
  bgColor?: string;
  delay?: number;
};

export const Badge: React.FC<BadgeProps> = ({
  label,
  color = COLORS.text,
  bgColor = COLORS.primaryDark,
  delay = 0,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const scale = spring({
    fps,
    frame: frame - delay,
    config: { damping: 12, mass: 0.5 },
  });

  return (
    <span
      style={{
        display: "inline-block",
        padding: "6px 16px",
        borderRadius: 20,
        background: bgColor,
        color,
        fontSize: 16,
        fontFamily: FONTS.mono,
        fontWeight: 600,
        transform: `scale(${scale})`,
        margin: "4px 4px",
        border: `1px solid ${color}33`,
      }}
    >
      {label}
    </span>
  );
};

// ─── Animated Counter ─────────────────────────────────────────────────────────

type CounterProps = {
  from: number;
  to: number;
  startFrame: number;
  duration: number;
  suffix?: string;
  prefix?: string;
  color?: string;
  fontSize?: number;
};

export const AnimatedCounter: React.FC<CounterProps> = ({
  from,
  to,
  startFrame,
  duration,
  suffix = "",
  prefix = "",
  color = COLORS.primary,
  fontSize = 72,
}) => {
  const frame = useCurrentFrame();
  const progress = interpolate(frame, [startFrame, startFrame + duration], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const value = Math.floor(from + (to - from) * progress);

  return (
    <span
      style={{
        fontFamily: FONTS.mono,
        fontSize,
        fontWeight: 800,
        color,
      }}
    >
      {prefix}
      {value.toLocaleString()}
      {suffix}
    </span>
  );
};

// ─── Glow Box ─────────────────────────────────────────────────────────────────

type GlowBoxProps = {
  children: React.ReactNode;
  glowColor?: string;
  delay?: number;
};

export const GlowBox: React.FC<GlowBoxProps> = ({
  children,
  glowColor = COLORS.glow,
  delay = 0,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const appear = spring({
    fps,
    frame: frame - delay,
    config: { damping: 15, mass: 0.8 },
  });

  return (
    <div
      style={{
        background: COLORS.bgCard,
        border: `1px solid ${COLORS.border}`,
        borderRadius: 16,
        padding: "28px 36px",
        boxShadow: `0 0 40px ${glowColor}`,
        transform: `scale(${appear})`,
        opacity: appear,
      }}
    >
      {children}
    </div>
  );
};

// ─── Slide Title ──────────────────────────────────────────────────────────────

type SlideTitleProps = {
  title: string;
  subtitle?: string;
};

export const SlideTitle: React.FC<SlideTitleProps> = ({ title, subtitle }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const titleY = interpolate(frame, [0, 20], [40, 0], {
    extrapolateRight: "clamp",
    easing: Easing.out(Easing.cubic),
  });
  const titleOpacity = interpolate(frame, [0, 15], [0, 1], {
    extrapolateRight: "clamp",
  });
  const subOpacity = interpolate(frame, [10, 25], [0, 1], {
    extrapolateRight: "clamp",
  });

  return (
    <div style={{ textAlign: "center", marginBottom: 40 }}>
      <h1
        style={{
          fontFamily: FONTS.display,
          fontSize: 56,
          fontWeight: 800,
          color: COLORS.text,
          margin: 0,
          transform: `translateY(${titleY}px)`,
          opacity: titleOpacity,
          letterSpacing: -1,
        }}
      >
        {title}
      </h1>
      {subtitle && (
        <p
          style={{
            fontFamily: FONTS.sans,
            fontSize: 24,
            color: COLORS.textMuted,
            marginTop: 12,
            opacity: subOpacity,
          }}
        >
          {subtitle}
        </p>
      )}
    </div>
  );
};

// ─── Background Grid ──────────────────────────────────────────────────────────

export const GridBackground: React.FC = () => {
  return (
    <AbsoluteFill
      style={{
        background: `
          radial-gradient(ellipse at 50% 0%, rgba(59,130,246,0.08) 0%, transparent 50%),
          ${COLORS.bg}
        `,
      }}
    >
      {/* Subtle dot grid */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          backgroundImage: `radial-gradient(${COLORS.border} 1px, transparent 1px)`,
          backgroundSize: "40px 40px",
          opacity: 0.4,
        }}
      />
    </AbsoluteFill>
  );
};

// ─── Check Mark Animation ─────────────────────────────────────────────────────

type CheckItemProps = {
  label: string;
  delay: number;
  color?: string;
};

export const CheckItem: React.FC<CheckItemProps> = ({
  label,
  delay,
  color = COLORS.success,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const scale = spring({
    fps,
    frame: frame - delay,
    config: { damping: 10, mass: 0.6, stiffness: 200 },
  });

  const opacity = interpolate(frame - delay, [0, 10], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 14,
        opacity,
        transform: `scale(${scale})`,
        marginBottom: 12,
      }}
    >
      <div
        style={{
          width: 28,
          height: 28,
          borderRadius: "50%",
          background: color,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          fontSize: 16,
          color: "#fff",
          fontWeight: 700,
          boxShadow: `0 0 12px ${color}66`,
        }}
      >
        ✓
      </div>
      <span
        style={{
          fontFamily: FONTS.sans,
          fontSize: 22,
          color: COLORS.text,
          fontWeight: 500,
        }}
      >
        {label}
      </span>
    </div>
  );
};
