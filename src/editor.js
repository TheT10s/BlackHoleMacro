import { EditorView, basicSetup } from 'codemirror';
import { syntaxHighlighting, HighlightStyle, StreamLanguage } from '@codemirror/language';
import { EditorState } from '@codemirror/state';
import { tags as t } from '@lezer/highlight';

const SINGULARITY_KEYWORDS = [
  'script', 'version', 'on', 'start', 'function', 'call', 'var',
  'if', 'else', 'loop', 'while', 'break', 'restart', 'wait', 'until',
  'return', 'not', 'matches', 'within', 'timeout', 'confidence',
  'pause', 'human', 'true', 'false', 'image', 'region', 'pixel',
];

const ACTION_PATTERN = /^(key|mouse)\.(tap|hold|release|type|click|press|move)\b/;
const IDENTIFIER_PATTERN = /^[a-zA-Z_][a-zA-Z0-9_]*/;

// SingularityScript language definition using stream parser
export const singularityScriptLanguage = StreamLanguage.define({
  name: 'singularityscript',
  token: (stream) => {
    // Skip whitespace
    if (stream.eatSpace()) return null;

    // Comments
    if (stream.match('//')) {
      stream.skipToEnd();
      return t.comment;
    }

    // Colors (#RRGGBB)
    if (stream.match(/#[0-9a-fA-F]{6}/)) {
      return t.color;
    }

    // Strings
    if (stream.match(/"/)) {
      while (!stream.eol()) {
        const ch = stream.next();
        if (ch === '\\') { stream.next(); continue; }
        if (ch === '"') break;
      }
      return t.string;
    }

    // Numbers (integers and floats)
    if (stream.match(/\d+(\.\d+)?/)) {
      return t.number;
    }

    // Range operator (..)
    if (stream.match('..')) {
      return t.operator;
    }

    // Action call syntax: key.tap, mouse.click, etc.
    if (stream.match(ACTION_PATTERN)) {
      return t.function(t.variableName);
    }

    // Keywords (namespace for key/mouse, bool for true/false)
    const word = stream.match(IDENTIFIER_PATTERN);
    if (word) {
      if (SINGULARITY_KEYWORDS.includes(stream.current())) {
        if (stream.current() === 'key' || stream.current() === 'mouse') return t.namespace;
        if (stream.current() === 'true' || stream.current() === 'false') return t.bool;
        return t.keyword;
      }
      return t.variableName;
    }

    // Operators and punctuation
    if (stream.match(/[=<>!+\-*/.,;:{}\[\]()]/)) {
      return t.operator;
    }

    // Fallback
    stream.next();
    return null;
  },
  // Indentation rules
  indent: (state, textAfter) => {
    const lines = state.doc.toString().split('\n');
    const currentLine = lines[state.selection.main.head.line] || '';
    if (/^\s*(}|else\b)/.test(textAfter)) return 0;
    if (/^\s*\{/.test(currentLine)) return 4;
    return 0;
  }
});

// Theme matching the galaxy/space UI
const galaxyTheme = EditorView.theme({
  '&': {
    backgroundColor: '#0b0e22',
    color: '#c9d1d9',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '13px',
    lineHeight: '1.6',
    height: '100%',
    minHeight: '200px',
  },
  '&.cm-focused': {
    outline: 'none',
  },
  '.cm-scroller': {
    fontFamily: "'JetBrains Mono', monospace",
  },
  '.cm-content': {
    padding: '16px',
  },
  '.cm-line': {
    padding: '0',
  },
  '.cm-gutters': {
    backgroundColor: '#0d1127',
    borderRight: '1px solid #1a2050',
    color: '#475569',
  },
  '.cm-lineNumbers': {
    minWidth: '3.5rem',
    paddingRight: '12px',
  },
  '.cm-activeLine': {
    backgroundColor: 'rgba(124, 58, 237, 0.1)',
  },
  '.cm-selectionBackground, .cm-content ::selection': {
    backgroundColor: 'rgba(124, 58, 237, 0.3)',
  },
  // Token colors matching galaxy theme
  '.cm-comment': { color: '#475569', fontStyle: 'italic' },
  '.cm-keyword': { color: '#7c3aed', fontWeight: '600' },
  '.cm-operator': { color: '#22d3ee' },
  '.cm-number': { color: '#fb923c' },
  '.cm-string': { color: '#34d399' },
  '.cm-color': { color: '#f87171', fontWeight: '600' },
  '.cm-variableName': { color: '#e2e8f0' },
  '.cm-function': { color: '#a78bfa' },
  '.cm-namespace': { color: '#7c3aed', fontWeight: '600' },
  '.cm-bool': { color: '#fb923c', fontWeight: '600' },
}, { dark: true });

// Highlight style for tokens (galaxy theme colors)
const highlightStyle = syntaxHighlighting(HighlightStyle.define([
  { tag: t.comment, color: '#475569', fontStyle: 'italic' },
  { tag: t.keyword, color: '#7c3aed', fontWeight: '600' },
  { tag: t.operator, color: '#22d3ee' },
  { tag: t.number, color: '#fb923c' },
  { tag: t.string, color: '#34d399' },
  { tag: t.color, color: '#f87171', fontWeight: '600' },
  { tag: t.variableName, color: '#e2e8f0' },
  { tag: t.function(t.variableName), color: '#a78bfa' },
  { tag: t.namespace, color: '#7c3aed', fontWeight: '600' },
  { tag: t.bool, color: '#fb923c', fontWeight: '600' },
  { tag: t.punctuation, color: '#64748b' },
]));

// Create editor instance (always writable)
export function createEditor(element, initialContent = '') {
  const view = new EditorView({
    doc: initialContent,
    extensions: [
      basicSetup,
      // Explicitly ensure the editor is editable (defensive)
      EditorState.readOnly.of(false),
      EditorView.editable.of(true),
      singularityScriptLanguage,
      galaxyTheme,
      highlightStyle,
      EditorView.lineWrapping,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          window.dispatchEvent(new CustomEvent('editor-change', {
            detail: { content: update.state.doc.toString() }
          }));
        }
      })
    ],
    parent: element
  });
  return view;
}

// Get editor content
export function getEditorContent(view) {
  return view.state.doc.toString();
}

// Set editor content
export function setEditorContent(view, content) {
  view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: content } });
}