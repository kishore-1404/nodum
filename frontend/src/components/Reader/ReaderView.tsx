import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { ArrowLeft, ChevronLeft, ChevronRight, Network, Maximize2, Minimize2 } from 'lucide-react';
import { useReaderStore, useGraphStore, useUIStore } from '../../stores';
import { ThinkingDock } from '../ThinkingDock/ThinkingDock';
import { MiniGraph } from '../Graph/MiniGraph';
import { clsx } from 'clsx';

export function ReaderView() {
  const { bookId } = useParams<{ bookId: string }>();
  const navigate = useNavigate();
  const {
    currentBook, chunks, currentChunkIndex, session, coverage, loading,
    openBook, setChunkIndex, nextChunk, prevChunk, startSession, endSession, fetchCoverage,
  } = useReaderStore();
  const { graphData, fetchGraph } = useGraphStore();
  const [dockExpanded, setDockExpanded] = useState(true);
  const [showMiniGraph, setShowMiniGraph] = useState(false);

  useEffect(() => {
    if (bookId) {
      openBook(bookId).then(() => {
        startSession();
        fetchGraph(bookId);
        fetchCoverage();
      });
    }
    return () => { endSession(); };
  }, [bookId]);

  // Keyboard navigation
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if (e.key === 'ArrowRight' || e.key === 'j') nextChunk();
      if (e.key === 'ArrowLeft' || e.key === 'k') prevChunk();
      if (e.key === 'g') setShowMiniGraph(v => !v);
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  const currentChunk = chunks[currentChunkIndex];
  const progress = chunks.length > 0 ? ((currentChunkIndex + 1) / chunks.length) * 100 : 0;

  if (loading || !currentBook) {
    return (
      <div className="h-screen flex items-center justify-center" style={{ background: 'var(--bg-primary)', color: 'var(--text-muted)' }}>
        <div className="thinking-indicator text-lg font-body">Opening your book...</div>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col" style={{ background: 'var(--bg-primary)' }}>
      {/* Top bar */}
      <header
        className="flex items-center justify-between px-4 h-12 border-b flex-shrink-0"
        style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)' }}
      >
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/')} className="focus-ring rounded p-1" style={{ color: 'var(--text-muted)' }}>
            <ArrowLeft size={18} />
          </button>
          <div className="w-2 h-2 rounded-full" style={{ background: currentBook.color }} />
          <span className="font-display font-semibold text-sm truncate max-w-[300px]" style={{ color: 'var(--text-primary)' }}>
            {currentBook.title}
          </span>
        </div>

        <div className="flex items-center gap-3">
          {coverage && (
            <span className="text-xs font-mono" style={{ color: 'var(--text-muted)' }}>
              {coverage.total_nodes} nodes · {coverage.coverage_percent.toFixed(0)}% covered
            </span>
          )}
          <button
            onClick={() => setShowMiniGraph(v => !v)}
            className={clsx('p-1.5 rounded transition-smooth focus-ring', showMiniGraph && 'bg-[var(--accent)]/20')}
            style={{ color: showMiniGraph ? 'var(--accent)' : 'var(--text-muted)' }}
            title="Toggle mini graph (G)"
          >
            <Network size={16} />
          </button>
        </div>
      </header>

      {/* Progress bar */}
      <div className="h-0.5 flex-shrink-0" style={{ background: 'var(--bg-tertiary)' }}>
        <div className="h-full transition-all duration-300" style={{ width: `${progress}%`, background: currentBook.color }} />
      </div>

      {/* Main reading area */}
      <div className="flex-1 flex overflow-hidden">
        {/* Reader panel */}
        <div className={clsx('flex-1 overflow-auto transition-all duration-300', dockExpanded ? '' : '')}>
          <div className="max-w-2xl mx-auto px-8 py-12">
            {/* Chapter indicator */}
            {currentChunk?.chapter_title && (
              <p className="text-xs font-mono uppercase tracking-widest mb-6" style={{ color: 'var(--text-muted)' }}>
                {currentChunk.chapter_title}
              </p>
            )}

            {/* Content */}
            <div className="reader-content">
              {currentChunk?.content.split('\n\n').map((para, i) => (
                <p key={i}>{para}</p>
              ))}
            </div>

            {/* Page navigation */}
            <div className="flex items-center justify-between mt-16 pb-8">
              <button
                onClick={prevChunk}
                disabled={currentChunkIndex === 0}
                className="flex items-center gap-1 px-4 py-2 rounded-lg text-sm font-medium transition-smooth focus-ring disabled:opacity-30"
                style={{ color: 'var(--text-secondary)', background: 'var(--bg-secondary)' }}
              >
                <ChevronLeft size={16} /> Previous
              </button>

              <span className="text-xs font-mono" style={{ color: 'var(--text-muted)' }}>
                {currentChunkIndex + 1} / {chunks.length}
              </span>

              <button
                onClick={nextChunk}
                disabled={currentChunkIndex === chunks.length - 1}
                className="flex items-center gap-1 px-4 py-2 rounded-lg text-sm font-medium transition-smooth focus-ring disabled:opacity-30"
                style={{ color: 'var(--text-secondary)', background: 'var(--bg-secondary)' }}
              >
                Next <ChevronRight size={16} />
              </button>
            </div>
          </div>
        </div>

        {/* Thinking dock (right panel) */}
        <div
          className={clsx(
            'border-l flex-shrink-0 flex flex-col transition-all duration-300 overflow-hidden',
            dockExpanded ? 'w-[400px]' : 'w-0'
          )}
          style={{ borderColor: 'var(--border)', background: 'var(--bg-secondary)' }}
        >
          {dockExpanded && <ThinkingDock />}
        </div>

        {/* Mini graph overlay */}
        {showMiniGraph && graphData && (
          <div
            className="absolute bottom-20 right-[420px] w-80 h-64 rounded-xl border shadow-2xl overflow-hidden z-10"
            style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)' }}
          >
            <div className="flex items-center justify-between px-3 py-2 border-b" style={{ borderColor: 'var(--border)' }}>
              <span className="text-xs font-medium" style={{ color: 'var(--text-muted)' }}>Session Graph</span>
              <button onClick={() => navigate('/graph')} className="text-xs underline" style={{ color: 'var(--accent)' }}>
                Full view
              </button>
            </div>
            <MiniGraph data={graphData} bookColor={currentBook.color} />
          </div>
        )}
      </div>

      {/* Dock toggle */}
      <button
        onClick={() => setDockExpanded(v => !v)}
        className="fixed bottom-4 right-4 z-20 p-2.5 rounded-full shadow-lg transition-smooth focus-ring"
        style={{ background: 'var(--accent)', color: '#fff' }}
        title={dockExpanded ? 'Collapse dock' : 'Expand dock'}
      >
        {dockExpanded ? <Minimize2 size={18} /> : <Maximize2 size={18} />}
      </button>
    </div>
  );
}
