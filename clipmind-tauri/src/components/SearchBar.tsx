import { useRef, useEffect } from 'react';
import { TABS, type TabId } from '../types';
import { AllIcon, CodeIcon, LinkIcon, OtherIcon, SearchIcon, SettingsIcon } from './Icons';

type Props = {
  query: string;
  onQueryChange: (q: string) => void;
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  count: number;
  onSettingsClick: () => void;
  semanticMode: boolean;
  onSemanticToggle: () => void;
};

const tabIcon = (icon: 'all' | 'code' | 'link' | 'other') => {
  const props = { size: 13 };
  if (icon === 'all') return <AllIcon {...props} />;
  if (icon === 'code') return <CodeIcon {...props} />;
  if (icon === 'link') return <LinkIcon {...props} />;
  return <OtherIcon {...props} />;
};

export const SearchBar = ({ query, onQueryChange, activeTab, onTabChange, count, onSettingsClick, semanticMode, onSemanticToggle }: Props) => {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const handler = () => inputRef.current?.focus();
    window.addEventListener('focus-search', handler);
    return () => window.removeEventListener('focus-search', handler);
  }, []);

  return (
    <div className="search-area">
      <div className="tab-strip">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            className={`tab-pill ${activeTab === tab.id ? 'active' : ''}`}
            onClick={() => onTabChange(tab.id)}
          >
            {tabIcon(tab.icon)}
            <span>{tab.label}</span>
          </button>
        ))}
      </div>
      <div className="search-field-container">
        <div className="search-icon-wrap">
          <SearchIcon size={16} color="var(--text-secondary)" />
        </div>
        <input
          ref={inputRef}
          className="search-input"
          type="text"
          placeholder="Search BuffBrain..."
          value={query}
          onChange={(e) => onQueryChange(e.target.value)}
          spellCheck={false}
          autoCorrect="off"
        />
        <div className="search-right">
          {count > 0 && <span className="clip-count-badge">{count} clip{count !== 1 ? 's' : ''}</span>}
          <button
            className={`semantic-toggle ${semanticMode ? 'active' : ''}`}
            onClick={onSemanticToggle}
            title={semanticMode ? 'Semantic search on' : 'Semantic search off'}
          >
            <span style={{ fontSize: 13 }}>{semanticMode ? '✨' : '🔍'}</span>
          </button>
          <div className="kbd-hint">
            <kbd>⌘</kbd><kbd>⇧</kbd><kbd>V</kbd>
          </div>
          <button className="settings-button" onClick={onSettingsClick} title="Settings (Cmd+,)">
            <SettingsIcon size={14} color="currentColor" />
          </button>
        </div>
      </div>
    </div>
  );
};
