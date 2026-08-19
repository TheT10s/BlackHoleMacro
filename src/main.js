import { invoke } from '@tauri-apps/api/core';

const resultEl = document.getElementById('test-result');
const pixelResultEl = document.getElementById('pixel-result');

function showResult(msg) {
  resultEl.textContent = msg;
}

// ─── Input Test Functions ────────────────────────────────────────────────────

window.testMouseMove = async () => {
  try {
    const res = await invoke('mouse_move', { x: 500, y: 300 });
    showResult(res.message);
  } catch (e) {
    showResult('Error: ' + e);
  }
};

window.testMouseClick = async () => {
  try {
    const res = await invoke('mouse_click', { button: 'Left' });
    showResult(res.message);
  } catch (e) {
    showResult('Error: ' + e);
  }
};

window.testKeyTap = async () => {
  try {
    const res = await invoke('key_tap', { key: 'a' });
    showResult(res.message);
  } catch (e) {
    showResult('Error: ' + e);
  }
};

window.testTypeText = async () => {
  const text = document.getElementById('text-input').value;
  if (!text) { showResult('Enter some text first'); return; }
  try {
    const res = await invoke('key_type_text', { text });
    showResult(res.message);
  } catch (e) {
    showResult('Error: ' + e);
  }
};

// ─── Vision Test Functions ────────────────────────────────────────────────────

window.testGetPixel = async () => {
  const x = parseInt(document.getElementById('pixel-x').value) || 0;
  const y = parseInt(document.getElementById('pixel-y').value) || 0;
  try {
    const pixel = await invoke('get_pixel_color', { x, y });
    pixelResultEl.innerHTML =
      '<span style="display:inline-block;width:16px;height:16px;border-radius:3px;background:' + pixel.hex + ';vertical-align:middle;margin-right:8px;border:1px solid #555"></span>' +
      '<strong>' + pixel.hex + '</strong> &mdash; RGB(' + pixel.r + ', ' + pixel.g + ', ' + pixel.b + ') at (' + pixel.x + ', ' + pixel.y + ')';
  } catch (e) {
    pixelResultEl.textContent = 'Error: ' + e;
  }
};

// ─── Init ────────────────────────────────────────────────────────────────────

async function init() {
  const status = document.getElementById('status');
  const windowList = document.getElementById('window-list');

  try {
    const greeting = await invoke('greet', { name: 'BlackHoleMacro' });
    status.textContent = greeting;

    const windows = await invoke('list_windows');
    windowList.innerHTML = windows.map(function(w) {
      return '<div class="window-item">' +
        '<strong>' + w.title + '</strong>' +
        '<span class="window-meta">' + w.process_name + ' (' + w.class_name + ')</span>' +
        '</div>';
    }).join('');

  } catch (e) {
    status.textContent = 'Error: ' + e;
    status.classList.remove('ok');
  }
}

init();