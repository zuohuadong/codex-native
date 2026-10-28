import React from "react";
import { createRoot } from "react-dom/client";
import "../../packages/ui/src/styles.css";
import { ZCodeIntlProvider } from "../../packages/ui/src/i18n/IntlProvider.js";
import { WorkflowCardHeader } from "../../packages/ui/src/components/workflow-timeline/WorkflowCardChrome.js";
import { WorkflowTimeline } from "../../packages/ui/src/components/workflow-timeline/WorkflowTimeline.js";
import type { WorkflowTimelineModel } from "../../packages/ui/src/components/workflow-timeline/timeline-model.js";

import { TooltipProvider } from "../../packages/ui/src/components/ui/tooltip.js";
import running from "./fixture.json";
import completed from "./completed.json";
const query = new URLSearchParams(location.search);
const fixture = query.get("state") === "completed" ? completed : running;
const requestedWidth = Number(query.get("width") ?? 960);
const contentWidth = Number.isFinite(requestedWidth) ? Math.max(280, Math.min(1152, requestedWidth)) : 960;
const status = (value: string) => ({ waiting: "pending", running: "running", completed: "done", failed: "failed" } as const)[value] ?? "pending";
const runningIndex = fixture.stages.findIndex(stage => stage.status === "running");
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
  arcs: [], bands: [],
  runningIndex: runningIndex < 0 ? undefined : runningIndex,
  live: fixture.status === "running",
};

function App() {
  const dark = query.get("theme") === "dark";
  document.documentElement.className = dark ? "theme-zai-dark" : "theme-zai-light";
  const activeAgents = fixture.stages.flatMap(stage => stage.nodes).filter(node => node.kind === "agent" && node.status === "running").length;
  return <ZCodeIntlProvider initialLocale="zh-CN"><TooltipProvider><main style={{ minHeight: "100vh", padding: 16, background: "var(--color-background)", color: "var(--color-foreground)" }}><section data-testid="reference-workflow" style={{ maxWidth: contentWidth, margin: "0 auto" }} className="wf-motion flex w-full min-w-0 flex-col gap-1 rounded-xl border border-border/70 bg-card/70 px-3.5 pb-2 pt-1.5 outline-none"><WorkflowCardHeader kind={fixture.status === "completed" ? "工作流已完成" : "工作流运行中"} name={fixture.title} detail={`${fixture.stages.length} 个阶段 · ${activeAgents} 个子代理工作中 · ${fixture.model}`} expanded={true} live={false} onOpenDetails={() => {}} /><WorkflowTimeline className="py-1" model={model} /></section></main></TooltipProvider></ZCodeIntlProvider>;
}
createRoot(document.getElementById("root")!).render(<App />);
