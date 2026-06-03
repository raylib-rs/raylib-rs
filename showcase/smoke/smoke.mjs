// Pages-site smoke gate. Loads a sample of example pages in headless
// Chromium and fails if any page aborts, throws, or fails to boot.
//
// Catches the deploy-time-only breakage classes that desktop CI can't see:
// missing resource bundles (#311's file_packager wiring), stripped FS_*
// runtime exports (-sFORCE_FILESYSTEM, the FS_createPath abort), 404'd
// artifacts, and shader-compile aborts on WebGL.
//
// Usage:
//   node showcase/smoke/smoke.mjs                  # serves showcase/_site locally
//   SMOKE_BASE_URL=https://raylib-rs.github.io/raylib-rs/ node showcase/smoke/smoke.mjs
//
// Requires playwright (`npm i --no-save playwright && npx playwright install chromium`).
// Exit code 0 = all sampled pages booted clean; 1 = at least one failure.

import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { extname, join, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

// One page per breakage class: a no-resource control (shapes), a
// loader-injected-but-no-file-loads page (the FS_createPath regression),
// resource-loading shaders (the original #311 report), plus a texture, a
// font (newly glsl-forked), and a light models page.
const SAMPLE_PAGES = [
  "examples/core/core_basic_window.html",
  "examples/core/core_2d_camera_platformer.html",
  "examples/shaders/shaders_eratosthenes_sieve.html",
  "examples/shaders/shaders_ascii_rendering.html",
  "examples/textures/textures_image_loading.html",
  "examples/text/text_font_sdf.html",
  "examples/models/models_geometric_shapes.html",
  "examples/shapes/shapes_basic_shapes.html",
];

// raylib's TraceLog reaches the browser console via emscripten's stdout.
// Any of these lines means the runtime initialised and raylib booted.
const BOOT_PATTERNS = [/INFO: rCore/i, /INFO: GL:/i, /INFO: RLGL:/i, /INFO: Initializing raylib/i];
const ERROR_PATTERNS = [/Aborted\(/, /RuntimeError/, /Uncaught /];

const WAIT_MS = Number(process.env.SMOKE_WAIT_MS || 20000);
const GRACE_MS = 2000; // after boot, linger to catch immediate post-boot aborts

const MIME = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".wasm": "application/wasm",
  ".data": "application/octet-stream",
  ".png": "image/png",
  ".json": "application/json",
};

function serveSite(root) {
  const server = createServer(async (req, res) => {
    const urlPath = decodeURIComponent(new URL(req.url, "http://x").pathname);
    let filePath = join(root, urlPath === "/" ? "index.html" : urlPath);
    try {
      const body = await readFile(filePath);
      res.writeHead(200, { "Content-Type": MIME[extname(filePath)] || "application/octet-stream" });
      res.end(body);
    } catch {
      res.writeHead(404, { "Content-Type": "text/plain" });
      res.end("not found");
    }
  });
  return new Promise((resolveServer) => {
    server.listen(0, "127.0.0.1", () => {
      const { port } = server.address();
      resolveServer({ server, baseUrl: `http://127.0.0.1:${port}/` });
    });
  });
}

async function checkPage(browser, baseUrl, pagePath) {
  const page = await browser.newPage();
  const errors = [];
  let booted = false;

  page.on("console", (msg) => {
    const text = msg.text();
    if (ERROR_PATTERNS.some((re) => re.test(text))) errors.push(`console: ${text.slice(0, 300)}`);
    if (BOOT_PATTERNS.some((re) => re.test(text))) booted = true;
  });
  page.on("pageerror", (err) => errors.push(`pageerror: ${String(err).slice(0, 300)}`));
  page.on("response", (resp) => {
    // Only same-origin artifacts; external links (GitHub etc.) aren't loaded
    // by the page itself, but guard anyway.
    if (resp.url().startsWith(baseUrl) && resp.status() >= 400) {
      errors.push(`HTTP ${resp.status()}: ${resp.url()}`);
    }
  });

  const url = baseUrl + pagePath;
  try {
    const resp = await page.goto(url, { waitUntil: "domcontentloaded", timeout: WAIT_MS });
    if (!resp || resp.status() >= 400) errors.push(`page HTTP ${resp ? resp.status() : "??"}`);
  } catch (e) {
    errors.push(`goto: ${String(e).slice(0, 200)}`);
  }

  // Poll until an error shows up, or boot + grace passes, or the window ends.
  const deadline = Date.now() + WAIT_MS;
  let bootedAt = null;
  while (Date.now() < deadline) {
    if (errors.length > 0) break;
    if (booted && bootedAt === null) bootedAt = Date.now();
    if (bootedAt !== null && Date.now() - bootedAt > GRACE_MS) break;
    await new Promise((r) => setTimeout(r, 250));
  }
  await page.close();

  if (errors.length > 0) return { pagePath, ok: false, detail: errors[0] };
  if (!booted) return { pagePath, ok: false, detail: `no raylib boot log within ${WAIT_MS}ms` };
  return { pagePath, ok: true, detail: "booted clean" };
}

async function main() {
  let baseUrl = process.env.SMOKE_BASE_URL;
  let server = null;
  if (!baseUrl) {
    const scriptDir = dirname(fileURLToPath(import.meta.url));
    const siteRoot = resolve(scriptDir, "..", "_site");
    if (!existsSync(join(siteRoot, "index.html"))) {
      console.error(`smoke: ${siteRoot}/index.html not found — run xtask-build-pages first, or set SMOKE_BASE_URL`);
      process.exit(1);
    }
    ({ server, baseUrl } = await serveSite(siteRoot));
    console.log(`smoke: serving ${siteRoot} at ${baseUrl}`);
  }
  if (!baseUrl.endsWith("/")) baseUrl += "/";

  const browser = await chromium.launch();
  const results = [];
  for (const pagePath of SAMPLE_PAGES) {
    const r = await checkPage(browser, baseUrl, pagePath);
    console.log(`${r.ok ? "PASS" : "FAIL"} ${r.pagePath} — ${r.detail}`);
    results.push(r);
  }
  await browser.close();
  if (server) server.close();

  const failed = results.filter((r) => !r.ok);
  console.log(`\nsmoke: ${results.length - failed.length}/${results.length} pages booted clean`);
  process.exit(failed.length > 0 ? 1 : 0);
}

main().catch((e) => {
  console.error("smoke: harness error:", e);
  process.exit(1);
});
