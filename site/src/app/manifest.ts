import type { MetadataRoute } from "next";
import { BRAND_HEADLINE, BRAND_SUPPORTING_LINE } from "@/lib/site";

// The web app manifest: the Celebtir name and the insignia icons (derived
// from assets/brand/celebtir-logo.png by scripts/brand-assets.mjs).
export const dynamic = "force-static";

export default function manifest(): MetadataRoute.Manifest {
  return {
    name: "Celebtir",
    short_name: "Celebtir",
    description: `${BRAND_HEADLINE}. ${BRAND_SUPPORTING_LINE} An evidence-linked scorecard of protocol upgradeability, admin powers and value exposure on Solana, with HyperEVM contract facts.`,
    start_url: "/",
    display: "browser",
    background_color: "#080E1A",
    theme_color: "#101A2B",
    icons: [
      { src: "/brand/celebtir-insignia-256.png", sizes: "256x256", type: "image/png" },
      { src: "/brand/celebtir-insignia-512.png", sizes: "512x512", type: "image/png" },
    ],
  };
}
