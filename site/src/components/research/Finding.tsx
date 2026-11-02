import { GroupedReason } from "@/components/ReasonText";
import { readableFinding } from "@/lib/presentation";
import { Termed } from "@/components/Term";

/** A readable observation plus an intact source reason; no rating-based claims. */
export function Finding({ text, summary, seen }: { text: string; summary?: string; seen?: Set<string> }) {
  return <div className="finding-copy">
    <p>{seen ? <Termed text={summary ?? readableFinding(text)} seen={seen} /> : summary ?? readableFinding(text)}</p>
    <details><summary>Complete recorded reason</summary><GroupedReason text={text} seen={seen} /></details>
  </div>;
}
