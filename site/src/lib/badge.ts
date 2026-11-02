// API-4: a small SVG badge per protocol showing its Stage, rebuilt with every
// daily snapshot, plus the snippets to embed it.

import { STAGE_COLORS, STAGE_LABELS } from "./labels";
import { chrome, contrast, ink, surfaces } from "./tokens";
import type { Stage } from "./types";

const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
// Verdana-ish average advance at 11 px, as badge renderers assume.
const width = (text: string) => Math.round(text.length * 6.6 + 14);

/** F5: an unassessed protocol's badge says so, never a Stage. */
const stageText = (stage: Stage | null) => (stage ? STAGE_LABELS[stage] : "Unassessed");

export const badgeLabel = (name: string, stage: Stage | null) => `${name} on Celebtir: ${stageText(stage)}`;

/** Dark or light text, whichever reads better on `bg`. */
function textOn(bg: string): string {
  return contrast(surfaces.page, bg) >= contrast(ink.primary, bg) ? surfaces.page : ink.primary;
}

export function badgeSvg(name: string, stage: Stage | null): string {
  const left = "Celebtir";
  const right = stageText(stage);
  const lw = width(left);
  const rw = width(right);
  const w = lw + rw;
  const label = esc(badgeLabel(name, stage));
  const bg = stage ? STAGE_COLORS[stage] : chrome.axis;
  return [
    `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="20" viewBox="0 0 ${w} 20" role="img" aria-label="${label}">`,
    `<title>${label}</title>`,
    `<rect width="${lw}" height="20" rx="3" fill="${chrome.axis}"/>`,
    `<rect x="${lw}" width="${rw}" height="20" rx="3" fill="${bg}"/>`,
    `<rect x="${lw}" width="4" height="20" fill="${bg}"/>`,
    `<g font-family="Verdana,DejaVu Sans,sans-serif" font-size="11" text-anchor="middle">`,
    `<text x="${lw / 2}" y="14" fill="${ink.primary}">${left}</text>`,
    `<text x="${lw + rw / 2}" y="14" fill="${textOn(bg)}" font-weight="bold">${esc(right)}</text>`,
    `</g>`,
    `</svg>`,
  ].join("");
}

/** HTML and Markdown that embed the badge, linking the protocol page. */
export function embedSnippets(siteUrl: string, id: string, name: string, stage: Stage | null) {
  const img = `${siteUrl}/badges/${id}.svg`;
  const page = `${siteUrl}/protocols/${id}/`;
  const alt = badgeLabel(name, stage);
  return {
    html: `<a href="${page}"><img src="${img}" alt="${esc(alt)}"></a>`,
    markdown: `[![${alt}](${img})](${page})`,
  };
}
