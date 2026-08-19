import { invoke } from '@tauri-apps/api/core';

// --- State ---
let logCount = 0;
let scriptRunning = false;

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

// --- Script Run (placeholder for Task 3.3) ---
window.runScript = function() {
  const code = document.getElementById('editor').value;
  if (!code.trim()) { appendLog('No script to run', 'error'); return; }
  appendLog('Run requested (interpreter integration coming in Task 3.3)', 'info');
  setStatus('running', 'RUNNING');
  setTimeout(function() { setStatus('ready', 'READY'); }, 2000);
};

window.pauseScript = function() { appendLog('Pause requested', 'info'); };
window.stopScript = function() { appendLog('Stop requested', 'info'); setStatus('ready', 'READY'); };

// --- File I/O (placeholder) ---
window.openFile = function() { appendLog('Open file (Tauri dialog coming in Task 3.2)', 'info'); };
window.saveFile = function() { appendLog('Save file (Tauri dialog coming in Task 3.2)', 'info'); };

// --- Init ---
async function init() {
  appendLog('BlackHoleMacro initialized', 'script');
  await loadWindows();
}

init();
