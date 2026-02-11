// AI Model Vault video theme — colors, fonts, and constants

export const COLORS = {
  // Primary palette
  bg: "#0a0e1a",
  bgLight: "#111827",
  bgCard: "#1a1f35",
  bgCardHover: "#232942",

  // Accent
  primary: "#3b82f6", // blue
  primaryLight: "#60a5fa",
  primaryDark: "#2563eb",

  // Semantic
  success: "#10b981",
  successLight: "#34d399",
  warning: "#f59e0b",
  error: "#ef4444",
  info: "#06b6d4",

  // Text
  text: "#f8fafc",
  textMuted: "#94a3b8",
  textDim: "#64748b",

  // Misc
  border: "#1e293b",
  glow: "rgba(59, 130, 246, 0.3)",
  glowGreen: "rgba(16, 185, 129, 0.3)",
  terminalBg: "#0d1117",
  terminalGreen: "#4ade80",
  terminalBlue: "#60a5fa",
  terminalYellow: "#fbbf24",
  terminalPrompt: "#a78bfa",
} as const;

export const VIDEO = {
  WIDTH: 1920,
  HEIGHT: 1080,
  FPS: 30,
} as const;

// Standard font stack
export const FONTS = {
  mono: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'Consolas', monospace",
  sans: "'Inter', 'SF Pro Display', -apple-system, 'Segoe UI', sans-serif",
  display:
    "'Inter', 'SF Pro Display', -apple-system, 'Segoe UI', sans-serif",
} as const;
