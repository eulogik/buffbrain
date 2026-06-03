import { useState, useEffect } from 'react';
import { api } from '../api';
import { Theme } from '../types';
import { BackIcon } from './Icons';

type Props = {
  onClose: () => void;
};

export const Settings = ({ onClose }: Props) => {
  const [theme, setTheme] = useState<Theme>('system');
  const [aiEnabled, setAiEnabled] = useState(false);
  const [apiKey, setApiKey] = useState('');
  const [hasKey, setHasKey] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    api.getConfig().then((c) => {
      setTheme(c.theme);
      setAiEnabled(c.ai_enabled);
    });
    api.hasApiKey().then(setHasKey);
  }, []);

  const save = async () => {
    setSaving(true);
    try {
      if (apiKey) {
        await api.setApiKey(apiKey);
        setHasKey(true);
        setApiKey('');
      }
      await api.setConfig(theme, aiEnabled);
      await new Promise((r) => setTimeout(r, 300));
    } finally {
      setSaving(false);
    }
  };

  const removeKey = async () => {
    await api.deleteApiKey();
    setHasKey(false);
  };

  return (
    <div className="settings">
      <div className="settings-header">
        <button className="back-button" onClick={onClose} title="Back">
          <BackIcon size={14} />
        </button>
        <div className="settings-title">Settings</div>
      </div>
      <div className="settings-body">
        <div className="settings-section">
          <div className="settings-label">Theme</div>
          <div className="settings-sublabel">Choose your preferred appearance</div>
          <div className="theme-toggle">
            {(['dark', 'light', 'system'] as Theme[]).map((t) => (
              <button
                key={t}
                className={theme === t ? 'active' : ''}
                onClick={() => setTheme(t)}
              >
                {t.charAt(0).toUpperCase() + t.slice(1)}
              </button>
            ))}
          </div>
        </div>

        <div className="settings-section">
          <div className="settings-row">
            <div>
              <div className="settings-label">AI Categorization</div>
              <div className="settings-sublabel">Use OpenRouter to auto-detect code vs text vs links</div>
            </div>
            <label className="switch">
              <input
                type="checkbox"
                checked={aiEnabled}
                onChange={(e) => setAiEnabled(e.target.checked)}
              />
              <span className="slider" />
            </label>
          </div>
        </div>

        <div className="settings-section">
          <div className="settings-label">OpenRouter API Key</div>
          <div className="settings-sublabel">Stored securely in macOS Keychain</div>
          {hasKey ? (
            <div className="api-key-row">
              <span className="api-key-status">✓ Key saved</span>
              <button className="link-button" onClick={removeKey}>Remove</button>
            </div>
          ) : (
            <div className="api-key-row">
              <input
                type="password"
                className="api-key-input"
                placeholder="sk-or-..."
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
              />
            </div>
          )}
        </div>

        <button className="save-button" onClick={save} disabled={saving}>
          {saving ? 'Saving...' : 'Save Changes'}
        </button>
      </div>
    </div>
  );
};
