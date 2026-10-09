// The home page embeds the real app UI built in demo mode (`npm run build:demo`
// at the repo root → dist-demo). Copy it to public/demo before building.
import { cpSync, existsSync, rmSync } from "node:fs";
import { fileURLToPath } from "node:url";

const from = fileURLToPath(new URL("../../dist-demo", import.meta.url));
const to = fileURLToPath(new URL("../public/demo", import.meta.url));
rmSync(to, { recursive: true, force: true });
if (existsSync(`${from}/index.html`)) {
  cpSync(from, to, { recursive: true });
  console.log("demo: copied dist-demo → public/demo");
} else if (process.env.CF_PAGES || process.env.CI) {
  console.error("demo: ../dist-demo is missing; run `npm run build:demo` in the repo root first");
  process.exit(1);
} else {
  console.warn("demo: ../dist-demo is missing; the home page demo will be empty (run `npm run build:demo` in the repo root)");
}
