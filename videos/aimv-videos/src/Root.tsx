// Root.tsx — Register all AIMV video compositions
import React from "react";
import { Composition } from "remotion";
import { VIDEO } from "./theme";
import { Intro } from "./compositions/Intro";
import { StoreAndEncrypt } from "./compositions/StoreAndEncrypt";
import { VersionTimeline } from "./compositions/VersionTimeline";
import { SecurityAudit } from "./compositions/SecurityAudit";
import { FormatSupport } from "./compositions/FormatSupport";
import { FullWorkflow } from "./compositions/FullWorkflow";
import { CLIInit } from "./compositions/CLIInit";
import { CLIStore } from "./compositions/CLIStore";
import { CLIVersions } from "./compositions/CLIVersions";
import { CLIConvert } from "./compositions/CLIConvert";
import { CLICompliance } from "./compositions/CLICompliance";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      {/* ── Showcase compositions ──────────────────────── */}
      <Composition
        id="Intro"
        component={Intro}
        durationInFrames={150}
        fps={VIDEO.FPS}
        width={VIDEO.WIDTH}
        height={VIDEO.HEIGHT}
      />
      <Composition
        id="StoreAndEncrypt"
        component={StoreAndEncrypt}
        durationInFrames={270}
        fps={VIDEO.FPS}
        width={VIDEO.WIDTH}
        height={VIDEO.HEIGHT}
      />
      <Composition
        id="VersionTimeline"
        component={VersionTimeline}
        durationInFrames={270}
        fps={VIDEO.FPS}
        width={VIDEO.WIDTH}
        height={VIDEO.HEIGHT}
      />
      <Composition
        id="SecurityAudit"
        component={SecurityAudit}
        durationInFrames={270}
        fps={VIDEO.FPS}
        width={VIDEO.WIDTH}
        height={VIDEO.HEIGHT}
      />
      <Composition
        id="FormatSupport"
        component={FormatSupport}
        durationInFrames={300}
        fps={VIDEO.FPS}
        width={VIDEO.WIDTH}
        height={VIDEO.HEIGHT}
      />
      <Composition
        id="FullWorkflow"
        component={FullWorkflow}
        durationInFrames={390}
        fps={VIDEO.FPS}
        width={VIDEO.WIDTH}
        height={VIDEO.HEIGHT}
      />

      {/* ── CLI example animations ─────────────────────── */}
      <Composition
        id="CLIInit"
        component={CLIInit}
        durationInFrames={330}
        fps={VIDEO.FPS}
        width={VIDEO.WIDTH}
        height={VIDEO.HEIGHT}
      />
      <Composition
        id="CLIStore"
        component={CLIStore}
        durationInFrames={420}
        fps={VIDEO.FPS}
        width={VIDEO.WIDTH}
        height={VIDEO.HEIGHT}
      />
      <Composition
        id="CLIVersions"
        component={CLIVersions}
        durationInFrames={480}
        fps={VIDEO.FPS}
        width={VIDEO.WIDTH}
        height={VIDEO.HEIGHT}
      />
      <Composition
        id="CLIConvert"
        component={CLIConvert}
        durationInFrames={400}
        fps={VIDEO.FPS}
        width={VIDEO.WIDTH}
        height={VIDEO.HEIGHT}
      />
      <Composition
        id="CLICompliance"
        component={CLICompliance}
        durationInFrames={340}
        fps={VIDEO.FPS}
        width={VIDEO.WIDTH}
        height={VIDEO.HEIGHT}
      />
    </>
  );
};
