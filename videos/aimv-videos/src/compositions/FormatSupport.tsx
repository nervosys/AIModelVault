// Format Support composition — 23+ formats showcase with animated grid
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
import { GridBackground, SlideTitle, Badge, GlowBox } from "../components/shared";

// ─── Format Card ──────────────────────────────────────────────────────────────

interface FormatCardProps {
  name: string;
  ext: string;
  color: string;
  delay: number;
}

const FormatCard: React.FC<FormatCardProps> = ({ name, ext, color, delay }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const s = spring({
    fps,
    frame: frame - delay,
    config: { damping: 14, mass: 0.5 },
  });

  const hover = interpolate(
    Math.sin((frame - delay) * 0.06),
    [-1, 1],
    [-2, 2]
  );

  return (
    <div
      style={{
        width: 170,
        padding: "16px 14px",
        background: `${COLORS.bgLight}cc`,
        borderRadius: 12,
        border: `1px solid ${color}40`,
        transform: `scale(${s}) translateY(${hover}px)`,
        opacity: s,
        textAlign: "center",
        boxShadow: `0 0 16px ${color}15`,
      }}
    >
      <div
        style={{
          fontFamily: FONTS.mono,
          fontSize: 11,
          color: `${color}cc`,
          background: `${color}15`,
          padding: "3px 10px",
          borderRadius: 6,
          display: "inline-block",
          marginBottom: 8,
          textTransform: "uppercase",
          letterSpacing: 1,
        }}
      >
        .{ext}
      </div>
      <div
        style={{
          fontFamily: FONTS.sans,
          fontSize: 15,
          color: COLORS.text,
          fontWeight: 600,
        }}
      >
        {name}
      </div>
    </div>
  );
};

// ─── Conversion Arrow ─────────────────────────────────────────────────────────

const ConversionArrow: React.FC<{
  from: string;
  to: string;
  delay: number;
}> = ({ from, to, delay }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const s = spring({
    fps,
    frame: frame - delay,
    config: { damping: 12, mass: 0.6 },
  });

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 16,
        opacity: s,
        transform: `translateX(${(1 - s) * 40}px)`,
      }}
    >
      <code
        style={{
          fontFamily: FONTS.mono,
          fontSize: 16,
          color: COLORS.warning,
          background: `${COLORS.warning}15`,
          padding: "6px 14px",
          borderRadius: 8,
        }}
      >
        {from}
      </code>
      <span style={{ fontSize: 22, color: COLORS.primary }}>→</span>
      <code
        style={{
          fontFamily: FONTS.mono,
          fontSize: 16,
          color: COLORS.success,
          background: `${COLORS.success}15`,
          padding: "6px 14px",
          borderRadius: 8,
        }}
      >
        {to}
      </code>
    </div>
  );
};

// ─── FormatSupport Composition ────────────────────────────────────────────────

export const FormatSupport: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const formats: FormatCardProps[] = [
    { name: "SafeTensors", ext: "safetensors", color: "#3b82f6", delay: 30 },
    { name: "PyTorch", ext: "pt", color: "#ee4c2c", delay: 40 },
    { name: "ONNX", ext: "onnx", color: "#005CED", delay: 50 },
    { name: "TensorFlow", ext: "pb", color: "#FF6F00", delay: 60 },
    { name: "GGUF", ext: "gguf", color: "#10b981", delay: 70 },
    { name: "GGML", ext: "ggml", color: "#059669", delay: 80 },
    { name: "TF Lite", ext: "tflite", color: "#FF6F00", delay: 90 },
    { name: "Keras", ext: "keras", color: "#D00000", delay: 100 },
    { name: "HDF5", ext: "h5", color: "#6366f1", delay: 110 },
    { name: "NumPy", ext: "npy", color: "#4DABCF", delay: 120 },
    { name: "CoreML", ext: "mlmodel", color: "#a855f7", delay: 130 },
    { name: "MLX", ext: "mlx", color: "#f59e0b", delay: 140 },
    { name: "JAX", ext: "jax", color: "#8b5cf6", delay: 150 },
    { name: "Pickle", ext: "pkl", color: "#ec4899", delay: 160 },
    { name: "MsgPack", ext: "msgpack", color: "#14b8a6", delay: 170 },
    { name: "JSON", ext: "json", color: "#eab308", delay: 180 },
    { name: "CBOR", ext: "cbor", color: "#f97316", delay: 190 },
    { name: "Parquet", ext: "parquet", color: "#06b6d4", delay: 200 },
    { name: "Arrow", ext: "arrow", color: "#ef4444", delay: 210 },
    { name: "FlexBuf", ext: "flexbuf", color: "#84cc16", delay: 220 },
    { name: "Protobuf", ext: "protobuf", color: "#22d3ee", delay: 230 },
    { name: "Bincode", ext: "bincode", color: "#f472b6", delay: 240 },
    { name: "BSON", ext: "bson", color: "#34d399", delay: 250 },
  ];

  const conversions = [
    { from: "PyTorch .pt", to: "SafeTensors", delay: 140 },
    { from: "ONNX .onnx", to: "GGUF", delay: 160 },
    { from: "HDF5 .h5", to: "NumPy .npy", delay: 180 },
  ];

  return (
    <AbsoluteFill>
      <GridBackground />
      <AbsoluteFill
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "flex-start",
          padding: "50px 60px",
          gap: 20,
        }}
      >
        <SlideTitle
          title="23+ Formats"
          subtitle="Store, convert, and serve any ML model format"
        />

        {/* Format grid */}
        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            gap: 12,
            justifyContent: "center",
            maxWidth: 1600,
          }}
        >
          {formats.map((f) => (
            <FormatCard key={f.ext} {...f} />
          ))}
        </div>

        {/* Conversion examples */}
        <Sequence from={130} layout="none">
          <div
            style={{
              marginTop: 16,
              display: "flex",
              gap: 50,
              alignItems: "center",
            }}
          >
            <span
              style={{
                fontFamily: FONTS.sans,
                fontSize: 18,
                color: COLORS.textMuted,
                fontWeight: 600,
              }}
            >
              Convert:
            </span>
            {conversions.map((c, i) => (
              <ConversionArrow key={i} {...c} />
            ))}
          </div>
        </Sequence>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
