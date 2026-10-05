# Contributing

The game runs entirely through GitHub Markdown links. Node.js 18 or newer is
needed only to generate and check the pages; those commands need no packages.
For the additional CI safety and Markdown checks, use Node.js 24 and install the
locked development tools with `npm ci --ignore-scripts`.

## Repository layout

- [README.md](README.md): playable Run 01 opener and alternate-run selection.
- [millionaire/questions.json](millionaire/questions.json): shared, sourced question pool.
- [millionaire/runs.json](millionaire/runs.json): seed and pinned question IDs for every run.
- [millionaire/templates](millionaire/templates): shared question and score-strip templates.
- [scripts/build-millionaire.mjs](scripts/build-millionaire.mjs): bank validation, generation, and link checks.
- [scripts/lib/trivia.mjs](scripts/lib/trivia.mjs): tiered selection, validation, game rules, and rendering.
- [scripts/trivia.test.mjs](scripts/trivia.test.mjs): helper tests using Node's built-in test runner.
- [assets](assets): header artwork and generated answer/joker badges.

The round pages, end screens, score strips, and trivia badges are generated files.
Edit their source templates or generator instead of editing the output directly.
The all-run build updates only the block between the trivia markers in the README.
Run 01 keeps the original millionaire paths. Run 02 keeps the former history paths
for existing links. Later runs live under millionaire/runs/NN. Directory names no
longer identify thematic packs: every run draws from the shared pool.

## Editing questions

Edit the shared JSON pool. Each object is one question, independent of a round
or prize. Array order does not control gameplay; runs pin questions by stable IDs.

```json
{
  "id": "python-name",
  "difficulty": 1,
  "topic": "python",
  "category": "Python / naming",
  "question": "What inspired Python's name?",
  "options": {
    "A": "A snake",
    "B": "Monty Python",
    "C": "A compiler acronym",
    "D": "A fictional character"
  },
  "correct": "B",
  "hint": "Think published comedy scripts.",
  "explanation": "The official Python FAQ attributes the name to Monty Python.",
  "reference": "https://docs.python.org/3/faq/general.html#why-is-it-called-python"
}
```

- `options` is an object with exactly the keys A, B, C, and D.
- `correct` names an option key, not its position. Display order is always A-D,
  regardless of the order of entries in the JSON object.
- Supply four distinct, nonempty options and an HTTPS source URL.
- IDs are unique, stable kebab-case strings. Do not reuse an ID for a different question.
- Difficulty is 1 (warm-up), 2 (easy), 3 (medium), 4 (hard), or 5 (expert).
- Each run uses three questions per tier, in increasing difficulty, without repetition.
- Each tier needs at least three questions. Judge difficulty using the options, not obscurity alone.
- Topic tags help the planner separate neighboring questions about the same subject.
- Questions have no prize; the round supplies the fixed 15-step prize ladder.
- Text fields must be single-line and cannot contain `<`, `>`, `[`, `]`, or `|`.
- Cite primary papers, archived documents, or official project histories when possible.
- Match the question's claim to the source; distinguish proposals, implementation,
  public releases, and later retrospectives. Avoid unsupported "first ever" claims.
- Avoid date recall and talk-dependent trivia. Attribute author opinions explicitly.

## Stable runs

Seeded selection happens at planning time, not when a player visits GitHub.
The initial five routes cover all 47 questions. Questions can recur across runs,
but never within one run. A complete manifest must expose every pool question.

`npm run plan` prints a proposed manifest from the seed in runs.json without
writing files. Ordering is independent of bank array order. The planner separates
adjacent topics where possible; it cannot guarantee equal topic counts.

The generator reads pinned IDs, not a fresh sample. After publication, do not
replace or reorder an existing run's IDs. Add newly planned routes with unused
two-digit run IDs to expose new questions; keep existing routes intact. Correct
factual errors in place and keep references current. Retiering a published
question requires deliberately updating affected routes because validation will
reject tier mismatches. Review planned manifests before publishing.

## Theme artwork

Terminal images have transparent backgrounds and light/dark palettes in
[scripts/lib/trivia.mjs](scripts/lib/trivia.mjs). GitHub selects the appropriate
variant through `<picture>` sources; other renderers can use the light fallback.
The unsuffixed SVG files are light variants; `-dark.svg` files are dark variants.

Edit the shared [header template](millionaire/templates/header.svg), score-strip
template, or badge generator rather than generated images. Regenerate all runs
after any palette or template change. Keep backgrounds transparent to support
GitHub's other background colors, including dimmed themes.

## Build and verify

Run from the repository root after editing the pool, manifest, or a shared template:

```sh
npm test
npm run build
npm run check
```

Build commands regenerate files. Check commands do not write files; they verify
exact output, full-bank coverage, tier order, answer routes, safety nets, layout
structure, and local link targets.
Include regenerated output with the source changes so the published game stays current.

For one run, use `node scripts/build-millionaire.mjs --run=02` and add `--check`
to verify it. The legacy build:history and check:history scripts select Run 02.
Use the all-run commands after shared changes and before publishing.

For a local visual check, open the README or a round in VS Code's Markdown preview.
GitHub strips custom CSS, so verify significant layout changes on GitHub as well.
Gameplay is honor-system: answers, hints, and navigation are public, not private state.

## Continuous integration

[Validate Trivia](.github/workflows/ci.yml) runs on pull requests, pushes to main,
and manual dispatch. It checks:

- Rules and rendering on Node 18, 22, and 24. Node 18 is EOL and tested only to
  preserve the documented generation minimum; use Node 24 for development.
- Exact generated pages and artwork, full pool coverage, tiers, answer routes,
  safety nets, and local image/link targets.
- Offline repository-wide Markdown/HTML links and local anchors through Lychee.
- All Markdown files, with the existing native-HTML allowlist. MD036 is disabled
  because standalone emphasized prize/checkpoint labels are intentional game UI.
- Well-formed generated SVGs with accessible names, dimensions, local paint IDs,
  and an allowlist of static elements and attributes. Scripts, event handlers,
  embedded HTML, stylesheets, external resources, and declarations are rejected.
- Parsed workflow YAML: immutable action pins, read-only tokens, hosted runners,
  bounded timeouts, no saved checkout credentials, and no privileged triggers.
- Public HTTPS source URLs without credentials, plus manifest seed and opening run.
- Locked development dependencies, with lifecycle scripts disabled and a failing
  audit for moderate-or-higher known advisories.

Run the extra deterministic checks locally with Node 24:

```sh
npm ci --ignore-scripts
npm run test:ci
npm run check:ci
npm audit --audit-level=moderate
```

[Audit External Links](.github/workflows/links.yml) runs weekly on Monday at
07:23 UTC or manually. It checks cited sources and documentation with TLS
verification, HTTPS enforcement, private-network exclusions, timeouts, retries,
and limited concurrency. Failures appear in the job summary and a 14-day report
artifact. It does not create issues or request write permissions, and is separate
from required PR checks because external sites can block bots or be unavailable.
The schedule becomes active only after the workflow reaches the default branch;
GitHub may delay scheduled runs or disable them after repository inactivity.

No broad domain exclusions, ignored 403/429 responses, accepted timeouts, or
disabled certificate checks are configured. One transport exception covers only
the Fourmilab sketch URL and its NoteA fragment: Lychee's HTTP/2 requests failed
locally, while an HTTPS/HTTP/1.1 request returned 200. A separate failing curl GET
checks that exact page with HTTP/1.1, retaining TLS verification and HTTPS-only
redirects. The page is not left unchecked. Revisit the exception when Lychee or
the source server changes.

Investigate failures: a bot block is not proof of a dead page. Add narrowly
scoped, documented exceptions only after manual verification. Reachability does
not establish factual correctness. Remote page fragments and the truth of an
answer still need editorial review.

Actions use full commit SHAs. Lychee v0.23.0 is downloaded over HTTPS and its
Linux archive SHA-256 is checked before extraction or execution. When updating
Lychee, update both workflows' version and digest from the upstream release.
[Dependabot](.github/dependabot.yml) proposes weekly action/tool updates; it does
not auto-merge them. There are no runtime dependencies or player-side scripts.

## GitHub settings

These require repository-owner configuration; files cannot enable them:

- Require the Validate Trivia jobs in a main-branch ruleset before merging.
  Do not require the scheduled external-link workflow.
- Enable secret scanning, push protection, Dependabot alerts/security updates,
  and private vulnerability reporting where available.
- Enable CodeQL default setup for JavaScript and GitHub Actions scanning where
  available. The static-output checks are not a general JavaScript security audit.
- Restrict Actions to approved actions with full SHA pins; keep the default
  workflow token read-only and disable automatic PR approval by workflows.
- Review workflow, lockfile, source-URL, and SVG-policy changes carefully. A
  contributor can modify checks in the same PR, so CI cannot replace review.
