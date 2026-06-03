import { Clip, ClipType } from '../types';
import { StarIcon, CloseIcon } from './Icons';

type Props = {
  clips: Clip[];
  selectedId: number | null;
  onSelect: (clip: Clip) => void;
  onPin: (id: number) => void;
  onDelete: (id: number) => void;
  onPaste: (id: number) => void;
};

const typeColor = (t: ClipType): string => {
  switch (t) {
    case 'code': return 'var(--type-code)';
    case 'link': return 'var(--type-link)';
    case 'image': return 'var(--type-image)';
    default: return 'var(--type-text)';
  }
};

const typeBadgeClass = (t: ClipType): string => {
  switch (t) {
    case 'code': return 'clip-type-badge--code';
    case 'link': return 'clip-type-badge--link';
    case 'image': return 'clip-type-badge--image';
    default: return 'clip-type-badge--text';
  }
};

const typeLabel = (t: ClipType): string => {
  switch (t) {
    case 'code': return 'Code';
    case 'link': return 'Link';
    case 'image': return 'Image';
    default: return 'Text';
  }
};

const formatTime = (ts: number): string => {
  const d = new Date(ts);
  const now = new Date();
  const diff = now.getTime() - d.getTime();
  if (diff < 60000) return 'now';
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h`;
  if (diff < 7 * 86400000) return `${Math.floor(diff / 86400000)}d`;
  return d.toLocaleDateString();
};

const previewText = (content: string, max = 150): string => {
  const collapsed = content.replace(/\s+/g, ' ').trim();
  return collapsed.length > max ? collapsed.slice(0, max) + '…' : collapsed;
};

export const ClipList = ({ clips, selectedId, onSelect, onPin, onDelete, onPaste: _onPaste }: Props) => {
  if (clips.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-icon">
          <svg width="22" height="22" viewBox="0 0 16 16" fill="none">
            <rect x="4" y="2.5" width="8" height="12" rx="1.5" stroke="currentColor" strokeWidth="1.3" />
            <path d="M6 1.5h4v2H6z M5.5 6.5h5 M5.5 9h5 M5.5 11.5h3" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
          </svg>
        </div>
        <div className="empty-title">No clips yet</div>
        <div className="empty-subtitle">Copy something to get started</div>
        <div className="empty-shortcut">
          Toggle window: <kbd>⌘</kbd><kbd>⇧</kbd><kbd>V</kbd>
        </div>
      </div>
    );
  }

  return (
    <div className="clip-list">
      {clips.map((clip) => (
        <div
          key={clip.id}
          data-clip-id={clip.id}
          className={`clip-item ${selectedId === clip.id ? 'selected' : ''}`}
          onClick={() => onSelect(clip)}
        >
          <div className="clip-type-bar" style={{ background: typeColor(clip.type) }} />
          <div className="clip-body">
            {clip.type === 'image' && clip.thumbnail ? (
              <div className="clip-image-row">
                <img className="clip-thumb" src={clip.thumbnail} alt="" />
                <div className="clip-image-meta">Image</div>
              </div>
            ) : (
              <div className="clip-text">{previewText(clip.content)}</div>
            )}
            <div className="clip-meta">
              <span className={`clip-type-badge ${typeBadgeClass(clip.type)}`}>
                {typeLabel(clip.type)}
              </span>
              {clip.source && <span className="clip-source">{clip.source}</span>}
              <span className="clip-time">{formatTime(clip.created_at)}</span>
            </div>
          </div>
          <div className="clip-actions">
            <button
              className={`action-btn ${clip.pinned ? 'active' : ''}`}
              onClick={(e) => {
                e.stopPropagation();
                onPin(clip.id);
              }}
              title={clip.pinned ? 'Unpin' : 'Pin'}
            >
              <StarIcon size={10} color="currentColor" filled={clip.pinned} />
            </button>
            <button
              className="action-btn delete"
              onClick={(e) => {
                e.stopPropagation();
                onDelete(clip.id);
              }}
              title="Delete"
            >
              <CloseIcon size={10} color="currentColor" />
            </button>
          </div>
        </div>
      ))}
    </div>
  );
};
