import { useEffect, useRef, useState, useCallback } from 'react';
import { api } from './api';
import type { Clip, TabId } from './types';
import { SearchBar } from './components/SearchBar';
import { ClipList } from './components/ClipList';
import { ClipPreview } from './components/ClipPreview';
import { Settings } from './components/Settings';

type View = 'main' | 'settings';

function App() {
  const [clips, setClips] = useState<Clip[]>([]);
  const [query, setQuery] = useState('');
  const [debouncedQuery, setDebouncedQuery] = useState('');
  const [activeTab, setActiveTab] = useState<TabId>('all');
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [view, setView] = useState<View>('main');
  const debounceRef = useRef<number | null>(null);

  const refresh = useCallback(async () => {
    const all = await api.getClips();
    setClips(all);
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    const unlistenFocus = api.onFocusSearch(() => {
      const input = document.querySelector<HTMLInputElement>('.search-input');
      input?.focus();
      input?.select();
    });
    const unlistenUpdated = api.onClipsUpdated(() => refresh());
    const unlistenText = api.onClipboardText(async (p) => {
      await api.insertClip(p.content, null);
      refresh();
    });
    const unlistenImage = api.onClipboardImage(async (thumb) => {
      await api.insertImageClip(thumb, null);
      refresh();
    });
    return () => {
      unlistenFocus.then((fn) => fn());
      unlistenUpdated.then((fn) => fn());
      unlistenText.then((fn) => fn());
      unlistenImage.then((fn) => fn());
    };
  }, [refresh]);

  useEffect(() => {
    if (debounceRef.current) window.clearTimeout(debounceRef.current);
    debounceRef.current = window.setTimeout(() => setDebouncedQuery(query), 150);
    return () => {
      if (debounceRef.current) window.clearTimeout(debounceRef.current);
    };
  }, [query]);

  useEffect(() => {
    api.getConfig().then((c) => {
      document.documentElement.setAttribute('data-theme', c.theme);
    });
  }, []);

  const filtered = clips.filter((c) => {
    if (debouncedQuery) {
      if (!c.content.toLowerCase().includes(debouncedQuery.toLowerCase())) return false;
    }
    if (activeTab === 'code') return c.type === 'code';
    if (activeTab === 'links') return c.type === 'link';
    if (activeTab === 'other') return c.type === 'text' || c.type === 'image';
    return true;
  });

  const selected = filtered.find((c) => c.id === selectedId) || filtered[0] || null;

  const onPaste = async (id: number) => {
    await api.pasteClip(id);
  };

  const onPin = async (id: number) => {
    await api.togglePin(id);
    refresh();
  };

  const onDelete = async (id: number) => {
    await api.deleteClip(id);
    if (selectedId === id) setSelectedId(null);
    refresh();
  };

  const onSelect = (clip: Clip) => setSelectedId(clip.id);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (view === 'settings') {
        if (e.key === 'Escape') setView('main');
        return;
      }
      if (e.key === 'Escape') {
        api.hideWindow();
        return;
      }
      if ((e.metaKey || e.ctrlKey) && e.key >= '1' && e.key <= '4') {
        e.preventDefault();
        const map: TabId[] = ['all', 'code', 'links', 'other'];
        setActiveTab(map[parseInt(e.key) - 1]);
        return;
      }
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        const idx = filtered.findIndex((c) => c.id === selectedId);
        const next = filtered[Math.min(idx + 1, filtered.length - 1)];
        if (next) {
          setSelectedId(next.id);
          scrollIntoView(next.id);
        }
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        const idx = filtered.findIndex((c) => c.id === selectedId);
        const prev = filtered[Math.max(idx - 1, 0)];
        if (prev) {
          setSelectedId(prev.id);
          scrollIntoView(prev.id);
        }
      } else if (e.key === 'Enter' && selected) {
        e.preventDefault();
        onPaste(selected.id);
      } else if (e.key === 'ArrowLeft' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        const map: TabId[] = ['all', 'code', 'links', 'other'];
        const idx = map.indexOf(activeTab);
        setActiveTab(map[Math.max(idx - 1, 0)]);
      } else if (e.key === 'ArrowRight' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        const map: TabId[] = ['all', 'code', 'links', 'other'];
        const idx = map.indexOf(activeTab);
        setActiveTab(map[Math.min(idx + 1, 3)]);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [filtered, selectedId, selected, view, activeTab]);

  const scrollIntoView = (id: number) => {
    requestAnimationFrame(() => {
      const el = document.querySelector(`[data-clip-id="${id}"]`);
      el?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    });
  };

  if (view === 'settings') {
    return <Settings onClose={() => setView('main')} />;
  }

  return (
    <div className="app">
      <SearchBar
        query={query}
        onQueryChange={setQuery}
        activeTab={activeTab}
        onTabChange={setActiveTab}
        count={clips.length}
        onSettingsClick={() => setView('settings')}
      />
      <div className="content">
        <div className="list-pane">
          <ClipList
            clips={filtered}
            selectedId={selectedId}
            onSelect={onSelect}
            onPin={onPin}
            onDelete={onDelete}
            onPaste={onPaste}
          />
        </div>
        <div className="preview-pane">
          <ClipPreview clip={selected} onPaste={onPaste} />
        </div>
      </div>
    </div>
  );
}

export default App;
