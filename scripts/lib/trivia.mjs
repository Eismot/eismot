import assert from 'node:assert/strict';

export const letters = ['A', 'B', 'C', 'D'];
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