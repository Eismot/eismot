import assert from "node:assert/strict";
import test from "node:test";
import { validateSourceUrl, validateSvg, validateWorkflow } from "./check-ci.mjs";

const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 20 20" role="img" aria-label="Sample"><path fill="#116329" d="M0 0h20v20H0z"/></svg>`;
const workflow = `name: Example
on: [pull_request]
permissions:
  contents: read
jobs:
  check:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@${"a".repeat(40)}
        with:
          persist-credentials: false
      - run: npm test
`;

test("SVG policy accepts static artwork and local paint references", () => {
  validateSvg(svg);
  validateSvg(svg.replace("<path", '<defs><pattern id="grid" width="2" height="2" patternUnits="userSpaceOnUse"/></defs><path').replace('fill="#116329"', 'fill="url(#grid)"'));
});

test("SVG policy rejects active content, remote resources, invalid XML, and missing accessible names", () => {
  for (const content of [
    svg.replace("</svg>", "<script>alert(1)</script></svg>"),
    svg.replace("</svg>", '<foreignObject><p>HTML</p></foreignObject></svg>'),
    svg.replace("</svg>", '<image href="https://example.com/tracker.png"/></svg>'),
    svg.replace("<path", '<path onload="alert(1)"'),
    svg.replace("<path", '<path style="fill:red"'),
    svg.replace('fill="#116329"', 'fill="url(https://example.com/paint.svg)"'),
    svg.replace('fill="#116329"', 'fill="url(#missing)"'),
    svg.replace('aria-label="Sample"', ""),
    svg.replace("</svg>", ""),
    '<!DOCTYPE svg SYSTEM "https://example.com/entity.dtd">' + svg,
    '<?xml-stylesheet href="https://example.com/style.css"?>' + svg,
  ]) assert.throws(() => validateSvg(content));
});

test("source URL policy rejects credentials, private literals, and non-HTTPS schemes", () => {
  validateSourceUrl("https://example.com/paper#section");
  for (const url of ["http://example.com", "javascript:alert(1)", "https://user:password@example.com", "https://127.0.0.1", "https://[::1]", "https://localhost", "https://service.internal"]) {
    assert.throws(() => validateSourceUrl(url));
  }
});

test("workflow policy enforces immutable actions, no stored credentials, and unprivileged execution", () => {
  validateWorkflow(workflow);
  for (const content of [
    workflow.replace("contents: read", "contents: write"),
    workflow.replace("pull_request", "pull_request_target"),
    workflow.replace("ubuntu-latest", "self-hosted"),
    workflow.replace("timeout-minutes: 10", "timeout-minutes: 60"),
    workflow.replace("a".repeat(40), "v4"),
    workflow.replace("persist-credentials: false", "persist-credentials: true"),
    workflow.replace("npm test", "echo '${{ github.event.pull_request.title }}'"),
    workflow + "permissions: write-all\n",
  ]) assert.throws(() => validateWorkflow(content));
});
