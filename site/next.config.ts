import type { NextConfig } from "next";

// A fully static site (UI-26): every route is pre-rendered to HTML at build
// time and served from GitHub Pages. No server, so no Cache Components.
const nextConfig: NextConfig = {
  output: "export",
  // `/protocols/x/` → `/protocols/x/index.html`, which GitHub Pages serves.
  trailingSlash: true,
  images: { unoptimized: true },
};

export default nextConfig;
