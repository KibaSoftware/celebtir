// Site-wide facts and notices.

export const BRAND_HEADLINE = "DeFi Risk Intelligence";
export const BRAND_SUPPORTING_LINE = "Independent. Evidence-based. Transparent.";

/** The project's public repository, set at deploy (`CELEBTIR_REPO_URL`). */
export const REPO_URL: string | null = process.env.CELEBTIR_REPO_URL || null;

/** The deployed site's address, for absolute links (badges, cards); set at deploy. */
export const SITE_URL: string | null = process.env.CELEBTIR_SITE_URL?.replace(/\/$/, "") || null;

/** LCH-2: on every page's footer and on the About page. */
export const NOTICE = "Not investment advice. Ratings describe on-chain configuration only and may contain errors.";

/** LCH-4: the corrections policy, published on the About page. */
export const CORRECTIONS = {
  acknowledgeHours: 72,
  resolveDays: 7,
};
