import { invoke } from '@tauri-apps/api/core';

export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauriRuntime()) {
    if (command === 'get_last_file') {
      return Promise.resolve('' as T);
    }

    if (command === 'test_backend') {
      return Promise.resolve('Browser preview is running without the Tauri backend.' as T);
    }

    if (command === 'get_toolchain_status') {
      return Promise.resolve([
        { name: 'psxpackager', available: false, detail: 'Tool probing runs inside Tauri.', path: null }
      ] as T);
    }

    if (command === 'run_conversion') {
      return Promise.resolve({
        success: false,
        message: 'Run this inside Tauri to process queue jobs.',
        output_path: null,
        command_preview: null
      } as T);
    }

    if (command === 'get_settings') {
      return Promise.resolve({ lastOutputFolder: '', gameName: '', gameId: '', windowWidth: 800, windowHeight: 600 } as T);
    }

    if (command === 'save_settings') {
      return Promise.resolve(undefined as T);
    }

    if (command === 'scrape_metadata') {
      return Promise.resolve({
        serial: '',
        title: 'Browser preview: scraping runs inside Tauri.',
        region: 'Unknown',
        coverPath: null,
        source: 'browser-preview',
        cached: false
      } as T);
    }
  }

  return invoke<T>(command, args);
}

export function onProgress(event: string, cb: (data: unknown) => void): Promise<() => void> {
  if (!isTauriRuntime()) return Promise.resolve(() => {});
  import('@tauri-apps/api/event').then(({ listen }) => listen(event, (e) => cb(e.payload)));
  return Promise.resolve(() => {});
}
