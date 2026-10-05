import assert from 'node:assert/strict';
import test from 'node:test';
import { escapeHtml, money, questionFile, renderAnswers, renderJokers, renderTemplate, safetyNet, themedImage, validateOptions } from './lib/trivia.mjs';

test('theme images select dark and light sources with an accessible light fallback', () => {
  const image = themedImage('../assets/trivia-a.svg', 'A & B', { width: '100%', title: '"Hint"' });
  assert(image.includes('media="(prefers-color-scheme: dark)" srcset="../assets/trivia-a-dark.svg"'));
  assert(image.includes('media="(prefers-color-scheme: light)" srcset="../assets/trivia-a.svg"'));
  assert(image.includes('<img src="../assets/trivia-a.svg" alt="A &amp; B" width="100%" title="&quot;Hint&quot;">'));
  assert.throws(() => themedImage('image.png', 'Image'), /must be SVG/);
});

test('answer IDs remain valid regardless of JSON entry order', () => {
  const question = { options: { D: 'Four', B: 'Two', A: 'One', C: 'Three' }, correct: 'B' };
  validateOptions(question);
  assert.equal(question.options[question.correct], 'Two');
});

test('option validation rejects legacy indexes, missing IDs, and duplicate text', () => {
  const options = { A: 'One', B: 'Two', C: 'Three', D: 'Four' };
  assert.throws(() => validateOptions({ options: Object.values(options), correct: 1 }), /object keyed/);
  assert.throws(() => validateOptions({ options, correct: 1 }), /correct must be/);
  assert.throws(() => validateOptions({ options, correct: 'E' }), /correct must be/);
  assert.throws(() => validateOptions({ options: { ...options, E: 'Five' }, correct: 'B' }), /exactly A, B, C, D/);
  assert.throws(() => validateOptions({ options: { A: 'One' }, correct: 'A' }), /exactly A, B, C, D/);
  assert.throws(() => validateOptions({ options: { ...options, D: ' ' }, correct: 'B' }), /nonempty strings/);
  assert.throws(() => validateOptions({ options: { ...options, D: ' One ' }, correct: 'B' }), /duplicate options/);
});

test('safety nets apply only after completing their checkpoint', () => {
  assert.deepEqual([0, 4, 5, 9, 10, 14, 15].map(safetyNet), [0, 0, 1000, 1000, 32000, 32000, 32000]);
});

test('prizes and page names have stable formatting', () => {
  assert.equal(money(1000000), '$1,000,000');
  assert.equal(questionFile(0), 'q01.md');
  assert.equal(questionFile(14), 'q15.md');
});

test('HTML content and attributes are escaped', () => {
  assert.equal(escapeHtml('&<>"\''), '&amp;&lt;&gt;&quot;&#39;');
});

test('templates reject missing, unknown, and malformed tokens', () => {
  assert.throws(() => renderTemplate('{{UNKNOWN}}', {}, []), /Unknown template token/);
  assert.throws(() => renderTemplate('No question', {}, ['QUESTION']), /Template must include/);
  assert.throws(() => renderTemplate('{{wrong}}', {}, []), /Unresolved or malformed/);
});

test('template replacement preserves dollar signs and normalizes whitespace', () => {
  assert.equal(renderTemplate('{{HINT}}\r\n\r\n\r\nEnd  ', { HINT: '$100 and $& stay literal' }, ['HINT']),
    '$100 and $& stay literal\n\nEnd\n');
});

test('answer grids preserve every target in a two-by-two layout', () => {
  const question = { options: { D: 'Five', C: 'Four', A: 'One & two', B: 'Three' } };
  const targets = ['game-over-0.md', 'q02.md', 'game-over-0.md', 'game-over-0.md'];
  const answers = renderAnswers(question, targets, '../assets/');
  const reordered = { options: { A: 'One & two', B: 'Three', C: 'Four', D: 'Five' } };
  assert.equal(answers, renderAnswers(reordered, targets, '../assets/'));
  assert.deepEqual([...answers.matchAll(/href="([^"]+)"/g)].map(match => match[1]), targets);
  assert.equal((answers.match(/<tr>/g) ?? []).length, 2);
  assert.equal((answers.match(/<td /g) ?? []).length, 4);
  assert.equal((answers.match(/<picture>/g) ?? []).length, 4);
  assert(answers.includes('<samp>One &amp; two</samp>'));
});

test('jokers retain the correct choice, escaped hint, bank, and navigation', () => {
  for (const correct of ['A', 'B', 'C', 'D']) {
    const jokers = renderJokers({ correct, hint: 'A & B' }, 1000, 5, '../assets/', '../README.md', 'q01.md');
    const choices = jokers.match(/<samp>([A-D]) and ([A-D])<\/samp>/);
    assert(choices);
    assert(choices.slice(1).includes(correct));
    assert.notEqual(choices[1], choices[2]);
    assert.equal((jokers.match(/<details>/g) ?? []).length, 3);
    assert.equal((jokers.match(/<picture>/g) ?? []).length, 3);
    assert(jokers.includes('<samp>A &amp; B</samp>'));
    assert(jokers.includes('$1,000 / 5 of 15 answered.'));
    assert(jokers.includes('href="../README.md"'));
    assert(jokers.includes('href="q01.md"'));
  }
});