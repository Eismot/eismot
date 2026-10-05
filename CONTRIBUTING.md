# Contributing

The game runs entirely through GitHub Markdown links. Node.js 18 or newer is
needed only to generate and check the pages. No package installation is required.

## Repository layout

- [README.md](README.md): playable first question and pack selection.
- [millionaire/questions.json](millionaire/questions.json): mixed engineering bank.
- [millionaire/history/questions.json](millionaire/history/questions.json): sourced history bank.
- [millionaire/templates](millionaire/templates): shared question and score-strip templates.
- [scripts/build-millionaire.mjs](scripts/build-millionaire.mjs): bank validation, generation, and link checks.
- [scripts/lib/trivia.mjs](scripts/lib/trivia.mjs): game rules and pure rendering helpers.
- [scripts/trivia.test.mjs](scripts/trivia.test.mjs): helper tests using Node's built-in test runner.
- [assets](assets): header artwork and generated answer/joker badges.

The round pages, end screens, score strips, and trivia badges are generated files.
Edit their source templates or generator instead of editing the output directly.
The history build also updates only the block between the trivia markers in the README.

## Editing questions

Edit the relevant JSON bank. Each object is one round; array order is play order.
Both packs currently require exactly 15 rounds, so replace or revise an existing
question rather than appending a sixteenth.

```json
{
  "prize": 100,
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
- Keep the prize ladder: 100, 200, 300, 500, 1000, 2000, 4000, 8000, 16000,
  32000, 64000, 125000, 250000, 500000, 1000000.
- Text fields must be single-line and cannot contain `<`, `>`, `[`, `]`, or `|`.
- Cite primary papers, archived documents, or official project histories when possible.
- Match the question's claim to the source; distinguish proposals, implementation,
  public releases, and later retrospectives. Avoid unsupported "first ever" claims.

## Theme artwork

Terminal images have transparent backgrounds and light/dark palettes in
[scripts/lib/trivia.mjs](scripts/lib/trivia.mjs). GitHub selects the appropriate
variant through `<picture>` sources; other renderers can use the light fallback.
The unsuffixed SVG files are light variants; `-dark.svg` files are dark variants.

Edit the shared [header template](millionaire/templates/header.svg), score-strip
template, or badge generator rather than generated images. Regenerate both packs
after any palette or template change. Keep backgrounds transparent to support
GitHub's other background colors, including dimmed themes.

## Build and verify

Run from the repository root after editing either bank or a shared template:

```sh
npm run build
npm run build:history
npm test
npm run check
npm run check:history
```

Build commands regenerate files. Check commands do not write files; they verify
exact output, answer routes, safety nets, layout structure, and local link targets.
Include regenerated output with the source changes so the published game stays current.

For a local visual check, open the README or a round in VS Code's Markdown preview.
GitHub strips custom CSS, so verify significant layout changes on GitHub as well.
Gameplay is honor-system: answers, hints, and navigation are public, not private state.
