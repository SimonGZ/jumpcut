#!/usr/bin/env node

import { readFile } from "node:fs/promises";
import path from "node:path";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

function parseArgs(argv) {
  const options = {
    pkgDir: null,
    target: "web",
    fixturesDir: null,
    warmup: 2,
    samples: 9,
    targetSampleMs: 40,
    metricPrefix: "wasm_node",
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--pkg-dir") {
      options.pkgDir = argv[++index];
    } else if (arg === "--target") {
      options.target = argv[++index];
    } else if (arg === "--fixtures-dir") {
      options.fixturesDir = argv[++index];
    } else if (arg === "--warmup") {
      options.warmup = Number(argv[++index]);
    } else if (arg === "--samples") {
      options.samples = Number(argv[++index]);
    } else if (arg === "--target-sample-ms") {
      options.targetSampleMs = Number(argv[++index]);
    } else if (arg === "--metric-prefix") {
      options.metricPrefix = argv[++index];
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!options.pkgDir || !options.fixturesDir) {
    throw new Error("--pkg-dir and --fixtures-dir are required");
  }

  if (options.target !== "web" && options.target !== "nodejs") {
    throw new Error(`Unsupported --target: ${options.target}`);
  }

  return options;
}

async function loadBindings(options) {
  if (options.target === "nodejs") {
    const require = createRequire(import.meta.url);
    return require(path.join(options.pkgDir, "jumpcut_wasm.js"));
  }

  const wrapperUrl = pathToFileURL(path.join(options.pkgDir, "jumpcut_wasm.js")).href;
  const wasmPath = path.join(options.pkgDir, "jumpcut_wasm_bg.wasm");
  const module = await import(wrapperUrl);
  const wasmBytes = await readFile(wasmPath);
  if (typeof module.initSync === "function") {
    module.initSync(wasmBytes);
  } else if (typeof module.default === "function") {
    await module.default(wasmBytes);
  } else {
    throw new Error("Web target package did not expose initSync/default initializer");
  }
  return module;
}

function sanitizeMetricPart(value) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "_").replace(/^_+|_+$/g, "");
}

function median(values) {
  const sorted = [...values].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 0) {
    return Math.round((sorted[middle - 1] + sorted[middle]) / 2);
  }
  return sorted[middle];
}

function benchmark(operation, options) {
  const timings = [];

  for (let iteration = 0; iteration < options.warmup; iteration += 1) {
    operation();
  }

  const estimateStart = process.hrtime.bigint();
  operation();
  const estimateNs = Number(process.hrtime.bigint() - estimateStart);
  const targetSampleNs = options.targetSampleMs * 1_000_000;
  const innerLoops = Math.max(1, Math.min(25, Math.ceil(targetSampleNs / Math.max(estimateNs, 1))));

  for (let sample = 0; sample < options.samples; sample += 1) {
    const start = process.hrtime.bigint();
    for (let iteration = 0; iteration < innerLoops; iteration += 1) {
      operation();
    }
    const elapsedNs = Number(process.hrtime.bigint() - start);
    timings.push(Math.round(elapsedNs / innerLoops));
  }

  return {
    medianNs: median(timings),
    innerLoops,
  };
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const bindings = await loadBindings(options);

  const fixtures = [
    { name: "108", path: path.join(options.fixturesDir, "108.fountain") },
    { name: "big_fish", path: path.join(options.fixturesDir, "Big-Fish.fountain") },
  ];

  const operations = [
    {
      name: "parse_to_json_string",
      fn: bindings.parse_to_json_string,
      args: (text) => [text],
    },
    {
      name: "parse_to_html_string",
      fn: bindings.parse_to_html_string,
      args: (text) => [text, true],
    },
    {
      name: "parse_to_fdx_string",
      fn: bindings.parse_to_fdx_string,
      args: (text) => [text],
    },
    {
      name: "parse_to_pdf_bytes",
      fn: bindings.parse_to_pdf_bytes,
      args: (text) => [text],
    },
  ].filter((operation) => typeof operation.fn === "function");

  if (operations.length === 0) {
    throw new Error("No wasm exports were available to benchmark");
  }

  for (const fixture of fixtures) {
    const text = await readFile(fixture.path, "utf8");
    for (const operation of operations) {
      let outputLength = -1;
      const result = benchmark(() => {
        const output = operation.fn(...operation.args(text));
        outputLength = output.length;
      }, options);

      const metricName = [
        options.metricPrefix,
        sanitizeMetricPart(operation.name),
        sanitizeMetricPart(fixture.name),
        "median_ns",
      ].join("_");

      console.log(`METRIC ${metricName}=${result.medianNs}`);
      console.log(
        `METRIC ${options.metricPrefix}_${sanitizeMetricPart(operation.name)}_${sanitizeMetricPart(fixture.name)}_output_len=${outputLength}`,
      );
    }
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
