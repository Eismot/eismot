import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import { isIP } from "node:net";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { DOMParser } from "@xmldom/xmldom";
import { parseDocument } from "yaml";
import { validateRuns } from "./lib/trivia.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const svgNamespace = "http://www.w3.org/2000/svg";
const svgElements = new Set(["svg", "g", "path", "text", "title", "desc", "defs", "pattern"]);
const svgAttributes = new Set([
  "xmlns", "width", "height", "viewBox", "role", "aria-label", "aria-labelledby",
  "id", "x", "y", "d", "fill", "fill-opacity", "stroke", "stroke-opacity",
  "font-family", "font-size", "textLength", "lengthAdjust", "text-anchor", "patternUnits",
]);

export function validateSvg(source, label = "SVG") {
  const document = new DOMParser({
    onError: (_level, message) => { throw new Error(`${label}: ${message}`); },
  }).parseFromString(source, "application/xml");
  assert(!document.doctype, `${label}: DOCTYPE is forbidden`);
  const element = document.documentElement;
  assert(element?.tagName === "svg" && element.namespaceURI === svgNamespace, `${label}: SVG root required`);
  assert.equal(element.getAttribute("role"), "img", `${label}: accessible image role required`);
  assert(Number(element.getAttribute("width")) > 0 && Number(element.getAttribute("height")) > 0,
    `${label}: positive dimensions required`);
  const viewBox = element.getAttribute("viewBox")?.trim().split(/\s+/).map(Number);
  assert(viewBox?.length === 4 && viewBox.every(Number.isFinite) && viewBox[2] > 0 && viewBox[3] > 0,
    `${label}: valid viewBox required`);
  const ids = new Map();
  const references = [];
  const nodes = [document];
  while (nodes.length) {
    const node = nodes.pop();
    assert(![7, 10].includes(node.nodeType), `${label}: processing instructions and declarations are forbidden`);
    if (node.nodeType === 1) {
      assert(node.namespaceURI === svgNamespace && svgElements.has(node.tagName), `${label}: unsafe element ${node.tagName}`);
      if (node.hasAttribute("id")) {
        const id = node.getAttribute("id");
        assert(!ids.has(id), `${label}: duplicate SVG ID ${id}`);
        ids.set(id, node);
      }
      for (const attribute of Array.from(node.attributes)) {
        assert(svgAttributes.has(attribute.name), `${label}: unsafe attribute ${attribute.name}`);
        if (["fill", "stroke"].includes(attribute.name)) {
          assert(/^(?:#[a-f\d]{3,8}|none|currentColor|url\(#[a-zA-Z][\w-]*\))$/i.test(attribute.value),
            `${label}: unsafe paint value ${attribute.value}`);
          const reference = /^url\(#([\w-]+)\)$/.exec(attribute.value);
          if (reference) references.push(reference[1]);
        }
      }
    }
    nodes.push(...Array.from(node.childNodes));
  }
  for (const id of references) assert(ids.has(id), `${label}: missing paint reference ${id}`);
  const accessibleIds = element.getAttribute("aria-labelledby")?.trim().split(/\s+/);
  assert(element.getAttribute("aria-label")?.trim() ||
    (accessibleIds?.length && accessibleIds.every(id => ids.get(id)?.textContent?.trim())),
  `${label}: accessible name required`);
}

export function validateSourceUrl(reference) {
  const url = new URL(reference);
  assert(url.protocol === "https:" && !url.username && !url.password, "Public HTTPS source without credentials required");
  assert(!isIP(url.hostname.replace(/^\[|\]$/g, "")) && url.hostname !== "localhost" &&
    !/\.(?:local|localhost|internal|invalid)$/.test(url.hostname), "Public source hostname required");
}

export function validateWorkflow(source, label = "Workflow") {
  const document = parseDocument(source, { uniqueKeys: true });
  assert.equal(document.errors.length, 0, `${label}: invalid YAML: ${document.errors.join("; ")}`);
  const workflow = document.toJS();
  assert.deepEqual(workflow.permissions, { contents: "read" }, `${label}: read-only contents permission required`);
  const events = typeof workflow.on === "string" ? [workflow.on] : Array.isArray(workflow.on) ? workflow.on : Object.keys(workflow.on ?? {});
  assert(events.length && events.every(event => ["push", "pull_request", "schedule", "workflow_dispatch"].includes(event)),
    `${label}: privileged or unsupported trigger`);
  for (const job of Object.values(workflow.jobs ?? {})) {
    assert.equal(job["runs-on"], "ubuntu-latest", `${label}: GitHub-hosted runner required`);
    assert(job["timeout-minutes"] > 0 && job["timeout-minutes"] <= 20, `${label}: bounded job timeout required`);
    if (job.permissions) assert.deepEqual(job.permissions, { contents: "read" }, `${label}: job permissions must remain read-only`);
    for (const step of job.steps ?? []) {
      if (step.uses) {
        assert(/^[\w-]+\/[\w-]+(?:\/[\w-]+)*@[a-f\d]{40}$/.test(step.uses), `${label}: action must use a full commit SHA`);
        if (step.uses.startsWith("actions/checkout@")) assert.equal(step.with?.["persist-credentials"], false,
          `${label}: checkout must not save credentials`);
      }
      assert(!/\$\{\{[^}]*github\.event\./.test(step.run ?? ""), `${label}: do not interpolate event data into shell scripts`);
    }
  }
}

async function repositoryFiles(directory = root) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    if ([".git", "node_modules", "lychee"].includes(entry.name)) continue;
    const filename = path.join(directory, entry.name);
    assert(!entry.isSymbolicLink(), `Repository symlink requires review: ${filename}`);
    if (entry.isDirectory()) files.push(...await repositoryFiles(filename));
    else if (entry.isFile()) files.push(filename);
  }
  return files.sort();
}

async function checkRepository() {
  const files = await repositoryFiles();
  const svgFiles = files.filter(filename => filename.endsWith(".svg") && !filename.includes(`${path.sep}templates${path.sep}`));
  for (const filename of svgFiles) validateSvg(await readFile(filename, "utf8"), path.relative(root, filename));
  const workflows = files.filter(filename => filename.includes(`${path.sep}.github${path.sep}workflows${path.sep}`) && /\.ya?ml$/.test(filename));
  assert(workflows.length > 0, "At least one CI workflow required");
  for (const filename of workflows) validateWorkflow(await readFile(filename, "utf8"), path.relative(root, filename));
  const bank = JSON.parse(await readFile(path.join(root, "millionaire/questions.json"), "utf8"));
  const manifest = JSON.parse(await readFile(path.join(root, "millionaire/runs.json"), "utf8"));
  assert(typeof manifest.seed === "string" && manifest.seed.trim(), "Manifest seed required");
  validateRuns(bank, manifest.runs);
  assert(manifest.runs.some(run => run.id === "01"), "README's opening Run 01 required");
  assert(manifest.runs.every(run => run.id !== "00"), "Run 00 is reserved");
  for (const question of bank) validateSourceUrl(question.reference);
  const { lint } = await import("markdownlint/promise");
  const config = JSON.parse(await readFile(path.join(root, ".markdownlint.json"), "utf8"));
  const markdownFiles = files.filter(filename => filename.endsWith(".md"));
  const result = await lint({ files: markdownFiles, config: { ...config, MD036: false } });
  const errors = Object.entries(result).flatMap(([filename, diagnostics]) =>
    diagnostics.map(error => `${path.relative(root, filename)}:${error.lineNumber} ${error.ruleNames[0]} ${error.ruleDescription} ${error.errorDetail ?? ""}`));
  assert.equal(errors.length, 0, errors.join("\n"));
  console.log(`Validated ${svgFiles.length} static SVGs, ${workflows.length} hardened workflows, ${bank.length} source URLs, and ${markdownFiles.length} Markdown files.`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) await checkRepository();
