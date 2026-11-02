import { ReasonText } from "./ReasonText";
import { Tooltip } from "./Tooltip";
import { withTerms } from "@/lib/glossary";

/**
 * UI-6: plain text with its glossary terms linked at their first use on the
 * page, each showing its definition on hover, focus or tap. Pass the page's
 * `seen` set to every call so a term links only once.
 */
export function Termed({ text, seen }: { text: string; seen: Set<string> }) {
  return (
    <>
      {withTerms(text, seen).map((p, i) =>
        typeof p === "string" ? (
          <ReasonText key={i} text={p} />
        ) : (
          <Tooltip key={i} href={`/glossary/#${p.slug}`} trigger={p.text}>
            {p.definition}
          </Tooltip>
        ),
      )}
    </>
  );
}
