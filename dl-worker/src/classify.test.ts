// npm test (node --test, TypeScript run natively by Node 22).
import { test } from "node:test";
import assert from "node:assert/strict";
import { classify, counts, source } from "./classify.ts";

const updater = "tauri-plugin-updater/2.13.2";
const chrome = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0 Safari/537.36";
const ps = "Mozilla/5.0 (Windows NT; Windows NT 10.0; en-US) WindowsPowerShell/5.1.26100.1";

test("sources", () => {
  assert.equal(source(updater, ""), "updater");
  assert.equal(source("curl/8.5.0", ""), "terminal");
  assert.equal(source(ps, ""), "powershell");
  assert.equal(source(chrome, "https://quickdesk.click/vi/download"), "website");
  assert.equal(source(chrome, "https://github.com/TankGum/quickdesk/releases"), "github");
  assert.equal(source(chrome, ""), "direct");
  assert.equal(source("Googlebot/2.1", ""), "bot");
  assert.equal(source(chrome, "not a url"), "direct");
});

test("packages are installs, or updates when the updater asks", () => {
  const deb = "/releases/0.6.0/QuickDesk_0.6.0_amd64.deb";
  assert.deepEqual(classify(deb, chrome, "https://quickdesk.click/download"), {
    kind: "install",
    system: "linux-deb",
    version: "0.6.0",
    source: "website",
  });
  assert.deepEqual(classify("/releases/0.6.0/QuickDesk_0.6.0_amd64.AppImage", updater, ""), {
    kind: "update",
    system: "linux-appimage",
    version: "0.6.0",
    source: "updater",
  });
  assert.equal(classify("/releases/0.6.0/QuickDesk-0.6.0-1.x86_64.rpm", "curl/8", "").system, "linux-rpm");
  assert.equal(classify("/releases/0.6.0/QuickDesk_0.6.0_x64-setup.exe", ps, "").system, "windows");
  assert.equal(classify("/releases/0.6.0/QuickDesk_0.6.0_aarch64.dmg", chrome, "").system, "macos-dmg");
  assert.equal(classify("/releases/0.6.0/QuickDesk_0.6.0_aarch64.app.tar.gz", "curl/8", "").system, "macos-app");
});

test("other files", () => {
  assert.equal(classify("/update.json", updater, "").kind, "check");
  assert.equal(classify("/latest.json", "curl/8", "").kind, "latest");
  assert.equal(classify("/releases/0.6.0/SHA256SUMS", "curl/8", "").kind, "sums");
  assert.deepEqual(classify("/releases/0.6.0/QuickDesk_0.6.0_amd64.deb.sig", "curl/8", ""), {
    kind: "signature",
    system: "linux-deb",
    version: "0.6.0",
    source: "terminal",
  });
  assert.equal(classify("/favicon.ico", chrome, "").kind, "other");
  assert.equal(classify("/releases/0.6.0/notes.txt", chrome, "").kind, "other");
});

test("one download per file", () => {
  assert.equal(counts("GET", 200, null), true);
  assert.equal(counts("HEAD", 200, null), false);
  assert.equal(counts("GET", 304, null), false);
  assert.equal(counts("GET", 404, null), false);
  assert.equal(counts("GET", 206, "bytes=0-1023"), true);
  assert.equal(counts("GET", 206, "bytes=1024-"), false);
});
