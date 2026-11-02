import localFont from "next/font/local";

// Self-hosted fonts (ADR 023): bundled into the static export from the
// fontsource packages, so no page fetches a font from a third party (LCH-6).
// Latin subsets only.

/** Inter: application text, navigation and tables. */
export const inter = localFont({
  src: "../../node_modules/@fontsource-variable/inter/files/inter-latin-wght-normal.woff2",
  weight: "100 900",
  variable: "--font-inter",
  display: "swap",
  adjustFontFallback: "Arial",
});

/** Cormorant: official wordmark treatment and selected major data figures. */
export const cormorant = localFont({
  src: "../../node_modules/@fontsource/cormorant/files/cormorant-latin-400-normal.woff2",
  weight: "400",
  variable: "--font-cormorant",
  display: "swap",
  adjustFontFallback: "Times New Roman",
});

/** JetBrains Mono: addresses, slots, signatures and IDs only. */
export const jetbrainsMono = localFont({
  src: "../../node_modules/@fontsource-variable/jetbrains-mono/files/jetbrains-mono-latin-wght-normal.woff2",
  weight: "100 800",
  variable: "--font-jetbrains",
  display: "swap",
  preload: false,
  adjustFontFallback: false,
});
