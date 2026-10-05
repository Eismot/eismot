import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import {
  escapeHtml,
  money,
  planRuns,
  questionFile,
  renderAnswers,
  renderJokers,
  renderTemplate,
  safetyNet,
  themedImage,
  tierForRound,
  validateBank,
  validateOptions,
  validateRuns,
} from "./lib/trivia.mjs";

const sampleBank = Array.from({ length: 25 }, (_, index) => ({
  id: `sample-${index}`,
  difficulty: Math.floor(index / 5) + 1,
  topic: `topic-${index % 3}`,
  category: "Sample",
  question: "Which option?",
  options: { A: "One", B: "Two", C: "Three", D: "Four" },
  correct: "B",
  hint: "A hint",
  explanation: "An explanation",
  reference: "https://example.com/source",
}));

test("published manifests use valid tiers and expose every approved pool question", async () => {
  const bank = JSON.parse(
    await readFile(
      new URL("../millionaire/questions.json", import.meta.url),
      "utf8",
    ),
  );
  const manifest = JSON.parse(
    await readFile(
      new URL("../millionaire/runs.json", import.meta.url),
      "utf8",
    ),
  );
  validateRuns(bank, manifest.runs);
  assert(bank.length >= 47);
  assert(manifest.runs.length >= 5);
  assert(
    !bank.some((question) =>
      /muratori|which year|when did/i.test(question.question),
    ),
  );
});

test("seeded runs cover the pool without repeating questions within a run", () => {
  const runs = planRuns(sampleBank, "test-seed");
  validateRuns(sampleBank, runs);
  assert.equal(runs.length, 2);
  assert.deepEqual(planRuns([...sampleBank].reverse(), "test-seed"), runs);
  assert.notDeepEqual(planRuns(sampleBank, "other-seed"), runs);
  assert.deepEqual(
    Array.from({ length: 15 }, (_, index) => tierForRound(index)),
    [1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5],
  );
});

test("bank validation rejects duplicate IDs, bad tiers, sparse pools, and question-level prizes", () => {
  assert.throws(
    () => validateBank([...sampleBank, sampleBank[0]]),
    /Duplicate question ID/,
  );
  assert.throws(
    () =>
      validateBank(
        sampleBank.map((question, index) =>
          index === 0 ? { ...question, difficulty: 6 } : question,
        ),
      ),
    /difficulty/,
  );
  assert.throws(
    () =>
      validateBank(sampleBank.filter((question) => question.difficulty !== 5)),
    /Tier 5/,
  );
  assert.throws(
    () =>
      validateBank(
        sampleBank.map((question, index) =>
          index === 0 ? { ...question, prize: 100 } : question,
        ),
      ),
    /prizes belong to rounds/,
  );
});

test("run validation rejects repetition, unknown IDs, wrong tiers, and incomplete coverage", () => {
  const runs = planRuns(sampleBank, "test-seed");
  const replaceFirst = (questions) => [{ ...runs[0], questions }, runs[1]];
  assert.throws(
    () =>
      validateRuns(
        sampleBank,
        replaceFirst(
          runs[0].questions.map((id, index) =>
            index === 1 ? runs[0].questions[0] : id,
          ),
        ),
      ),
    /repeated question/,
  );
  assert.throws(
    () =>
      validateRuns(
        sampleBank,
        replaceFirst(["missing", ...runs[0].questions.slice(1)]),
      ),
    /unknown question/,
  );
  const reordered = [...runs[0].questions];
  [reordered[0], reordered[3]] = [reordered[3], reordered[0]];
  assert.throws(
    () => validateRuns(sampleBank, replaceFirst(reordered)),
    /wrong tier/,
  );
  assert.throws(
    () => validateRuns(sampleBank, [runs[0]]),
    /Every bank question/,
  );
});

test("theme images select dark and light sources with an accessible light fallback", () => {
  const image = themedImage("../assets/trivia-a.svg", "A & B", {
    width: "100%",
    title: '"Hint"',
  });
  assert(
    image.includes(
      'media="(prefers-color-scheme: dark)" srcset="../assets/trivia-a-dark.svg"',
    ),
  );
  assert(
    image.includes(
      'media="(prefers-color-scheme: light)" srcset="../assets/trivia-a.svg"',
    ),
  );
  assert(
    image.includes(
      '<img src="../assets/trivia-a.svg" alt="A &amp; B" width="100%" title="&quot;Hint&quot;">',
    ),
  );
  assert.throws(() => themedImage("image.png", "Image"), /must be SVG/);
});

test("answer IDs remain valid regardless of JSON entry order", () => {
  const question = {
    options: { D: "Four", B: "Two", A: "One", C: "Three" },
    correct: "B",
  };
  validateOptions(question);
  assert.equal(question.options[question.correct], "Two");
});

test("option validation rejects legacy indexes, missing IDs, and duplicate text", () => {
  const options = { A: "One", B: "Two", C: "Three", D: "Four" };
  assert.throws(
    () => validateOptions({ options: Object.values(options), correct: 1 }),
    /object keyed/,
  );
  assert.throws(
    () => validateOptions({ options, correct: 1 }),
    /correct must be/,
  );
  assert.throws(
    () => validateOptions({ options, correct: "E" }),
    /correct must be/,
  );
  assert.throws(
    () => validateOptions({ options: { ...options, E: "Five" }, correct: "B" }),
    /exactly A, B, C, D/,
  );
  assert.throws(
    () => validateOptions({ options: { A: "One" }, correct: "A" }),
    /exactly A, B, C, D/,
  );
  assert.throws(
    () => validateOptions({ options: { ...options, D: " " }, correct: "B" }),
    /nonempty strings/,
  );
  assert.throws(
    () =>
      validateOptions({ options: { ...options, D: " One " }, correct: "B" }),
    /duplicate options/,
  );
});

test("safety nets apply only after completing their checkpoint", () => {
  assert.deepEqual(
    [0, 4, 5, 9, 10, 14, 15].map(safetyNet),
    [0, 0, 1000, 1000, 32000, 32000, 32000],
  );
});

test("prizes and page names have stable formatting", () => {
  assert.equal(money(1000000), "$1,000,000");
  assert.equal(questionFile(0), "q01.md");
  assert.equal(questionFile(14), "q15.md");
});

test("HTML content and attributes are escaped", () => {
  assert.equal(escapeHtml("&<>\"'"), "&amp;&lt;&gt;&quot;&#39;");
});

test("templates reject missing, unknown, and malformed tokens", () => {
  assert.throws(
    () => renderTemplate("{{UNKNOWN}}", {}, []),
    /Unknown template token/,
  );
  assert.throws(
    () => renderTemplate("No question", {}, ["QUESTION"]),
    /Template must include/,
  );
  assert.throws(
    () => renderTemplate("{{wrong}}", {}, []),
    /Unresolved or malformed/,
  );
});

test("template replacement preserves dollar signs and normalizes whitespace", () => {
  assert.equal(
    renderTemplate(
      "{{HINT}}\r\n\r\n\r\nEnd  ",
      { HINT: "$100 and $& stay literal" },
      ["HINT"],
    ),
    "$100 and $& stay literal\n\nEnd\n",
  );
});

test("answer grids preserve every target in a two-by-two layout", () => {
  const question = {
    options: { D: "Five", C: "Four", A: "One & two", B: "Three" },
  };
  const targets = [
    "game-over-0.md",
    "q02.md",
    "game-over-0.md",
    "game-over-0.md",
  ];
  const answers = renderAnswers(question, targets, "../assets/");
  const reordered = {
    options: { A: "One & two", B: "Three", C: "Four", D: "Five" },
  };
  assert.equal(answers, renderAnswers(reordered, targets, "../assets/"));
  assert.deepEqual(
    [...answers.matchAll(/href="([^"]+)"/g)].map((match) => match[1]),
    targets,
  );
  assert.equal((answers.match(/<tr>/g) ?? []).length, 2);
  assert.equal((answers.match(/<td /g) ?? []).length, 4);
  assert.equal((answers.match(/<picture>/g) ?? []).length, 4);
  assert(answers.includes("<samp>One &amp; two</samp>"));
});

test("jokers retain the correct choice, escaped hint, bank, and navigation", () => {
  for (const correct of ["A", "B", "C", "D"]) {
    const jokers = renderJokers(
      { correct, hint: "A & B" },
      1000,
      5,
      "../assets/",
      "../README.md",
      "q01.md",
    );
    const choices = jokers.match(/<samp>([A-D]) and ([A-D])<\/samp>/);
    assert(choices);
    assert(choices.slice(1).includes(correct));
    assert.notEqual(choices[1], choices[2]);
    assert.equal((jokers.match(/<details>/g) ?? []).length, 3);
    assert.equal((jokers.match(/<picture>/g) ?? []).length, 3);
    assert(jokers.includes("<samp>A &amp; B</samp>"));
    assert(jokers.includes("$1,000 / 5 of 15 answered."));
    assert(jokers.includes('href="../README.md"'));
    assert(jokers.includes('href="q01.md"'));
  }
});
