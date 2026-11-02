// Brand assets: every insignia file is derived from the supplied
// assets/brand/celebtir-logo.png by resizing only. The insignia is never
// redrawn, recolored or cleaned up; replace the source PNG and rerun
// `node scripts/brand-assets.mjs` to regenerate them all.
import { copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import sharp from "sharp";

const site = fileURLToPath(new URL("../", import.meta.url));
const source = path.join(site, "assets", "brand", "celebtir-logo.png");
const brand = path.join(site, "public", "brand");
const app = path.join(site, "src", "app");
mkdirSync(brand, { recursive: true });

const png = (size) =>
  sharp(source).resize(size, size, { fit: "contain", kernel: "lanczos3", background: { r: 0, g: 0, b: 0, alpha: 0 } }).png({ compressionLevel: 9 }).toBuffer();

// Standalone variant: the source itself, transparent background.
copyFileSync(source, path.join(brand, "celebtir-insignia.png"));
for (const size of [64, 128, 256, 512]) writeFileSync(path.join(brand, `celebtir-insignia-${size}.png`), await png(size));
writeFileSync(path.join(app, "icon.png"), await png(512));
writeFileSync(path.join(app, "apple-icon.png"), await png(180));

// favicon.ico: PNG-compressed entries at 16, 32 and 48 px.
const sizes = [16, 32, 48];
const images = await Promise.all(sizes.map(png));
const header = Buffer.alloc(6 + 16 * sizes.length);
header.writeUInt16LE(0, 0);
header.writeUInt16LE(1, 2);
header.writeUInt16LE(sizes.length, 4);
let offset = header.length;
sizes.forEach((s, i) => {
  const e = 6 + 16 * i;
  header.writeUInt8(s, e);
  header.writeUInt8(s, e + 1);
  header.writeUInt16LE(1, e + 4);
  header.writeUInt16LE(32, e + 6);
  header.writeUInt32LE(images[i].length, e + 8);
  header.writeUInt32LE(offset, e + 12);
  offset += images[i].length;
});
writeFileSync(path.join(app, "favicon.ico"), Buffer.concat([header, ...images]));
for (const s of sizes) writeFileSync(path.join(brand, `favicon-${s}.png`), images[sizes.indexOf(s)]);
console.log("brand-assets: insignia, icon.png, apple-icon.png, favicon.ico (16/32/48)");
