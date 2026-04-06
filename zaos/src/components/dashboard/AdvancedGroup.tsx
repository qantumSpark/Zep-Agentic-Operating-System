import { CollapsibleSection } from "../common/CollapsibleSection";
import { WorkflowSection } from "./WorkflowSection";
import { PipelineSection } from "./PipelineSection";
import { ProductSection } from "./ProductSection";
import { UnifiedAgentsSection } from "./UnifiedAgentsSection";
import { ActionsFeed } from "./ActionsFeed";
import { DiffViewer } from "./DiffViewer";
import { IterationTracker } from "./IterationTracker";
import { ScreenshotGallery } from "./ScreenshotGallery";
import { SessionHistory } from "./SessionHistory";
import { SessionMetrics } from "./SessionMetrics";
import { HooksManager } from "./HooksManager";
import { RulesManager } from "./RulesManager";

export function AdvancedGroup() {
  return (
    <div className="space-y-0">
      <CollapsibleSection title="Workflow (complet)" defaultOpen={false}>
        <WorkflowSection />
      </CollapsibleSection>

      <CollapsibleSection title="Pipeline" defaultOpen={false}>
        <PipelineSection />
      </CollapsibleSection>

      <CollapsibleSection title="Product (complet)" defaultOpen={false}>
        <ProductSection />
      </CollapsibleSection>

      <CollapsibleSection title="Agents" defaultOpen={false}>
        <UnifiedAgentsSection />
      </CollapsibleSection>

      <CollapsibleSection title="Actions (complet)" defaultOpen={false}>
        <ActionsFeed />
      </CollapsibleSection>

      <CollapsibleSection title="Diffs" defaultOpen={false}>
        <DiffViewer />
      </CollapsibleSection>

      <CollapsibleSection title="Iteration Loop" defaultOpen={false}>
        <IterationTracker />
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

      <CollapsibleSection title="Hooks" defaultOpen={false}>
        <HooksManager />
      </CollapsibleSection>

      <CollapsibleSection title="Rules" defaultOpen={false}>
        <RulesManager />
      </CollapsibleSection>
    </div>
  );
}
