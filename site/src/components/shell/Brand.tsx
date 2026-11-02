import { ScopedLink as Link } from "@/components/scope/ScopedLink";
import styles from "./Shell.module.css";

/**
 * Header lockup: the supplied Celebtir insignia (public/brand, derived from
 * assets/brand/celebtir-logo.png by scripts/brand-assets.mjs) and the wordmark. The
 * insignia is square, so width equals height at every size.
 */
export function Brand() {
  return (
    <Link href="/" className={styles.brand} aria-label="Celebtir home">
      {/* Static export: next/image would not resize it anyway (images.unoptimized). */}
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img
        src="/brand/celebtir-insignia-64.png"
        srcSet="/brand/celebtir-insignia-64.png 1x, /brand/celebtir-insignia-128.png 2x"
        width={36}
        height={36}
        alt=""
        className={styles.insignia}
      />
      <span className={styles.wordmark}>Celebtir</span>
    </Link>
  );
}
