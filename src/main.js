import { invoke } from '@tauri-apps/api/core';

async function init() {
  const status = document.getElementById('status');
  const windowList = document.getElementById('window-list');

  try {
    const greeting = await invoke('greet', { name: 'IrisMacro' });
    status.textContent = greeting;

    // List all visible windows
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
