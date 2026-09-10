// Generates the macOS-style app-icon squircle (superellipse, n = 5) used to clip
// the Octanest mark in the DOM and to mask raster app icons.
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";

const N = 5;
const SIZE = 1024;
const STEPS = 720;
const OUT = path.resolve(import.meta.dirname, "../apps/web/public/brand/squircle.svg");

const points = [];
for (let i = 0; i < STEPS; i += 1) {
  const t = (i / STEPS) * 2 * Math.PI;
  const c = Math.cos(t);
  const s = Math.sin(t);
  const x = Math.sign(c) * Math.abs(c) ** (2 / N);
  const y = Math.sign(s) * Math.abs(s) ** (2 / N);
  points.push(`${(((x + 1) / 2) * SIZE).toFixed(2)},${(((y + 1) / 2) * SIZE).toFixed(2)}`);
}

const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${SIZE} ${SIZE}" width="${SIZE}" height="${SIZE}"><path fill="#ffffff" d="M${points.join(" L")} Z"/></svg>\n`;

mkdirSync(path.dirname(OUT), { recursive: true });
writeFileSync(OUT, svg);
console.log(`wrote ${OUT} (${svg.length} bytes)`);
