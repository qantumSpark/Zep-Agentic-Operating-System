import { CollapsibleSection } from "../common/CollapsibleSection";
import { MissionGroup } from "./MissionGroup";
import { DecisionsGroup } from "./DecisionsGroup";
import { EvidenceGroup } from "./EvidenceGroup";
import { ProgressGroup } from "./ProgressGroup";
import { AdvancedGroup } from "./AdvancedGroup";

/**
 * Dashboard Decision-First: 5 groups answering product piloting questions.
 *
 * Mission    — "What are we building?"
 * Decisions  — "What needs my input?"
 * Evidence   — "What proves it's ready (or not)?"
 * Progress   — "Where are we?"
 * Advanced   — Technical deep-dive (collapsed by default)
 */
export function DashboardPanel() {
  return (
    <div className="flex flex-col h-full bg-zinc-900 overflow-y-auto">
      <CollapsibleSection title="Mission" defaultOpen={true}>
        <MissionGroup />
      </CollapsibleSection>

      <CollapsibleSection title="Decisions" defaultOpen={true}>
        <DecisionsGroup />
      </CollapsibleSection>

      <CollapsibleSection title="Evidence" defaultOpen={true}>
        <EvidenceGroup />
      </CollapsibleSection>

      <CollapsibleSection title="Progress" defaultOpen={true}>
        <ProgressGroup />
      </CollapsibleSection>

      <CollapsibleSection title="Advanced" defaultOpen={false}>
        <AdvancedGroup />
      </CollapsibleSection>
    </div>
  );
}
