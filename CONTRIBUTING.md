# Contributing

The game runs entirely through GitHub Markdown links. Node.js 18 or newer is
needed only to generate and check the pages. No package installation is required.

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
