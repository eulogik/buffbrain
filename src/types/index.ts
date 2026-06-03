export type ClipType = 'text' | 'code' | 'link' | 'image';

export interface Clip {
  id: number;
  content: string;
  type: ClipType;
  source: string | null;
  created_at: number;
  pinned: boolean;
  thumbnail: string | null;
  score?: number | null;
}

export type TabId = 'all' | 'code' | 'links' | 'other';

export const TABS: { id: TabId; label: string; icon: 'all' | 'code' | 'link' | 'other' }[] = [
  { id: 'all', label: 'All', icon: 'all' },
  { id: 'code', label: 'Code', icon: 'code' },
  { id: 'links', label: 'Links', icon: 'link' },
  { id: 'other', label: 'Other', icon: 'other' },
];

export type Theme = 'dark' | 'light' | 'system';

export interface AppConfig {
  theme: Theme;
  ai_enabled: boolean;
  auto_hide: boolean;
  max_clips: number;
  show_tray: boolean;
  auto_start: boolean;
}
