import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn()
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn()
}));

import { isTauriRuntime, invokeCommand, onProgress } from '../tauri';

beforeEach(() => {
  delete (window as any).__TAURI_INTERNALS__;
});

describe('isTauriRuntime', () => {
  it('returns false in jsdom', () => {
    expect(isTauriRuntime()).toBe(false);
  });

  it('returns true when __TAURI_INTERNALS__ exists', () => {
    (window as any).__TAURI_INTERNALS__ = { invoke: vi.fn() };
    expect(isTauriRuntime()).toBe(true);
  });
});

describe('invokeCommand', () => {
  it('returns stub for get_last_file when not in Tauri', async () => {
    const result = await invokeCommand<string>('get_last_file');
    expect(result).toBe('');
  });

  it('returns stub for test_backend when not in Tauri', async () => {
    const result = await invokeCommand<string>('test_backend');
    expect(result).toContain('Browser preview');
  });

  it('returns stub for get_settings when not in Tauri', async () => {
    const result = await invokeCommand<{ lastOutputFolder: string }>('get_settings');
    expect(result).toEqual({ lastOutputFolder: '' });
  });

  it('returns stub for scrape_metadata when not in Tauri', async () => {
    const result = await invokeCommand<{ source: string }>('scrape_metadata');
    expect(result.source).toBe('browser-preview');
  });

  it('returns stub for get_toolchain_status when not in Tauri', async () => {
    const result = await invokeCommand<Array<{ name: string; available: boolean }>>('get_toolchain_status');
    expect(result).toHaveLength(1);
    expect(result[0].name).toBe('psxpackager');
  });

  it('returns stub for run_conversion when not in Tauri', async () => {
    const result = await invokeCommand<{ success: boolean }>('run_conversion');
    expect(result.success).toBe(false);
  });
});

describe('onProgress', () => {
  it('returns a cleanup function when not in Tauri', async () => {
    const cleanup = await onProgress('test', () => {});
    expect(typeof cleanup).toBe('function');
  });
});
