import { Clip, ClipType } from '../types';
import { PasteIcon, CodeIcon, LinkIcon, AllIcon } from './Icons';

type Props = {
  clip: Clip | null;
  onPaste: (id: number) => void;
};

const typeIcon = (t: ClipType, size = 13) => {
  switch (t) {
    case 'code': return <CodeIcon size={size} />;
    case 'link': return <LinkIcon size={size} />;
    case 'image': return <AllIcon size={size} />;
    default: return <AllIcon size={size} />;
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
  if (diff < 60000) return 'Just now';
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
  if (diff < 7 * 86400000) return `${Math.floor(diff / 86400000)}d ago`;
  return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
};

export const ClipPreview = ({ clip, onPaste }: Props) => {
  if (!clip) {
    return (
      <div className="clip-preview-empty">
        <div className="clip-preview-empty-icon">
          <svg width="18" height="18" viewBox="0 0 16 16" fill="none">
            <rect x="4" y="2.5" width="8" height="12" rx="1.5" stroke="currentColor" strokeWidth="1.3" />
            <path d="M6 1.5h4v2H6z M5.5 6.5h5 M5.5 9h5 M5.5 11.5h3" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
          </svg>
        </div>
        Select a clip to preview
      </div>
    );
  }

  return (
    <div className="clip-preview-panel">
      <div className="clip-preview-header">
        <div className="clip-preview-type">
          {typeIcon(clip.type)}
          <span>{typeLabel(clip.type)}</span>
          {clip.pinned && <span style={{ marginLeft: 4, opacity: 0.5, fontSize: 10 }}>• Pinned</span>}
        </div>
        <div className="clip-preview-time">{formatTime(clip.created_at)}</div>
      </div>
      <div className="clip-preview-body">
        {clip.type === 'image' && clip.thumbnail ? (
          <img className="preview-image" src={clip.thumbnail} alt="" />
        ) : (
          <pre className="preview-text">{clip.content}</pre>
        )}
      </div>
      <div className="clip-preview-footer">
        <button className="paste-button" onClick={() => onPaste(clip.id)}>
          <PasteIcon size={12} color="currentColor" />
          Paste (Enter)
        </button>
      </div>
    </div>
  );
};
