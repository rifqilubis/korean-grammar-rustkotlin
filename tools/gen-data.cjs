#!/usr/bin/env node
/* =====================================================================
   gen-data.cjs — mengubah js/data.js (versi web) menjadi modul Rust
   `kg-core/src/data_gen.rs`.

   Sumber kebenaran tetap tools/data-final.js di repo web. Alurnya:

       # di repo web
       node tools/build-data.js          # data-final.js -> js/data.js
       # di repo ini
       node tools/gen-data.cjs [path/ke/js/data.js]

   Jalankan: node tools/gen-data.cjs ../korean-grammar/js/data.js
   ===================================================================== */
'use strict';

const fs = require('fs');
const path = require('path');

const DEFAULT_SRC = path.join(__dirname, '..', '..', 'korean-grammar', 'js', 'data.js');
const OUT = path.join(__dirname, '..', 'kg-core', 'src', 'data_gen.rs');

const srcPath = path.resolve(process.argv[2] || DEFAULT_SRC);
if (!fs.existsSync(srcPath)) {
  console.error('Berkas data tidak ditemukan: ' + srcPath);
  process.exit(1);
}

// data.js hanya mendaftarkan window.KG_TOPICS; jalankan di sandbox kecil.
const window = {};
new Function('window', fs.readFileSync(srcPath, 'utf8'))(window);
const TOPICS = window.KG_TOPICS;
if (!Array.isArray(TOPICS) || !TOPICS.length) {
  console.error('window.KG_TOPICS kosong — data.js rusak?');
  process.exit(1);
}

/* ------------------------------------------------------------ validasi */
const problems = [];
function need(cond, msg) { if (!cond) problems.push(msg); }

TOPICS.forEach((t, i) => {
  ['id', 'no', 'title', 'titleId', 'level', 'icon', 'intro'].forEach((k) =>
    need(t[k] !== undefined && t[k] !== null, 'topik #' + i + ' tidak punya "' + k + '"'));
  need(Array.isArray(t.blocks), 'topik ' + t.id + ' tanpa blocks');
  need(Array.isArray(t.quiz), 'topik ' + t.id + ' tanpa quiz');
  need(Array.isArray(t.lat), 'topik ' + t.id + ' tanpa lat');
  (t.quiz || []).forEach((q, qi) => {
    need(q.options && q.options.length === 4, 'topik ' + t.id + ' soal ' + (qi + 1) + ' pilihan bukan 4');
    need(Number.isInteger(q.answer) && q.answer >= 0 && q.answer < 4, 'topik ' + t.id + ' soal ' + (qi + 1) + ' indeks jawaban salah');
    need(!!q.why, 'topik ' + t.id + ' soal ' + (qi + 1) + ' tanpa pembahasan');
    need(!!q.q, 'topik ' + t.id + ' soal ' + (qi + 1) + ' tanpa pertanyaan');
  });
  (t.blocks || []).forEach((b, bi) => {
    const ok = ['heading', 'text', 'formula', 'rules', 'examples', 'note'].includes(b.type);
    need(ok, 'topik ' + t.id + ' blok ' + bi + ' tipe tidak dikenal: ' + b.type);
  });
});
if (problems.length) {
  console.error('Data tidak valid:\n  ' + problems.join('\n  '));
  process.exit(1);
}

/* --------------------------------------------------------- serialisasi */
const q = (s) => JSON.stringify(String(s == null ? '' : s));

const lines = [];
const w = (s) => lines.push(s);

w('// BERKAS HASIL BUILD — jangan disunting manual.');
w('// Dihasilkan oleh tools/gen-data.cjs dari js/data.js milik aplikasi web.');
w('// Bangun ulang: node tools/gen-data.cjs <path/ke/korean-grammar/js/data.js>');
w('//');
w('// ' + TOPICS.length + ' topik · ' + TOPICS.reduce((n, t) => n + t.quiz.length, 0) + ' soal · ' +
  TOPICS.reduce((n, t) => n + t.lat.length, 0) + ' latihan menulis.');
w('#![allow(clippy::all)]');
w('');
w('use crate::model::*;');
w('');
w('/// Seluruh bank materi, urut nomor topik.');
w('pub static TOPICS: &[Topic] = &[');

for (const t of TOPICS) {
  w('    Topic {');
  w('        id: ' + q(t.id) + ',');
  w('        no: ' + t.no + ',');
  w('        title: ' + q(t.title) + ',');
  w('        title_id: ' + q(t.titleId) + ',');
  w('        level: ' + t.level + ',');
  w('        icon: ' + q(t.icon) + ',');
  w('        intro: ' + q(t.intro) + ',');
  w('        blocks: &[');
  for (const b of t.blocks) {
    switch (b.type) {
      case 'heading': w('            Block::Heading { text: ' + q(b.text) + ' },'); break;
      case 'text': w('            Block::Text { text: ' + q(b.text) + ' },'); break;
      case 'note': w('            Block::Note { text: ' + q(b.text) + ' },'); break;
      case 'formula':
        w('            Block::Formula { parts: &[' + b.parts.map(q).join(', ') + '] },');
        break;
      case 'rules':
        w('            Block::Rules { items: &[');
        for (const r of b.items) {
          w('                Rule { a: ' + q(r.a) + ', b: ' + q(r.b || '') + ', c: ' + q(r.c || '') + ' },');
        }
        w('            ] },');
        break;
      case 'examples':
        w('            Block::Examples { items: &[');
        for (const e of b.items) {
          w('                Example { ko: ' + q(e.ko) + ', rom: ' + q(e.rom || '') +
            ', id: ' + q(e.id || '') + ', why: ' + q(e.why || '') + ' },');
        }
        w('            ] },');
        break;
      default: throw new Error('tipe blok tak dikenal: ' + b.type);
    }
  }
  w('        ],');
  w('        quiz: &[');
  for (const item of t.quiz) {
    w('            Question {');
    w('                q: ' + q(item.q) + ',');
    w('                options: [' + item.options.map(q).join(', ') + '],');
    w('                answer: ' + item.answer + ',');
    w('                why: ' + q(item.why) + ',');
    w('            },');
  }
  w('        ],');
  w('        lat: &[');
  for (const l of t.lat) {
    w('            Exercise { ko: ' + q(l.ko) + ', rom: ' + q(l.rom || '') + ', id: ' + q(l.id || '') + ' },');
  }
  w('        ],');
  w('    },');
}
w('];');
w('');

fs.writeFileSync(OUT, lines.join('\n'), 'utf8');
console.log('Ditulis: ' + OUT);
console.log('  ' + TOPICS.length + ' topik · ' +
  TOPICS.reduce((n, t) => n + t.quiz.length, 0) + ' soal · ' +
  TOPICS.reduce((n, t) => n + t.lat.length, 0) + ' latihan');
