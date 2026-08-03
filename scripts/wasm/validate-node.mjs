#!/usr/bin/env node

import path from "node:path";
import { createRequire } from "node:module";

function packageDirectory(argv) {
  const index = argv.indexOf("--pkg-dir");
  if (index === -1 || !argv[index + 1]) {
    throw new Error("Usage: validate-node.mjs --pkg-dir <generated-node-package>");
  }
  return argv[index + 1];
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(`WASM validation failed: ${message}`);
  }
}

function main() {
  const pkgDir = packageDirectory(process.argv.slice(2));
  const require = createRequire(import.meta.url);
  const wasm = require(path.join(pkgDir, "jumpcut_wasm.js"));

  const fountain = `Title: WASM Validation\nAuthor: JumpCut\n\nINT. TEST LAB - DAY\n\nA validation scene.\n\nADA\nIt works.`;

  const fountainOutput = wasm.parse_to_fountain_string(fountain);
  assert(fountainOutput.includes("INT. TEST LAB - DAY"), "Fountain export lost scene heading");
  assert(fountainOutput.includes("ADA"), "Fountain export lost dialogue");

  const json = JSON.parse(wasm.parse_to_json_string(fountain));
  assert(Array.isArray(json.elements) && json.elements.length > 0, "JSON export has no elements");

  const fdx = wasm.parse_to_fdx_string(fountain);
  assert(fdx.includes("<FinalDraft"), "FDX export is not a Final Draft document");

  const html = wasm.parse_to_html_string(fountain, true);
  assert(html.includes("TEST LAB"), "HTML export lost screenplay content");

  const htmlWithOptions = wasm.parse_to_html_string_with_options(fountain, true, true, true);
  assert(htmlWithOptions.includes("TEST LAB"), "HTML options export lost screenplay content");

  const pdf = wasm.parse_to_pdf_bytes(fountain);
  assert(Buffer.from(pdf).subarray(0, 5).toString("ascii") === "%PDF-", "PDF export lacks PDF header");

  const fdxFountain = wasm.parse_fdx_to_fountain_string(fdx);
  assert(fdxFountain.includes("INT. TEST LAB - DAY"), "FDX-to-Fountain export lost scene heading");

  const fdxHtml = wasm.parse_fdx_to_html_string(fdx, true);
  assert(fdxHtml.includes("TEST LAB"), "FDX-to-HTML export lost screenplay content");

  const fdxHtmlWithOptions = wasm.parse_fdx_to_html_string_with_options(fdx, true, true, true);
  assert(fdxHtmlWithOptions.includes("TEST LAB"), "FDX HTML options export lost screenplay content");

  const fdxPdf = wasm.parse_fdx_to_pdf_bytes(fdx);
  assert(Buffer.from(fdxPdf).subarray(0, 5).toString("ascii") === "%PDF-", "FDX-to-PDF export lacks PDF header");

  const embeddedFontsHtml = wasm.parse_to_html_string_with_embedded_courier_prime(
    fountain,
    true,
    false,
    false,
    "AA==",
    "AA==",
    "AA==",
    "AA==",
  );
  assert(embeddedFontsHtml.includes("data:font/ttf;base64,AA=="), "embedded-font HTML export omitted font data");

  console.log("WASM Node validation passed");
}

main();
