import { ScopedLink } from "@/components/scope/ScopedLink";
import { Termed } from "@/components/Term";
import type { Block, Inline } from "@/lib/markdown";
import styles from "./Reference.module.css";

function Text({ parts, seen }: { parts: Inline[]; seen?: Set<string> }) {
  return (
    <>
      {parts.map((p, i) =>
        p.kind === "code" ? (
          <code key={i}>{p.text}</code>
        ) : p.kind === "link" ? (
          <ScopedLink key={i} href={p.href}>
            {p.text}
          </ScopedLink>
        ) : (
          <span key={i}>{seen ? <Termed text={p.text} seen={seen} /> : p.text}</span>
        ),
      )}
    </>
  );
}

/**
 * Renders parsed Markdown blocks; headings start at `h${base}`. With `seen`,
 * glossary terms in paragraphs and lists link at their first use (UI-6).
 */
export function Markdown({ blocks, base = 2, seen }: { blocks: Block[]; base?: 2 | 3; seen?: Set<string> }) {
  return (
    <div className={styles.prose}>
      {blocks.map((b, i) => {
        if (b.kind === "heading") {
          const H = b.level === 2 && base === 2 ? "h3" : "h4";
          return (
            <H key={i}>
              <Text parts={b.text} />
            </H>
          );
        }
        if (b.kind === "list") {
          return (
            <ul key={i}>
              {b.items.map((it, j) => (
                <li key={j}>
                  <Text parts={it} seen={seen} />
                </li>
              ))}
            </ul>
          );
        }
        return (
          <p key={i}>
            <Text parts={b.text} seen={seen} />
          </p>
        );
      })}
    </div>
  );
}
