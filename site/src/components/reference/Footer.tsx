import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import { NOTICE } from "@/lib/site";
import styles from "./Reference.module.css";

/** LCH-2: in the root layout, so it is on every route. */
export function Footer() {
  return (
    <footer className={styles.footer}>
      <p>
        {NOTICE} <Link href="/about/#corrections">How to report one</Link>.
      </p>
      <nav aria-label="Reference" className={styles.footerNav}>
        <Link href="/about/">About</Link>
        <Link href="/glossary/">Glossary</Link>
        <Link href="/api/">API</Link>
        <Link href="/controllers/">Shared controllers</Link>
        <Link href="/archived/">Archived protocols</Link>
        <a href="/feeds/all.atom">Atom feed</a>
        <a href="/llms.txt">llms.txt</a>
      </nav>
      <p className={styles.small}>Code under MIT or Apache-2.0. Data under CC BY 4.0. No cookies.</p>
    </footer>
  );
}
