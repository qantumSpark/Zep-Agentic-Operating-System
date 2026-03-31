import { CollapsibleSection } from "../common/CollapsibleSection";
import { WorkflowSection } from "./WorkflowSection";
import { PipelineSection } from "./PipelineSection";
import { MemorySection } from "./MemorySection";
import { ActionsFeed } from "./ActionsFeed";
import { AgentsSection } from "./AgentsSection";
import { ScreenshotGallery } from "./ScreenshotGallery";
import { SessionHistory } from "./SessionHistory";
import { SessionMetrics } from "./SessionMetrics";
import { DiffViewer } from "./DiffViewer";
import { IterationTracker } from "./IterationTracker";

/**
 * Dashboard container: vertically stacked collapsible sections
 */
export function DashboardPanel() {
  return (
    <div className="flex flex-col h-full bg-zinc-900 overflow-y-auto">
      <CollapsibleSection title="Workflow" defaultOpen={true}>
        <WorkflowSection />
      </CollapsibleSection>

      <CollapsibleSection title="Pipeline" defaultOpen={true}>
        <PipelineSection />
      </CollapsibleSection>

      <CollapsibleSection title="Memory" defaultOpen={true}>
        <MemorySection />
      </CollapsibleSection>

      <CollapsibleSection title="Actions (live)" defaultOpen={true}>
        <ActionsFeed />
      </CollapsibleSection>

      <CollapsibleSection title="Agents" defaultOpen={false}>
        <AgentsSection />
      </CollapsibleSection>

      <CollapsibleSection title="Iteration Loop" defaultOpen={false}>
        <IterationTracker />
      </CollapsibleSection>

      <CollapsibleSection title="Diffs" defaultOpen={false}>
        <DiffViewer />
      </CollapsibleSection>

      <CollapsibleSection title="Screenshots" defaultOpen={false}>
        <ScreenshotGallery />
      </CollapsibleSection>

      <CollapsibleSection title="Sessions" defaultOpen={false}>
        <SessionHistory />
      </CollapsibleSection>

      <CollapsibleSection title="Metriques" defaultOpen={false}>
        <SessionMetrics />
      </CollapsibleSection>
    </div>
  );
}
