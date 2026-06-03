import type { JSX } from 'react';

type IconProps = { size?: number; color?: string };

export const AllIcon = ({ size = 14, color = 'currentColor' }: IconProps): JSX.Element => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
    <rect x="2" y="3" width="3" height="3" rx="0.7" stroke={color} strokeWidth="1.4" />
    <rect x="2" y="8" width="3" height="3" rx="0.7" stroke={color} strokeWidth="1.4" />
    <rect x="7" y="3" width="7" height="2" rx="0.7" stroke={color} strokeWidth="1.4" />
    <rect x="7" y="8" width="7" height="2" rx="0.7" stroke={color} strokeWidth="1.4" />
  </svg>
);

export const CodeIcon = ({ size = 14, color = 'currentColor' }: IconProps): JSX.Element => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
    <path d="M5.5 4.5L2 8l3.5 3.5M10.5 4.5L14 8l-3.5 3.5" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
);

export const LinkIcon = ({ size = 14, color = 'currentColor' }: IconProps): JSX.Element => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
    <path d="M7 9.5l2-2M6 6.5l1-1a2.5 2.5 0 113.5 3.5l-1 1M10 9.5l-1 1a2.5 2.5 0 11-3.5-3.5l1-1" stroke={color} strokeWidth="1.4" strokeLinecap="round" />
  </svg>
);

export const OtherIcon = ({ size = 14, color = 'currentColor' }: IconProps): JSX.Element => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
    <circle cx="4" cy="8" r="1" fill={color} />
    <circle cx="8" cy="8" r="1" fill={color} />
    <circle cx="12" cy="8" r="1" fill={color} />
  </svg>
);

export const SearchIcon = ({ size = 16, color = 'currentColor' }: IconProps): JSX.Element => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
    <circle cx="7" cy="7" r="4.5" stroke={color} strokeWidth="1.4" />
    <path d="M10.5 10.5L14 14" stroke={color} strokeWidth="1.4" strokeLinecap="round" />
  </svg>
);

export const SettingsIcon = ({ size = 16, color = 'currentColor' }: IconProps): JSX.Element => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
    <circle cx="8" cy="8" r="1.7" stroke={color} strokeWidth="1.4" />
    <path d="M13.5 8a5.5 5.5 0 00-.1-1.1l1.2-1-1.3-2.2-1.5.5a5.5 5.5 0 00-1.9-1.1L9.5 1.5h-3l-.4 1.6a5.5 5.5 0 00-1.9 1.1l-1.5-.5L1.4 5.9l1.2 1A5.5 5.5 0 002.5 8c0 .4 0 .8.1 1.1l-1.2 1 1.3 2.2 1.5-.5a5.5 5.5 0 001.9 1.1l.4 1.6h3l.4-1.6a5.5 5.5 0 001.9-1.1l1.5.5 1.3-2.2-1.2-1c.1-.3.1-.7.1-1.1z" stroke={color} strokeWidth="1.2" />
  </svg>
);

export const StarIcon = ({ size = 12, color = 'currentColor', filled = false }: IconProps & { filled?: boolean }): JSX.Element => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill={filled ? color : 'none'}>
    <path d="M8 1.5l1.95 4.4 4.55.5-3.5 3.1.95 4.5L8 11.7l-3.95 2.3.95-4.5-3.5-3.1 4.55-.5L8 1.5z" stroke={color} strokeWidth="1.2" strokeLinejoin="round" />
  </svg>
);

export const CloseIcon = ({ size = 12, color = 'currentColor' }: IconProps): JSX.Element => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
    <path d="M4 4l8 8M12 4l-8 8" stroke={color} strokeWidth="1.5" strokeLinecap="round" />
  </svg>
);

export const BackIcon = ({ size = 14, color = 'currentColor' }: IconProps): JSX.Element => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
    <path d="M10 4L6 8l4 4" stroke={color} strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
);

export const ClipboardIcon = ({ size = 14, color = 'currentColor' }: IconProps): JSX.Element => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
    <rect x="4" y="2.5" width="8" height="12" rx="1.5" stroke={color} strokeWidth="1.3" />
    <path d="M6 1.5h4v2H6z M5.5 6.5h5 M5.5 9h5 M5.5 11.5h3" stroke={color} strokeWidth="1.2" strokeLinecap="round" />
  </svg>
);

export const PasteIcon = ({ size = 13, color = 'currentColor' }: IconProps): JSX.Element => (
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
    <rect x="2" y="3" width="9" height="11" rx="1.5" stroke={color} strokeWidth="1.3" />
    <path d="M5 14V5h6v9" stroke={color} strokeWidth="1.2" />
    <path d="M5 8.5h6M5 11h4" stroke={color} strokeWidth="1.2" strokeLinecap="round" />
    <path d="M4 3V1.5a.5.5 0 01.5-.5h4a.5.5 0 01.5.5V3" stroke={color} strokeWidth="1.2" />
  </svg>
);
