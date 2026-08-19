import { defineConfig } from 'vite';
import { normalizePath } from 'vite';

var ROOT = normalizePath(process.cwd());

export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: function(path) {
        var abs = normalizePath(path);
        return abs.includes(ROOT + '/target') ||
               abs.includes(ROOT + '/src-tauri');
      },
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'esnext',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
