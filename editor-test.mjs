import { EditorState } from '@codemirror/state';
import { basicSetup } from 'codemirror';
import { singularityScriptLanguage } from './src/editor.js';

const DOC = `script "Example" {
    version: 1
    var count = 0
    function attackCycle() {
        loop {
            key.tap("1")
            pause human
            key.tap("4")
            pause 128..200
            if not (pixel(396, 82) matches #008D5B within 10) {
                break
            }
        }
    }
    on start {
        key.hold("<tab>+4")
        pause 500
        key.release("<tab>+4")
        if pixel(396, 82) matches #008D5B within 10 {
            call attackCycle()
        }
    }
}`;

// Build state with the real language. Sync-parse happens at create() -
// a "failed to advance stream" crash would throw here.
const state = EditorState.create({
  doc: DOC,
  extensions: [basicSetup, singularityScriptLanguage, EditorState.readOnly.of(false)]
});

console.log('readOnly:', state.readOnly);
console.log('doc length:', state.doc.length);
console.log('language parsed OK - tokenizer always advances');
