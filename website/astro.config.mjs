import { defineConfig } from "astro/config";

// Static site for quickdesk.click, deployed by Cloudflare Pages.
export default defineConfig({
  site: "https://quickdesk.click",
  // docs.html, served by Pages at /docs (no trailing-slash redirect).
  build: { format: "file" },
  trailingSlash: "never",
  // The changelog pages read ../CHANGELOG*.md from the repo root.
  vite: { server: { fs: { allow: [".."] } } },
});
