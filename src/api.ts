import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { Clip, AppConfig, Theme } from './types';

export const api = {
  getClips: () => invoke<Clip[]>('get_clips'),
  semanticSearch: (query: string) => invoke<Clip[]>('semantic_search', { query }),
  insertClip: (content: string, source: string | null) =>
    invoke<Clip>('insert_clip', { content, source }),
  insertImageClip: (thumbnail: string, source: string | null) =>
    invoke<Clip>('insert_image_clip', { thumbnail, source }),
  togglePin: (id: number) => invoke<boolean>('toggle_pin', { id }),
  deleteClip: (id: number) => invoke<void>('delete_clip', { id }),
  clearUnpinned: () => invoke<void>('clear_unpinned'),
  countClips: () => invoke<number>('count_clips'),
  pasteClip: (id: number) => invoke<void>('paste_clip', { id }),
  hideWindow: () => invoke<void>('hide_window'),
  showWindow: () => invoke<void>('show_window'),
  getConfig: () => invoke<AppConfig>('get_config'),
  setConfig: (theme: Theme | null, aiEnabled: boolean | null, showTray?: boolean | null, autoStart?: boolean | null) =>
    invoke<void>('set_config', { theme, aiEnabled, showTray, autoStart }),
  setApiKey: (key: string) => invoke<void>('set_api_key', { key }),
  hasApiKey: () => invoke<boolean>('has_api_key'),
  deleteApiKey: () => invoke<void>('delete_api_key'),
  getClipboardText: () => invoke<string>('get_clipboard_text'),
  writeClipboardText: (text: string) => invoke<void>('write_clipboard_text', { text }),

  onFocusSearch: (callback: () => void): Promise<UnlistenFn> =>
    listen('focus-search', () => callback()),
  onClipsUpdated: (callback: () => void): Promise<UnlistenFn> =>
    listen('clips-updated', () => callback()),
  onClipboardText: (
    callback: (payload: { content: string; type: string }) => void
  ): Promise<UnlistenFn> => listen('clipboard-text', (e) => callback(e.payload as any)),
  onClipboardImage: (callback: (thumbnail: string) => void): Promise<UnlistenFn> =>
    listen('clipboard-image', (e) => callback(e.payload as any)),
};
