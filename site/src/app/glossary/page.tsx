import type { Metadata } from "next";
import ref from "@/components/reference/Reference.module.css";
import styles from "@/components/page/Page.module.css";
import { GLOSSARY } from "@/lib/glossary";

export const metadata: Metadata = {
  title: "Glossary",
  description: "The terms Celebtir uses: upgrade authority, time lock, config authority, permanent delegate and more.",
};

// UI-6: every term linked from the site lands on its entry here.
export default function GlossaryPage() {
  const terms = [...GLOSSARY].sort((a, b) => a.term.localeCompare(b.term, "en", { sensitivity: "base" }));
  return (
    <>
      <h1>Glossary</h1>
      <p className={styles.lede}>Terms used across the site. Each one is linked, with its definition, the first time it appears on a page.</p>
      <nav className={ref.alphabet} aria-label="Glossary A to Z">{Array.from({ length: 26 }, (_, i) => String.fromCharCode(65 + i)).map(letter => {
        const first = terms.find(t => t.term[0]?.toUpperCase() === letter);
        return first ? <a key={letter} href={`#${first.slug}`}>{letter}</a> : <span key={letter}>{letter}</span>;
      })}</nav>
      <dl className={ref.glossary}>
        {terms.map((t) => (
          <div key={t.slug}>
            <dt id={t.slug}>{t.term[0]!.toUpperCase() + t.term.slice(1)}</dt>
            <dd>{t.definition}</dd>
          </div>
        ))}
      </dl>
    </>
  );
}
