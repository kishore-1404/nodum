import { useEffect, useState } from 'react';
import { Key, Sun, Moon, Check, ExternalLink } from 'lucide-react';
import { useUIStore } from '../../stores';
import { api } from '../../api/client';
import type { LlmProviderConfig } from '../../types';

const PROVIDERS = [
  { id: 'claude', name: 'Claude (Anthropic)', models: ['claude-sonnet-4-20250514', 'claude-haiku-4-5-20251001'], icon: '🟠' },
  { id: 'openai', name: 'OpenAI', models: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo'], icon: '🟢' },
  { id: 'gemini', name: 'Gemini (Google)', models: ['gemini-2.0-flash', 'gemini-2.0-pro'], icon: '🔵' },
  { id: 'ollama', name: 'Ollama (Local)', models: ['llama3.1', 'mistral', 'deepseek-r1'], icon: '🟣' },
];

export function SettingsView() {
  const { theme, setTheme } = useUIStore();
  const [providers, setProviders] = useState<LlmProviderConfig[]>([]);
  const [editProvider, setEditProvider] = useState<string | null>(null);
  const [apiKey, setApiKey] = useState('');
  const [model, setModel] = useState('');
  const [baseUrl, setBaseUrl] = useState('');
  const [saving, setSaving] = useState(false);
  const [success, setSuccess] = useState('');

  useEffect(() => {
    api.getProviders().then(setProviders).catch(() => {});
  }, []);

  const handleSave = async (providerId: string) => {
    setSaving(true);
    try {
      const result = await api.configureProvider({
        provider: providerId,
        api_key: apiKey || undefined,
        model_name: model,
        base_url: baseUrl || undefined,
        is_default: providers.length === 0,
      });
      setProviders(prev => {
        const filtered = prev.filter(p => p.id !== result.id);
        return [result, ...filtered];
      });
      setEditProvider(null);
      setApiKey('');
      setSuccess(`${providerId} configured successfully`);
      setTimeout(() => setSuccess(''), 3000);
    } catch { /* ignore */ }
    setSaving(false);
  };

  const setDefault = async (providerId: string, modelName: string) => {
    await api.configureProvider({ provider: providerId, model_name: modelName, is_default: true });
    const updated = await api.getProviders();
    setProviders(updated);
    setSuccess('Default provider updated');
    setTimeout(() => setSuccess(''), 3000);
  };

  return (
    <div className="p-8 max-w-2xl mx-auto">
      <h1 className="font-display text-3xl font-bold mb-2" style={{ color: 'var(--text-primary)' }}>Settings</h1>
      <p className="text-sm mb-8" style={{ color: 'var(--text-muted)' }}>Configure your LLM providers and preferences.</p>

      {success && (
        <div className="mb-6 px-4 py-3 rounded-lg flex items-center gap-2 text-sm"
          style={{ background: 'rgba(74,124,89,0.1)', color: '#4a7c59' }}>
          <Check size={16} /> {success}
        </div>
      )}

      {/* Theme toggle */}
      <section className="mb-10">
        <h2 className="font-display text-lg font-semibold mb-4" style={{ color: 'var(--text-primary)' }}>Appearance</h2>
        <div className="flex gap-3">
          <button
            onClick={() => setTheme('dark')}
            className="flex items-center gap-2 px-4 py-3 rounded-lg border text-sm font-medium transition-smooth"
            style={{
              background: theme === 'dark' ? 'var(--accent)' : 'var(--bg-secondary)',
              borderColor: theme === 'dark' ? 'var(--accent)' : 'var(--border)',
              color: theme === 'dark' ? '#fff' : 'var(--text-secondary)',
            }}
          >
            <Moon size={16} /> Dark
          </button>
          <button
            onClick={() => setTheme('light')}
            className="flex items-center gap-2 px-4 py-3 rounded-lg border text-sm font-medium transition-smooth"
            style={{
              background: theme === 'light' ? 'var(--accent)' : 'var(--bg-secondary)',
              borderColor: theme === 'light' ? 'var(--accent)' : 'var(--border)',
              color: theme === 'light' ? '#fff' : 'var(--text-secondary)',
            }}
          >
            <Sun size={16} /> Light
          </button>
        </div>
      </section>

      {/* LLM Providers */}
      <section>
        <h2 className="font-display text-lg font-semibold mb-1" style={{ color: 'var(--text-primary)' }}>LLM Providers</h2>
        <p className="text-xs mb-4" style={{ color: 'var(--text-muted)' }}>
          Configure API keys for the LLMs you want to use. Your keys are stored encrypted.
        </p>

        <div className="space-y-3">
          {PROVIDERS.map(p => {
            const configured = providers.find(up => up.provider === p.id);
            const isEditing = editProvider === p.id;

            return (
              <div key={p.id} className="rounded-xl border p-4" style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)' }}>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span className="text-xl">{p.icon}</span>
                    <div>
                      <span className="font-medium text-sm" style={{ color: 'var(--text-primary)' }}>{p.name}</span>
                      {configured && (
                        <span className="ml-2 text-[10px] font-mono px-1.5 py-0.5 rounded"
                          style={{ background: 'rgba(74,124,89,0.15)', color: '#4a7c59' }}>
                          {configured.is_default ? 'DEFAULT' : 'CONFIGURED'}
                        </span>
                      )}
                      {configured && (
                        <p className="text-xs mt-0.5 font-mono" style={{ color: 'var(--text-muted)' }}>
                          Model: {configured.model_name}
                        </p>
                      )}
                    </div>
                  </div>

                  <div className="flex items-center gap-2">
                    {configured && !configured.is_default && (
                      <button
                        onClick={() => setDefault(p.id, configured.model_name)}
                        className="text-xs px-2 py-1 rounded transition-smooth"
                        style={{ color: 'var(--accent)' }}
                      >
                        Set default
                      </button>
                    )}
                    <button
                      onClick={() => {
                        setEditProvider(isEditing ? null : p.id);
                        setModel(configured?.model_name || p.models[0]);
                        setBaseUrl(configured?.base_url || '');
                        setApiKey('');
                      }}
                      className="flex items-center gap-1 text-xs px-3 py-1.5 rounded-lg transition-smooth"
                      style={{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }}
                    >
                      <Key size={12} /> {configured ? 'Update' : 'Configure'}
                    </button>
                  </div>
                </div>

                {isEditing && (
                  <div className="mt-4 pt-4 border-t space-y-3" style={{ borderColor: 'var(--border)' }}>
                    {p.id !== 'ollama' && (
                      <div>
                        <label className="text-xs font-medium mb-1 block" style={{ color: 'var(--text-secondary)' }}>API Key</label>
                        <input
                          type="password"
                          value={apiKey}
                          onChange={e => setApiKey(e.target.value)}
                          placeholder={configured ? '••••••••' : 'Enter your API key'}
                          className="w-full px-3 py-2 rounded-lg border text-sm focus-ring font-mono"
                          style={{ background: 'var(--bg-tertiary)', borderColor: 'var(--border)', color: 'var(--text-primary)' }}
                        />
                      </div>
                    )}

                    <div>
                      <label className="text-xs font-medium mb-1 block" style={{ color: 'var(--text-secondary)' }}>Model</label>
                      <select
                        value={model}
                        onChange={e => setModel(e.target.value)}
                        className="w-full px-3 py-2 rounded-lg border text-sm focus-ring"
                        style={{ background: 'var(--bg-tertiary)', borderColor: 'var(--border)', color: 'var(--text-primary)' }}
                      >
                        {p.models.map(m => <option key={m} value={m}>{m}</option>)}
                      </select>
                    </div>

                    {(p.id === 'ollama' || p.id === 'openai') && (
                      <div>
                        <label className="text-xs font-medium mb-1 block" style={{ color: 'var(--text-secondary)' }}>
                          Base URL {p.id === 'ollama' && '(default: http://localhost:11434)'}
                        </label>
                        <input
                          type="url"
                          value={baseUrl}
                          onChange={e => setBaseUrl(e.target.value)}
                          placeholder={p.id === 'ollama' ? 'http://localhost:11434' : 'https://api.openai.com/v1'}
                          className="w-full px-3 py-2 rounded-lg border text-sm focus-ring font-mono"
                          style={{ background: 'var(--bg-tertiary)', borderColor: 'var(--border)', color: 'var(--text-primary)' }}
                        />
                      </div>
                    )}

                    <div className="flex gap-2 pt-2">
                      <button
                        onClick={() => handleSave(p.id)}
                        disabled={saving}
                        className="flex-1 py-2 rounded-lg text-sm font-semibold text-white transition-smooth"
                        style={{ background: 'var(--accent)' }}
                      >
                        {saving ? 'Saving...' : 'Save'}
                      </button>
                      <button
                        onClick={() => setEditProvider(null)}
                        className="px-4 py-2 rounded-lg text-sm transition-smooth"
                        style={{ background: 'var(--bg-tertiary)', color: 'var(--text-muted)' }}
                      >
                        Cancel
                      </button>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </section>

      {/* About */}
      <section className="mt-10 pt-8 border-t" style={{ borderColor: 'var(--border)' }}>
        <div className="flex items-center gap-2 mb-2">
          <span className="font-display text-lg font-bold" style={{ color: 'var(--accent)' }}>Nodum</span>
          <span className="text-xs font-mono" style={{ color: 'var(--text-muted)' }}>v0.1.0</span>
        </div>
        <p className="text-xs leading-relaxed" style={{ color: 'var(--text-muted)' }}>
          The Knowledge-Connecting Book Reader. Open source under MIT License.
        </p>
        <a
          href="https://github.com/nodum-app/nodum"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-1 text-xs mt-2 underline"
          style={{ color: 'var(--accent)' }}
        >
          <ExternalLink size={12} /> View on GitHub
        </a>
      </section>
    </div>
  );
}
