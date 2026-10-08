# Contributing

The game runs entirely through GitHub Markdown links. Rust 1.81 or newer and
Cargo are needed only to generate and check the pages. Install Rust through
[rustup](https://rustup.rs/). Node.js and npm are no longer required.
Cargo.lock pins the generator's dependencies; use `--locked` for reproducible builds.

## Repository layout

- [README.md](README.md): playable Run 01 opener and alternate-run selection.
- [millionaire/questions.json](millionaire/questions.json): shared, sourced question pool.
- [millionaire/runs.json](millionaire/runs.json): seed and pinned question IDs for every run.
- [millionaire/templates](millionaire/templates): shared question and score-strip templates.
- [Cargo.toml](Cargo.toml): Rust package, minimum toolchain, and dependencies.
- [src/main.rs](src/main.rs): command-line interface and contextual error reporting.
- [src/trivia.rs](src/trivia.rs): typed models, bank validation, seeded selection, and game rules.
- [src/render.rs](src/render.rs): strict templates, themed images, answer grids, and jokers.
- [src/build.rs](src/build.rs): generation, exact-output verification, and local link checks.
- [src/safety.rs](src/safety.rs): parsed SVG, source URL, and workflow safety checks.
- Tests live beside the Rust modules and use the published outputs as compatibility fixtures.
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

`cargo run --locked -- plan` prints a proposed manifest from the seed in runs.json without
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
[src/render.rs](src/render.rs). GitHub selects the appropriate
variant through `<picture>` sources; other renderers can use the light fallback.
The unsuffixed SVG files are light variants; `-dark.svg` files are dark variants.

Edit the shared [header template](millionaire/templates/header.svg), score-strip
template, or badge generator rather than generated images. Regenerate all runs
after any palette or template change. Keep backgrounds transparent to support
GitHub's other background colors, including dimmed themes.

## Build and verify

Run from the repository root after editing the pool, manifest, or a shared template:

```sh
cargo test --locked
cargo run --locked -- build
cargo run --locked -- check
```

Build commands regenerate files. Check commands do not write files; they verify
exact output, full-bank coverage, tier order, answer routes, safety nets, layout
structure, and local link targets.
Include regenerated output with the source changes so the published game stays current.

For one run, use `cargo run --locked -- build --run=02` and replace `build` with
`check` to verify it. The legacy `--pack=history` option also selects Run 02.
Use the all-run commands after shared changes and before publishing.

`plan` and `check-ci` do not accept run selection. All commands accept
`--root=PATH` to target a repository instead of the current directory. The CLI
prints contextual failures to stderr and exits nonzero; checks never write game files.
Builds render and validate the complete selected output before writing any files.

Answers are an enum, option objects have exactly four named fields, difficulty
is a validated type, and a run contains a fixed 15-question array. Deserialization
rejects unknown fields, invalid answers, invalid difficulty, and incomplete rounds.
Bank and manifest validation checks IDs, distinct options, safe text, public HTTPS
sources, tier order, repeated questions, opening Run 01, and full-pool coverage.
Seeded planning preserves the original SHA-256 ranking and topic separation.

For a local visual check, open the README or a round in VS Code's Markdown preview.
GitHub strips custom CSS, so verify significant layout changes on GitHub as well.
Gameplay is honor-system: answers, hints, and navigation are public, not private state.

## Continuous integration

[Validate Trivia](.github/workflows/ci.yml) runs on pull requests, pushes to main,
and manual dispatch. It checks:

- Rust formatting, warning-free Clippy, and tests on Rust 1.81 and stable.
- Exact generated pages and artwork, full pool coverage, tiers, answer routes,
  safety nets, and local image/link targets.
- Offline repository-wide Markdown/HTML links and local anchors through Lychee.
- All Markdown files through rumdl, with the existing native-HTML allowlist. MD036 is disabled
  because standalone emphasized prize/checkpoint labels are intentional game UI.
- Well-formed generated SVGs with accessible names, dimensions, local paint IDs,
  and an allowlist of static elements and attributes. Scripts, event handlers,
  embedded HTML, stylesheets, external resources, and declarations are rejected.
- Parsed workflow YAML: immutable action pins, read-only tokens, hosted runners,
  bounded timeouts, no saved checkout credentials, and no privileged triggers.
- Public HTTPS source URLs without credentials, plus manifest seed and opening run.
- Locked Cargo dependencies, with RustSec auditing that fails on known
  vulnerabilities and advisory warnings. Review dependency build scripts as well
  as source changes; Cargo may execute dependency build scripts during compilation.

Run the extra deterministic checks locally:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked -- check-ci
rumdl check .
cargo audit --deny warnings
```

The Markdown linter and dependency auditor are separate Rust tools, not generator
dependencies. Install rumdl 0.2.78 from its
[release binaries](https://github.com/rvben/rumdl/releases/tag/v0.2.78), or use
`uvx --from rumdl==0.2.78 rumdl check .` to run its native binary in an isolated
environment. rumdl reads the existing [.markdownlint.json](.markdownlint.json).
Install the auditor using a current stable toolchain with
`cargo +stable install --locked cargo-audit --version 0.22.2`.
Installing these separate tools may require a newer Rust compiler than the
generator's minimum; prebuilt rumdl binaries need no Rust toolchain.

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
rumdl v0.2.78 is also downloaded over HTTPS and verified against its pinned
Linux archive SHA-256 before extraction or execution.
[Dependabot](.github/dependabot.yml) proposes weekly action/tool updates; it does
not auto-merge them. There are no runtime dependencies or player-side scripts.

## GitHub settings

These require repository-owner configuration; files cannot enable them:

- Require the Validate Trivia jobs in a main-branch ruleset before merging.
  Do not require the scheduled external-link workflow.
- Enable secret scanning, push protection, Dependabot alerts/security updates,
  and private vulnerability reporting where available.
- Enable GitHub Actions security scanning where available. The static-output
  checks are not a general Rust security audit; review Rust and dependency changes.
- Restrict Actions to approved actions with full SHA pins; keep the default
  workflow token read-only and disable automatic PR approval by workflows.
- Review workflow, lockfile, source-URL, and SVG-policy changes carefully. A
  contributor can modify checks in the same PR, so CI cannot replace review.
