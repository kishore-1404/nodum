import { useState, useRef, useEffect } from 'react';
import { Send, Check, X, ChevronDown, ChevronUp, Sparkles, AlertCircle, Lightbulb, Link2 } from 'lucide-react';
import { useDockStore, useGraphStore } from '../../stores';
import { clsx } from 'clsx';
import type { ThoughtResponse } from '../../types';

export function ThinkingDock() {
  const [input, setInput] = useState('');
  const [historyOpen, setHistoryOpen] = useState(false);
  const { isProcessing, lastResponse, history, error, submitThought, confirmNode, clearError } = useDockStore();
  const { fetchGraph } = useGraphStore();
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const responseRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (lastResponse && responseRef.current) {
      responseRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [lastResponse]);

  const handleSubmit = async () => {
    if (!input.trim() || isProcessing) return;
    const text = input.trim();
    setInput('');
    try {
      await submitThought(text);
    } catch { /* error shown in UI */ }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      handleSubmit();
    }
  };

  const handleConfirmNode = async (response: ThoughtResponse) => {
    const { node_proposal, connections } = response.response;
    const confirmedConnections = connections.map(c => ({
      target_node_id: c.existing_node_id,
      relation_type: c.relation_type,
    }));
    await confirmNode(
      response.thought_id,
      node_proposal.label,
      node_proposal.description,
      node_proposal.node_type,
      confirmedConnections
    );
    await fetchGraph();
  };

  const statusIcon = (status: string) => {
    switch (status) {
      case 'correct': return <Check size={14} className="text-[#4a7c59]" />;
      case 'partially_correct': return <AlertCircle size={14} className="text-[#b8860b]" />;
      case 'incorrect': return <X size={14} className="text-[#a0522d]" />;
      case 'vague': return <Lightbulb size={14} className="text-[#7d7464]" />;
      default: return null;
    }
  };

  const statusLabel = (status: string) => {
    switch (status) {
      case 'correct': return 'You got it right';
      case 'partially_correct': return 'Partially correct';
      case 'incorrect': return 'Needs correction';
      case 'vague': return 'Try being more specific';
      default: return status;
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="px-4 py-3 border-b flex-shrink-0" style={{ borderColor: 'var(--border)' }}>
        <div className="flex items-center gap-2">
          <Sparkles size={16} style={{ color: 'var(--accent)' }} />
          <h3 className="font-display text-sm font-semibold" style={{ color: 'var(--text-primary)' }}>
            Thinking Dock
          </h3>
        </div>
        <p className="text-xs mt-0.5" style={{ color: 'var(--text-muted)' }}>
          Drop your rough thoughts here. No need to be formal.
        </p>
      </div>

      {/* Response area */}
      <div className="flex-1 overflow-auto px-4 py-3 space-y-4">
        {error && (
          <div className="p-3 rounded-lg text-xs border" style={{ background: 'rgba(160,82,45,0.1)', borderColor: 'rgba(160,82,45,0.2)', color: '#a0522d' }}>
            {error}
            <button onClick={clearError} className="ml-2 underline">Dismiss</button>
          </div>
        )}

        {/* Processing indicator */}
        {isProcessing && (
          <div className="flex items-center gap-3 p-4 rounded-lg" style={{ background: 'var(--bg-tertiary)' }}>
            <div className="flex gap-1">
              <div className="w-2 h-2 rounded-full thinking-indicator" style={{ background: 'var(--accent)', animationDelay: '0ms' }} />
              <div className="w-2 h-2 rounded-full thinking-indicator" style={{ background: 'var(--accent)', animationDelay: '200ms' }} />
              <div className="w-2 h-2 rounded-full thinking-indicator" style={{ background: 'var(--accent)', animationDelay: '400ms' }} />
            </div>
            <span className="text-xs" style={{ color: 'var(--text-muted)' }}>Checking your understanding...</span>
          </div>
        )}

        {/* Latest response */}
        {lastResponse && !isProcessing && (
          <div ref={responseRef} className="space-y-3 animate-slide-up">
            {/* Accuracy check */}
            <div className="p-3 rounded-lg border" style={{ background: 'var(--bg-tertiary)', borderColor: 'var(--border)' }}>
              <div className="flex items-center gap-2 mb-2">
                {statusIcon(lastResponse.response.accuracy_check.status)}
                <span className={clsx('text-xs font-semibold', `status-${lastResponse.response.accuracy_check.status}`)}>
                  {statusLabel(lastResponse.response.accuracy_check.status)}
                </span>
              </div>
              <p className="text-sm leading-relaxed" style={{ color: 'var(--text-secondary)' }}>
                {lastResponse.response.accuracy_check.explanation}
              </p>
              {lastResponse.response.accuracy_check.author_context && (
                <p className="text-xs mt-2 pt-2 border-t italic" style={{ color: 'var(--text-muted)', borderColor: 'var(--border)' }}>
                  {lastResponse.response.accuracy_check.author_context}
                </p>
              )}
            </div>

            {/* Connections */}
            {lastResponse.response.connections.length > 0 && (
              <div className="p-3 rounded-lg border" style={{ background: 'var(--bg-tertiary)', borderColor: 'var(--border)' }}>
                <div className="flex items-center gap-2 mb-2">
                  <Link2 size={14} style={{ color: 'var(--accent)' }} />
                  <span className="text-xs font-semibold" style={{ color: 'var(--accent)' }}>Connections found</span>
                </div>
                {lastResponse.response.connections.map((conn, i) => (
                  <div key={i} className="flex items-start gap-2 mt-2 first:mt-0">
                    <span className="text-xs font-mono px-1.5 py-0.5 rounded mt-0.5" style={{ background: 'var(--bg-surface)', color: 'var(--text-secondary)' }}>
                      {conn.relation_type.replace(/_/g, ' ')}
                    </span>
                    <div>
                      <span className="text-sm font-medium" style={{ color: 'var(--text-primary)' }}>
                        {conn.existing_node_label}
                      </span>
                      <p className="text-xs mt-0.5" style={{ color: 'var(--text-muted)' }}>
                        {conn.explanation}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {/* Node proposal */}
            <div className="p-3 rounded-lg border" style={{ background: 'var(--bg-tertiary)', borderColor: 'var(--accent)', borderWidth: '1px' }}>
              <div className="flex items-center justify-between mb-2">
                <div>
                  <span className="text-xs font-semibold" style={{ color: 'var(--accent)' }}>Proposed Node</span>
                  <span className="text-xs ml-2 px-1.5 py-0.5 rounded" style={{ background: 'var(--bg-surface)', color: 'var(--text-muted)' }}>
                    {lastResponse.response.node_proposal.node_type}
                  </span>
                </div>
                <span className="text-xs font-mono" style={{ color: 'var(--text-muted)' }}>
                  {(lastResponse.response.node_proposal.confidence_score * 100).toFixed(0)}%
                </span>
              </div>
              <h4 className="font-display font-semibold text-sm" style={{ color: 'var(--text-primary)' }}>
                {lastResponse.response.node_proposal.label}
              </h4>
              <p className="text-xs mt-1 leading-relaxed" style={{ color: 'var(--text-secondary)' }}>
                {lastResponse.response.node_proposal.description}
              </p>

              {/* Confirm / dismiss */}
              <div className="flex gap-2 mt-3">
                <button
                  onClick={() => handleConfirmNode(lastResponse)}
                  className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-lg text-xs font-semibold text-white transition-smooth focus-ring"
                  style={{ background: 'var(--accent)' }}
                >
                  <Check size={14} /> Add to Graph
                </button>
                <button
                  className="px-3 py-2 rounded-lg text-xs font-medium transition-smooth focus-ring"
                  style={{ background: 'var(--bg-surface)', color: 'var(--text-muted)' }}
                >
                  Skip
                </button>
              </div>
            </div>
          </div>
        )}

        {/* History toggle */}
        {history.length > 1 && (
          <button
            onClick={() => setHistoryOpen(v => !v)}
            className="flex items-center gap-1 text-xs w-full justify-center py-2"
            style={{ color: 'var(--text-muted)' }}
          >
            {historyOpen ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
            {history.length - 1} previous {history.length === 2 ? 'thought' : 'thoughts'}
          </button>
        )}

        {historyOpen && history.slice(0, -1).reverse().map((item, i) => (
          <div key={i} className="p-3 rounded-lg opacity-60" style={{ background: 'var(--bg-tertiary)' }}>
            <p className="text-xs italic mb-1" style={{ color: 'var(--text-muted)' }}>"{item.thought}"</p>
            <div className="flex items-center gap-1">
              {statusIcon(item.response.response.accuracy_check.status)}
              <span className="text-xs" style={{ color: 'var(--text-muted)' }}>
                → {item.response.response.node_proposal.label}
              </span>
            </div>
          </div>
        ))}

        {/* Empty state */}
        {!lastResponse && !isProcessing && history.length === 0 && (
          <div className="text-center py-12" style={{ color: 'var(--text-muted)' }}>
            <Sparkles size={32} className="mx-auto mb-3 opacity-30" />
            <p className="text-sm font-body">What are you understanding?</p>
            <p className="text-xs mt-1 max-w-[240px] mx-auto">
              Type your rough thought about what you just read. Something like: "I think this means..."
            </p>
          </div>
        )}
      </div>

      {/* Input area */}
      <div className="p-3 border-t flex-shrink-0" style={{ borderColor: 'var(--border)' }}>
        <div className="relative">
          <textarea
            ref={inputRef}
            value={input}
            onChange={e => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="What do you think this means?"
            rows={3}
            className="w-full px-4 py-3 pr-12 rounded-lg border text-sm resize-none focus-ring transition-smooth font-body"
            style={{ background: 'var(--bg-tertiary)', borderColor: 'var(--border)', color: 'var(--text-primary)' }}
            disabled={isProcessing}
          />
          <button
            onClick={handleSubmit}
            disabled={!input.trim() || isProcessing}
            className="absolute right-2 bottom-2 p-2 rounded-lg transition-smooth focus-ring disabled:opacity-30"
            style={{ background: input.trim() ? 'var(--accent)' : 'transparent', color: input.trim() ? '#fff' : 'var(--text-muted)' }}
          >
            <Send size={16} />
          </button>
        </div>
        <p className="text-[10px] mt-1.5 text-center" style={{ color: 'var(--text-muted)' }}>
          ⌘+Enter to submit
        </p>
      </div>
    </div>
  );
}
