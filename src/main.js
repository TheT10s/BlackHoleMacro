import { invoke } from '@tauri-apps/api/core';

const resultEl = document.getElementById('test-result');

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

// ─── Init ────────────────────────────────────────────────────────────────────

async function init() {
  const status = document.getElementById('status');
  const windowList = document.getElementById('window-list');

  try {
    const greeting = await invoke('greet', { name: 'BlackHoleMacro' });
    status.textContent = greeting;

    const windows = await invoke('list_windows');
    windowList.innerHTML = windows.map(w =>
      `<div class="window-item">
        <strong>${w.title}</strong>
        <span class="window-meta">${w.process_name} (${w.class_name})</span>
      </div>`
    ).join('');

  } catch (e) {
    status.textContent = 'Error: ' + e;
    status.classList.remove('ok');
  }
}

init();
