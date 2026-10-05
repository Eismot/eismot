import assert from 'node:assert/strict';
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { escapeHtml, fiftyFifty, letters, money, questionFile, renderAnswers, renderJokers, renderTemplate, safetyNet, themedImage, themePath, themes, validateOptions } from './lib/trivia.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const argumentsList = process.argv.slice(2);
assert(argumentsList.every(argument => argument === '--check' || argument === '--pack=history'),
  'Supported arguments: --check, --pack=history');
const historyPack = argumentsList.includes('--pack=history');
const sharedDirectory = path.join(root, 'millionaire');
const gameDirectory = historyPack ? path.join(sharedDirectory, 'history') : sharedDirectory;
const arcadePath = historyPack ? '../../README.md' : '../README.md';
const questions = JSON.parse(await readFile(path.join(gameDirectory, 'questions.json'), 'utf8'));
const questionTemplate = (await readFile(path.join(sharedDirectory, 'templates/question.md.tmpl'), 'utf8'))
  .replaceAll('../README.md', arcadePath);
const hudTemplate = await readFile(path.join(sharedDirectory, 'templates/hud.svg'), 'utf8');
const headerTemplate = await readFile(path.join(sharedDirectory, 'templates/header.svg'), 'utf8');
const checkOnly = process.argv.includes('--check');

const referenceLabel = 'Source';
const prizes = [100, 200, 300, 500, 1000, 2000, 4000, 8000, 16000, 32000, 64000, 125000, 250000, 500000, 1000000];
const outputs = new Map();
const assetPrefix = historyPack ? '../../assets/' : '../assets/';

const badges = new Map();
for (const [theme, palette] of Object.entries(themes)) {
  badges.set(themePath('assets/arcade.svg', theme), renderTemplate(headerTemplate, {
    GREEN: palette.green, AMBER: palette.amber, MUTED: palette.muted,
  }, ['GREEN', 'AMBER', 'MUTED']));
  for (const [name, label, color] of [
    ...letters.map(letter => [letter.toLowerCase(), `> ${letter}`, palette.green]),
    ['question', '> QUESTION', palette.amber],
    ['fifty', '50:50', palette.green],
    ['hint', '[?]', palette.amber],
    ['cash', '[$]', palette.green],
  ]) {
    const width = ['fifty', 'hint', 'cash'].includes(name) ? 160 : 320;
    const fontSize = name === 'question' ? 20 : 26;
    const border = '─'.repeat(width === 160 ? 16 : 36);
    badges.set(themePath(`assets/trivia-${name}.svg`, theme), `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="56" viewBox="0 0 ${width} 56" role="img" aria-label="${escapeHtml(label)}">
  <g font-family="Consolas, 'Liberation Mono', monospace" fill="${color}">
    <g font-size="14" fill-opacity=".6">
      <text x="8" y="14" textLength="${width - 16}" lengthAdjust="spacingAndGlyphs">┌${border}┐</text>
      <text x="8" y="34">│</text>
      <text x="${width - 16}" y="34">│</text>
      <text x="8" y="52" textLength="${width - 16}" lengthAdjust="spacingAndGlyphs">└${border}┘</text>
    </g>
    <text x="24" y="36" font-size="${fontSize}">${escapeHtml(label)}</text>
    <text x="${width - 34}" y="36" font-size="20" fill-opacity=".65">▌</text>
  </g>
</svg>\n`);
  }
}

assert.equal(questions.length, 15, 'The game must have exactly 15 rounds');
assert.deepEqual(questions.map(question => question.prize), prizes, 'Unexpected prize ladder');

for (const [index, question] of questions.entries()) {
  validateOptions(question, `Q${index + 1}`);
  for (const field of ['category', 'question', 'hint', 'explanation', 'reference']) {
    assert.equal(typeof question[field], 'string', `Q${index + 1}: missing ${field}`);
    assert(question[field].trim().length > 0, `Q${index + 1}: empty ${field}`);
  }
  assert.equal(new URL(question.reference).protocol, 'https:');
  assert([...Object.values(question.options), question.question, question.hint, question.explanation, question.category]
    .every(text => !/[\r\n<>\[\]|]/.test(text)), `Q${index + 1}: unsupported Markdown characters`);

  const current = index === 0 ? 0 : prizes[index - 1];
  const floor = safetyNet(index);
  const correctTarget = index === 14 ? 'win.md' : questionFile(index + 1);
  const wrongTarget = `game-over-${floor}.md`;
  const targets = letters.map(letter => letter === question.correct ? correctTarget : wrongTarget);
  const answers = renderAnswers(question, targets, assetPrefix);
  const remaining = fiftyFifty(question);
  const previous = index === 0 ? '' : `<details>\n<summary>Previous answer</summary>\n\n## Q${index}: ${questions[index - 1].correct} / ${questions[index - 1].options[questions[index - 1].correct]}\n\n${questions[index - 1].explanation}\n\n[${referenceLabel}](${questions[index - 1].reference})\n\n</details>`;
  const checkpoint = index === 5 || index === 10 ? `**Safety net secured: ${money(floor)}**` : '';
  const ladder = prizes.map((prize, prizeIndex) => {
    const state = prizeIndex === index ? 'Current' : prizeIndex < index ? 'Answered' : 'Next';
    const checkpointLabel = prizeIndex === 4 || prizeIndex === 9 ? ' / Safety net' : '';
    return `| ${prizeIndex + 1} | ${money(prize)} | ${state}${checkpointLabel} |`;
  }).reverse().join('\n');

  const hudPath = `hud/${questionFile(index).replace('.md', '.svg')}`;
  const values = {
    REFERENCE_LABEL: referenceLabel,
    HUD_PATH: hudPath,
    HUD: themedImage(hudPath, `Round ${String(index + 1).padStart(2, '0')} of 15. Playing for ${money(question.prize)}. Bank ${money(current)}. Safety net ${money(floor)}.`),
    ROUND: String(index + 1).padStart(2, '0'),
    PRIZE: money(question.prize),
    BANK: money(current),
    SAFETY_NET: money(floor),
    COMPLETED: index,
    CHECKPOINT: checkpoint,
    CATEGORY: question.category,
    QUESTION_BADGE: themedImage(`${assetPrefix}trivia-question.svg`, 'Question'),
    QUESTION: escapeHtml(question.question),
    ANSWERS: answers,
    JOKERS: renderJokers(question, current, index, assetPrefix, arcadePath, 'q01.md'),
    FIFTY_FIFTY: remaining.join(' and '),
    HINT: question.hint,
    PREVIOUS_DEBRIEF: previous,
    LADDER: ladder,
    ANSWER: `${question.correct} / ${question.options[question.correct]}`,
    EXPLANATION: question.explanation,
    REFERENCE: question.reference,
  };
  const required = ['HUD', 'QUESTION_BADGE', 'QUESTION', 'ANSWERS', 'JOKERS'];
  const page = renderTemplate(questionTemplate, values, required);
  outputs.set(questionFile(index), page);

  for (const [theme, palette] of Object.entries(themes)) {
    const progressLights = prizes.map((_, prizeIndex) => {
      const color = prizeIndex < index ? palette.green : prizeIndex === index ? palette.amber : palette.inactive;
      return `<path fill="${color}" d="M${32 + prizeIndex * 44} 88h40v4h-40z"/>`;
    }).join('\n  ');
    outputs.set(themePath(hudPath, theme), renderTemplate(hudTemplate, {
      GREEN: palette.green,
      MUTED: palette.muted,
      ROUND: values.ROUND,
      PRIZE: values.PRIZE,
      BANK: values.BANK,
      SAFETY_NET: values.SAFETY_NET,
      COMPLETED: values.COMPLETED,
      PROGRESS_LIGHTS: progressLights,
    }, ['ROUND', 'PRIZE', 'BANK', 'SAFETY_NET', 'PROGRESS_LIGHTS', 'GREEN', 'MUTED']));
  }

  const answerLinks = [...answers.matchAll(/href="([^"]+)"/g)].map(match => match[1]);
  assert.equal(answerLinks.length, 4);
  assert.equal(answerLinks.filter(target => target === correctTarget).length, 1);
  assert.equal(answerLinks[letters.indexOf(question.correct)], correctTarget);
  assert.equal(answerLinks.filter(target => target === wrongTarget).length, 3);
  assert(page.includes(answers), 'Question template must preserve clickable answer selectors');
  assert(page.includes(values.JOKERS), 'Question template must preserve working jokers');
  assert.equal((answers.match(/<tr>/g) ?? []).length, 2, 'Answers must have two rows');
  assert.equal((answers.match(/<td /g) ?? []).length, 4, 'Answers must have four cells');
  assert.equal((page.match(/<picture>/g) ?? []).length, 9, 'Every terminal image must support both themes');
  assert.equal((values.JOKERS.match(/<details>/g) ?? []).length, 3, 'Three jokers required');
  for (const letter of letters) {
    assert.equal(answers.split(`alt="${letter}"`).length - 1, 1, `Answer selector ${letter} must appear once`);
  }
  assert(current >= floor);
  assert.equal(remaining.length, 2);
  assert(remaining.includes(question.correct));
}

for (const floor of [0, 1000, 32000]) {
  outputs.set(`game-over-${floor}.md`, `# Game over

**Final prize: ${money(floor)}**

Go back for the answer and source.

[Restart](q01.md) / [Games](${arcadePath})
`);
}

const finalQuestion = questions.at(-1);
outputs.set('win.md', `# Complete

**15 / 15. Final prize: $1,000,000**

## Final answer: ${finalQuestion.correct} / ${finalQuestion.options[finalQuestion.correct]}

${finalQuestion.explanation}

[Source](${finalQuestion.reference})

[Restart](q01.md) / [Games](${arcadePath})
`);

const existingPages = (await readdir(gameDirectory)).filter(name => name.endsWith('.md'));
assert(existingPages.every(name => outputs.has(name)), 'Unexpected game page: remove stale generated pages explicitly');
for (const [filename, content] of badges) {
  if (checkOnly) {
    assert.equal(await readFile(path.join(root, filename), 'utf8'), content, `${filename} is stale`);
  } else {
    await mkdir(path.dirname(path.join(root, filename)), { recursive: true });
    await writeFile(path.join(root, filename), content);
  }
}
for (const [filename, content] of outputs) {
  if (checkOnly) {
    assert.equal(await readFile(path.join(gameDirectory, filename), 'utf8'), content,
      `${filename} is stale; run node scripts/build-millionaire.mjs${historyPack ? ' --pack=history' : ''}`);
  } else {
    await mkdir(path.dirname(path.join(gameDirectory, filename)), { recursive: true });
    await writeFile(path.join(gameDirectory, filename), content);
  }
}

const readmePath = path.join(root, 'README.md');
let readme = await readFile(readmePath, 'utf8');
if (historyPack) {
  const openingQuestion = questions[0];
  const openingAnswers = renderAnswers(openingQuestion, letters.map(letter =>
    `millionaire/history/${letter === openingQuestion.correct ? 'q02.md' : 'game-over-0.md'}`), 'assets/');
  const opening = `<!-- trivia:start -->
<a href="millionaire/history/q01.md">${themedImage('assets/arcade.svg', 'Computer and language history. Question 1 of 15. Playing for $100. Bank $0. Safety net $0.')}</a>

> ${themedImage('assets/trivia-question.svg', 'Question')}
>
> **<samp>${escapeHtml(openingQuestion.question)}</samp>**

${openingAnswers}

${renderJokers(openingQuestion, 0, 0, 'assets/', 'README.md', 'millionaire/history/q01.md')}

<details>
<summary><samp>[+] Answer & source (spoiler)</samp></summary>

**${openingQuestion.correct}.** ${openingQuestion.options[openingQuestion.correct]}

${openingQuestion.explanation}

[Source](${openingQuestion.reference})

</details>
<!-- trivia:end -->`;
  const openingPattern = /<!-- trivia:start -->[\s\S]*?<!-- trivia:end -->/g;
  assert.equal([...readme.matchAll(openingPattern)].length, 1, 'README must contain one trivia block');
  const updatedReadme = readme.replace(openingPattern, () => opening);
  if (checkOnly) {
    assert.equal(readme, updatedReadme, 'README trivia is stale; run node scripts/build-millionaire.mjs --pack=history');
  } else {
    await writeFile(readmePath, updatedReadme);
    readme = updatedReadme;
  }
  const openingTargets = [...openingAnswers.matchAll(/href="([^"]+)"/g)].map(match => match[1]);
  assert.equal(openingTargets.length, 4);
  assert.equal(openingTargets.filter(target => target === 'millionaire/history/q02.md').length, 1);
  assert.equal(openingTargets[letters.indexOf(openingQuestion.correct)], 'millionaire/history/q02.md');
  assert.equal(openingTargets.filter(target => target === 'millionaire/history/game-over-0.md').length, 3);
}

const pages = new Map([...outputs].map(([filename, content]) => [path.join(gameDirectory, filename), content]));
pages.set(readmePath, readme);
let localLinks = 0;
for (const [filename, content] of pages) {
  const targets = [
    ...[...content.matchAll(/\]\(\s*([^\s)]+)\s*\)/g)].map(match => match[1]),
    ...[...content.matchAll(/(?:href|src|srcset)="([^"]+)"/g)].map(match => match[1]),
  ];
  for (const target of targets) {
    if (/^https:\/\//.test(target)) continue;
    assert(!/^[a-z]+:|^\/|[#?]/i.test(target), `Unsupported link in ${filename}: ${target}`);
    const resolved = path.resolve(path.dirname(filename), target);
    assert(!path.relative(root, resolved).startsWith('..'), `Link escapes repository: ${target}`);
    await readFile(resolved);
    localLinks += 1;
  }
  assert.equal((content.match(/<details>/g) ?? []).length, (content.match(/<\/details>/g) ?? []).length,
    `Unbalanced details in ${filename}`);
}

const pageCount = [...outputs.keys()].filter(filename => filename.endsWith('.md')).length;
console.log(`${checkOnly ? 'Verified' : 'Generated and verified'} ${pageCount} game pages and ${outputs.size - pageCount} HUD images; master templates, 15 rounds, 2 safety nets, 1 correct route per round, ${localLinks} local links.`);