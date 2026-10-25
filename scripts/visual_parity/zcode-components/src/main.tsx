import React from "react";
import { createRoot } from "react-dom/client";
import "../../packages/ui/src/styles.css";
import { ZCodeIntlProvider } from "../../packages/ui/src/i18n/IntlProvider.js";
import { WorkflowCardHeader } from "../../packages/ui/src/components/workflow-timeline/WorkflowCardChrome.js";
import { WorkflowTimeline } from "../../packages/ui/src/components/workflow-timeline/WorkflowTimeline.js";
import type { WorkflowTimelineModel } from "../../packages/ui/src/components/workflow-timeline/timeline-model.js";

import { TooltipProvider } from "../../packages/ui/src/components/ui/tooltip.js";
import fixture from "./fixture.json";
const status = (value: string) => ({ waiting: "pending", running: "running", completed: "done", failed: "failed" } as const)[value] ?? "pending";
const model: WorkflowTimelineModel = {
  stations: fixture.stages.map(stage => ({
    id: stage.id, naming: { id: stage.id, name: stage.label }, status: status(stage.status),
    visited: stage.progress !== null, rounds: stage.progress ? 1 : 0, onLoop: false, track: 0,
    ...(stage.progress ? { fraction: stage.progress } : {}),
    pills: stage.nodes.map(node => ({
      key: node.id, lane: { id: node.id, laneClass: node.kind === "script" ? "workspace" : "agent", name: node.label },
      laneClass: node.kind === "script" ? "workspace" : "agent", runtimeName: node.label,
      avatarIndex: node.avatar_index, status: status(node.status), stepIds: [],
    })),
  })),
  rails: fixture.stages.slice(1).map((_, i) => ({ from: i, to: i + 1, ink: "faint" })),
  arcs: [], bands: [], runningIndex: 0, live: true,
};

function App() {
  const dark = new URLSearchParams(location.search).get("theme") === "dark";
  document.documentElement.className = dark ? "theme-zai-dark" : "theme-zai-light";
  return <ZCodeIntlProvider initialLocale="zh-CN"><TooltipProvider><main style={{ minHeight: "100vh", padding: 16, background: "var(--color-background)", color: "var(--color-foreground)" }}><section style={{ maxWidth: 960, margin: "0 auto" }} className="wf-motion flex w-full min-w-0 flex-col gap-1 rounded-xl border border-border/70 bg-card/70 px-3.5 pb-2 pt-1.5 outline-none"><WorkflowCardHeader kind="工作流运行中" name="多 Agent 对话卡片 · 离线视觉样例 · running" detail="5 个阶段 · 2 个子代理工作中 · Fixture model" expanded={true} live={false} onOpenDetails={() => {}} /><WorkflowTimeline className="py-1" model={model} /></section></main></TooltipProvider></ZCodeIntlProvider>;
}
createRoot(document.getElementById("root")!).render(<App />);
