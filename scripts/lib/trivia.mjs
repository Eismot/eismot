import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

export const letters = ['A', 'B', 'C', 'D'];
export const prizes = [100, 200, 300, 500, 1000, 2000, 4000, 8000, 16000, 32000, 64000, 125000, 250000, 500000, 1000000];
export const tierForRound = index => Math.floor(index / 3) + 1;
export const money = amount => `$${amount.toLocaleString('en-US')}`;
export const questionFile = index => `q${String(index + 1).padStart(2, '0')}.md`;
export const safetyNet = completed => completed >= 10 ? 32000 : completed >= 5 ? 1000 : 0;
export const escapeHtml = text => String(text).replace(/[&<>"']/g, character => ({
  '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;',
})[character]);

export const themes = {
  light: { green: '#116329', amber: '#7d4e00', muted: '#57606a', inactive: '#d0d7de' },
  dark: { green: '#9be9a8', amber: '#f3d889', muted: '#a2b6a8', inactive: '#304637' },
};

export const themePath = (filename, theme) => theme === 'dark' ? filename.replace(/\.svg$/, '-dark.svg') : filename;

export function themedImage(filename, alt, { width, title } = {}) {
  assert(filename.endsWith('.svg'), 'Theme images must be SVG files');
  const attributes = `${width ? ` width="${escapeHtml(width)}"` : ''}${title ? ` title="${escapeHtml(title)}"` : ''}`;
  return `<picture><source media="(prefers-color-scheme: dark)" srcset="${escapeHtml(themePath(filename, 'dark'))}"><source media="(prefers-color-scheme: light)" srcset="${escapeHtml(filename)}"><img src="${escapeHtml(filename)}" alt="${escapeHtml(alt)}"${attributes}></picture>`;
}

export function validateOptions(question, label = 'Question') {
  assert(question.options && typeof question.options === 'object' && !Array.isArray(question.options),
    `${label}: options must be an object keyed by A, B, C, D`);
  assert.deepEqual(Object.keys(question.options).sort(), letters, `${label}: exactly A, B, C, D required`);
  assert(letters.includes(question.correct), `${label}: correct must be A, B, C, or D`);
  const options = letters.map(letter => question.options[letter]);
  assert(options.every(option => typeof option === 'string' && option.trim().length > 0),
    `${label}: options must be nonempty strings`);
  assert.equal(new Set(options.map(option => option.trim())).size, 4, `${label}: duplicate options`);
}

export function validateBank(bank) {
  assert(Array.isArray(bank) && bank.length > 0, 'Question bank must be a nonempty array');
  const ids = new Set();
  for (const question of bank) {
    assert(typeof question.id === 'string' && /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(question.id), 'Question needs a stable kebab-case ID');
    assert(!ids.has(question.id), `Duplicate question ID: ${question.id}`);
    ids.add(question.id);
    assert(Number.isInteger(question.difficulty) && question.difficulty >= 1 && question.difficulty <= 5,
      `${question.id}: difficulty must be an integer from 1 to 5`);
    assert(!Object.hasOwn(question, 'prize'), `${question.id}: prizes belong to rounds, not questions`);
    validateOptions(question, question.id);
    for (const field of ['topic', 'category', 'question', 'hint', 'explanation', 'reference']) {
      assert(typeof question[field] === 'string' && question[field].trim().length > 0, `${question.id}: missing ${field}`);
    }
    assert.equal(new URL(question.reference).protocol, 'https:', `${question.id}: HTTPS reference required`);
    assert([...Object.values(question.options), question.topic, question.question, question.hint, question.explanation, question.category]
      .every(text => !/[\r\n<>\[\]|]/.test(text)), `${question.id}: unsupported Markdown characters`);
  }
  for (let difficulty = 1; difficulty <= 5; difficulty++) {
    assert(bank.filter(question => question.difficulty === difficulty).length >= 3, `Tier ${difficulty} needs at least three questions`);
  }
}

export function planRuns(bank, seed) {
  validateBank(bank);
  assert(typeof seed === 'string' && seed.trim().length > 0, 'A nonempty run seed is required');
  const decks = Array.from({ length: 5 }, (_, tierIndex) => {
    const ranked = bank.filter(question => question.difficulty === tierIndex + 1).map(question => ({
      question, rank: createHash('sha256').update(`${seed}:${question.id}`).digest('hex'),
    })).sort((left, right) => left.rank < right.rank ? -1 : left.rank > right.rank ? 1 : left.question.id.localeCompare(right.question.id));
    const deck = [];
    while (ranked.length > 0) {
      const differentTopic = ranked.findIndex(entry => entry.question.topic !== deck.at(-1)?.topic);
      deck.push(ranked.splice(Math.max(0, differentTopic), 1)[0].question);
    }
    return deck;
  });
  const count = Math.max(...decks.map(deck => Math.ceil(deck.length / 3)));
  return Array.from({ length: count }, (_, runIndex) => ({
    id: String(runIndex + 1).padStart(2, '0'),
    questions: decks.flatMap(deck => Array.from({ length: 3 }, (_, offset) => deck[(runIndex * 3 + offset) % deck.length].id)),
  }));
}

export function validateRuns(bank, runs) {
  validateBank(bank);
  assert(Array.isArray(runs) && runs.length > 0, 'At least one run required');
  const byId = new Map(bank.map(question => [question.id, question]));
  const used = new Set();
  const runIds = new Set();
  for (const run of runs) {
    assert(typeof run.id === 'string' && /^\d{2}$/.test(run.id), 'Run ID must contain two digits');
    assert(!runIds.has(run.id), `Duplicate run ID: ${run.id}`);
    runIds.add(run.id);
    assert(Array.isArray(run.questions) && run.questions.length === 15, `Run ${run.id}: exactly 15 questions required`);
    assert.equal(new Set(run.questions).size, 15, `Run ${run.id}: repeated question`);
    for (const [index, id] of run.questions.entries()) {
      assert(byId.has(id), `Run ${run.id}: unknown question ${id}`);
      assert.equal(byId.get(id).difficulty, tierForRound(index), `Run ${run.id}: wrong tier at round ${index + 1}`);
      used.add(id);
    }
  }
  assert.equal(used.size, bank.length, 'Every bank question must appear in at least one run');
}

export function renderAnswers(question, targets, prefix) {
  const cells = letters.map((letter, optionIndex) =>
    `<td width="50%"><a href="${escapeHtml(targets[optionIndex])}">${themedImage(`${prefix}trivia-${letter.toLowerCase()}.svg`, letter, { width: '100%' })}<br><samp>${escapeHtml(question.options[letter])}</samp></a></td>`);
  return `<table width="100%">\n<tr>\n${cells.slice(0, 2).join('\n')}\n</tr>\n<tr>\n${cells.slice(2).join('\n')}\n</tr>\n</table>`;
}

export function fiftyFifty(question) {
  const distractor = letters[(letters.indexOf(question.correct) + 2) % letters.length];
  return letters.filter(letter => letter === question.correct || letter === distractor);
}

export function renderJokers(question, bank, completed, prefix, gamesPath, restartPath) {
  const remaining = fiftyFifty(question).join(' and ');
  return `<table width="100%">
<tr>
<td width="33%" valign="top"><details><summary>${themedImage(`${prefix}trivia-fifty.svg`, '50:50', { width: '100%', title: '50:50: once per run' })}</summary><p><samp>${remaining}</samp></p></details></td>
<td width="34%" valign="top"><details><summary>${themedImage(`${prefix}trivia-hint.svg`, 'Hint', { width: '100%', title: 'Hint: once per run' })}</summary><p><samp>${escapeHtml(question.hint)}</samp></p></details></td>
<td width="33%" valign="top"><details><summary>${themedImage(`${prefix}trivia-cash.svg`, 'Cash out', { width: '100%', title: `Cash out: ${money(bank)}` })}</summary><p><samp>${escapeHtml(money(bank))} / ${completed} of 15 answered.</samp></p><p><a href="${gamesPath}">Games</a> / <a href="${restartPath}">Restart</a></p></details></td>
</tr>
</table>`;
}

export function renderTemplate(template, values, required) {
  const tokens = [...template.matchAll(/\{\{([A-Z_]+)\}\}/g)].map(match => match[1]);
  for (const token of required) {
    assert(tokens.includes(token), `Template must include {{${token}}}`);
  }
  const rendered = template.replace(/\{\{([A-Z_]+)\}\}/g, (_, token) => {
    assert(Object.hasOwn(values, token), `Unknown template token: {{${token}}}`);
    return String(values[token]);
  });
  assert(!/\{\{|\}\}/.test(rendered), 'Unresolved or malformed template token');
  return `${rendered.replace(/\r\n/g, '\n').replace(/\n{3,}/g, '\n\n').trimEnd()}\n`;
}