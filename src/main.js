import { invoke } from '@tauri-apps/api/core';
import { createEditor, getEditorContent, setEditorContent } from './editor.js';

// --- State ---
let logCount = 0;
let scriptRunning = false;
let editorView = null;

// --- Helpers ---
function timestamp() {
  const d = new Date();
  return d.getHours().toString().padStart(2,'0') + ':' +
         d.getMinutes().toString().padStart(2,'0') + ':' +
         d.getSeconds().toString().padStart(2,'0');
}

function appendLog(msg, type) {
  const body = document.getElementById('log-body');
  const entry = document.createElement('div');
  entry.className = 'log-entry ' + (type || '');
  entry.innerHTML = '<span class="time">' + timestamp() + '</span><span class="msg"></span>';
  entry.querySelector('.msg').textContent = msg;
  body.appendChild(entry);
  body.scrollTop = body.scrollHeight;
  logCount++;
  document.getElementById('log-count').textContent = logCount + ' entries';
}

window.clearLog = function() {
  document.getElementById('log-body').innerHTML = '';
  logCount = 0;
  document.getElementById('log-count').textContent = '0 entries';
};

function setStatus(state, text) {
  const badge = document.getElementById('status-badge');
  badge.className = 'status-badge ' + state;
  badge.textContent = text;
}

// --- Window List ---
async function loadWindows() {
  try {
    const windows = await invoke('list_windows');
    const list = document.getElementById('window-list');
    const select = document.getElementById('target-window');
    list.innerHTML = '';
    select.innerHTML = '<option value="">No window selected</option>';
    windows.forEach(function(w) {
      // Sidebar list
      const div = document.createElement('div');
      div.className = 'window-item';
      div.innerHTML = '<div class="title">' + w.title + '</div>' +
                      '<div class="meta">' + w.process_name + ' (' + w.class_name + ')</div>';
      div.onclick = function() {
        document.querySelectorAll('.window-item').forEach(function(el) { el.classList.remove('selected'); });
        div.classList.add('selected');
        select.value = w.title;
        appendLog('Target: ' + w.title, 'info');
      };
      list.appendChild(div);
      // Header dropdown
      const opt = document.createElement('option');
      opt.value = w.title;
      opt.textContent = w.title;
      select.appendChild(opt);
    });
  } catch (e) {
    appendLog('Failed to load windows: ' + e, 'error');
  }
}

// --- Pixel Picker ---
window.pickPixel = async function() {
  const x = parseInt(document.getElementById('pick-x').value) || 0;
  const y = parseInt(document.getElementById('pick-y').value) || 0;
  try {
    const pixel = await invoke('get_pixel_color', { x, y });
    document.getElementById('color-swatch').style.background = pixel.hex;
    document.getElementById('color-hex').textContent = pixel.hex;
    document.getElementById('color-rgb').textContent = 'rgb(' + pixel.r + ', ' + pixel.g + ', ' + pixel.b + ')';
    appendLog('pixel(' + x + ', ' + y + ') = ' + pixel.hex, 'match');
  } catch (e) {
    appendLog('Pixel pick failed: ' + e, 'error');
  }
};

// --- Script Run ---
let scriptRunning = false;

window.runScript = async function() {
  let code;
  if (editorView) {
    code = getEditorContent(editorView);
  } else {
    const fallback = document.getElementById('editor-fallback');
    code = fallback ? fallback.value : document.getElementById('editor').value;
  }
  if (!code || !code.trim()) { appendLog('No script to run', 'error'); return; }
  if (scriptRunning) { appendLog('Script already running', 'error'); return; }

  scriptRunning = true;
  setStatus('running', 'RUNNING');
  appendLog('Starting script execution', 'script');

  try {
    const log = await invoke('run_script', { script: code });
    // Process log events
    for (const event of log) {
      if (event.Info) {
        appendLog(event.Info, 'action');
      } else if (event.VariableChanged) {
        appendLog(event.VariableChanged[0] + ' = ' + event.VariableChanged[1], 'var');
      } else if (event.FunctionCalled) {
        appendLog('Function called: ' + event.FunctionCalled, 'info');
      } else if (event.ScriptStarted) {
        appendLog('Script started: ' + event.ScriptStarted, 'script');
      } else if (event.ScriptFinished) {
        appendLog('Script finished: ' + event.ScriptFinished[0] + (event.ScriptFinished[1] ? ' (success)' : ' (stopped)'), 'script');
      }
    }
    scriptRunning = false;
    setStatus('ready', 'READY');
    appendLog('Script execution complete', 'script');
  } catch (e) {
    scriptRunning = false;
    setStatus('error', 'ERROR');
    appendLog('Script error: ' + e, 'error');
  }
};

window.pauseScript = async function() {
  if (!scriptRunning) { appendLog('No script running to pause', 'info'); return; }
  appendLog('Pausing script', 'info');
  try {
    await invoke('pause_script');
    setStatus('ready', 'PAUSED');
  } catch (e) {
    appendLog('Pause failed: ' + e, 'error');
  }
};

window.stopScript = async function() {
  appendLog('Stopping script', 'info');
  try {
    await invoke('stop_script');
    scriptRunning = false;
    setStatus('ready', 'READY');
    appendLog('Script stopped', 'script');
  } catch (e) {
    appendLog('Stop failed: ' + e, 'error');
    scriptRunning = false;
    setStatus('ready', 'READY');
  }
};

// --- File I/O (placeholder) ---
window.openFile = function() { appendLog('Open file (Tauri dialog coming in Task 3.2)', 'info'); };
window.saveFile = function() { appendLog('Save file (Tauri dialog coming in Task 3.2)', 'info'); };

// --- Init ---
async function init() {
  appendLog('BlackHoleMacro initialized', 'script');
  await loadWindows();

  // Initialize CodeMirror editor
  const editorElement = document.getElementById('editor');
  if (editorElement) {
    const defaultScript = `script "Example" {
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
        pause 500

        if pixel(396, 82) matches #008D5B within 10 {
            call attackCycle()
        }
    }
}`;
    try {
      editorView = createEditor(editorElement, defaultScript);
      // Expose for debugging in devtools console
      window.editorView = editorView;
    } catch (err) {
      console.error('CodeMirror init failed, falling back to textarea:', err);
      appendLog('Editor fallback (CodeMirror error): ' + err, 'error');
      const ta = document.createElement('textarea');
      ta.id = 'editor-fallback';
      ta.spellcheck = false;
      ta.value = defaultScript;
      ta.style.cssText = 'width:100%;height:100%;background:#0b0e22;color:#c9d1d9;border:none;padding:16px;font-family:JetBrains Mono,monospace;font-size:13px;line-height:1.6;resize:none;';
      editorElement.innerHTML = '';
      editorElement.appendChild(ta);
    }

    // Listen for content changes
    window.addEventListener('editor-change', (e) => {
      // Could update file name indicator, etc.
    });
  }
}

init();
