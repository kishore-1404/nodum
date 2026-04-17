import { useEffect, useState, useRef } from 'react';
import { Search, Filter, X, BookOpen, Trash2, Link2 } from 'lucide-react';
import { useGraphStore, useLibraryStore } from '../../stores';
import { ForceGraph } from './ForceGraph';
import { api } from '../../api/client';
import type { GraphNode } from '../../types';
import { clsx } from 'clsx';

export function GraphFullView() {
  const { graphData, selectedNode, filterBookId, loading, fetchGraph, selectNode, setFilter, deleteNode } = useGraphStore();
  const { books, fetchBooks } = useLibraryStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<GraphNode[]>([]);
  const [dimensions, setDimensions] = useState({ width: 800, height: 600 });
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    fetchGraph();
    fetchBooks();
  }, []);

  useEffect(() => {
    const updateDimensions = () => {
      if (containerRef.current) {
        const rect = containerRef.current.getBoundingClientRect();
        setDimensions({ width: rect.width, height: rect.height });
      }
    };
    updateDimensions();
    window.addEventListener('resize', updateDimensions);
    return () => window.removeEventListener('resize', updateDimensions);
  }, []);

  const handleSearch = async () => {
    if (!searchQuery.trim()) { setSearchResults([]); return; }
    try {
      const results = await api.searchNodes(searchQuery);
      setSearchResults(results.map(n => ({
        ...n,
        book_color: null,
        book_title: null,
        connection_count: 0,
      })));
    } catch { /* ignore */ }
  };

  useEffect(() => {
    const timer = setTimeout(handleSearch, 300);
    return () => clearTimeout(timer);
  }, [searchQuery]);

  const stats = graphData ? {
    nodes: graphData.nodes.length,
    edges: graphData.edges.length,
    avgConfidence: graphData.nodes.length > 0
      ? (graphData.nodes.reduce((s, n) => s + n.confidence_score, 0) / graphData.nodes.length * 100).toFixed(0)
      : 0,
    weakNodes: graphData.nodes.filter(n => n.connection_count <= 1).length,
  } : null;

  return (
    <div className="h-full flex flex-col">
      {/* Header bar */}
      <div className="flex items-center justify-between px-6 py-4 border-b" style={{ borderColor: 'var(--border)' }}>
        <div>
          <h1 className="font-display text-2xl font-bold" style={{ color: 'var(--text-primary)' }}>
            Knowledge Graph
          </h1>
          {stats && (
            <p className="text-xs mt-1 font-mono" style={{ color: 'var(--text-muted)' }}>
              {stats.nodes} nodes · {stats.edges} edges · {stats.avgConfidence}% avg confidence · {stats.weakNodes} weak
            </p>
          )}
        </div>

        <div className="flex items-center gap-3">
          {/* Book filter */}
          <div className="flex items-center gap-1">
            <Filter size={14} style={{ color: 'var(--text-muted)' }} />
            <select
              value={filterBookId || ''}
              onChange={e => setFilter(e.target.value || null)}
              className="text-xs px-2 py-1.5 rounded border focus-ring"
              style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)', color: 'var(--text-secondary)' }}
            >
              <option value="">All books</option>
              {books.map(b => (
                <option key={b.id} value={b.id}>{b.title}</option>
              ))}
            </select>
          </div>

          {/* Search */}
          <div className="relative">
            <Search size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2" style={{ color: 'var(--text-muted)' }} />
            <input
              type="text"
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              placeholder="Search nodes..."
              className="pl-8 pr-3 py-1.5 w-52 rounded border text-xs focus-ring"
              style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)', color: 'var(--text-primary)' }}
            />
            {searchResults.length > 0 && (
              <div
                className="absolute top-full left-0 right-0 mt-1 rounded-lg border shadow-xl z-20 max-h-48 overflow-auto"
                style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)' }}
              >
                {searchResults.map(n => (
                  <button
                    key={n.id}
                    onClick={() => { selectNode(n); setSearchQuery(''); setSearchResults([]); }}
                    className="w-full text-left px-3 py-2 text-xs hover:bg-[var(--bg-tertiary)] transition-smooth"
                  >
                    <span style={{ color: 'var(--text-primary)' }}>{n.label}</span>
                    <span className="ml-2 font-mono" style={{ color: 'var(--text-muted)' }}>{n.node_type}</span>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Graph + detail panel */}
      <div className="flex-1 flex overflow-hidden">
        {/* Graph */}
        <div ref={containerRef} className="flex-1 relative">
          {loading ? (
            <div className="absolute inset-0 flex items-center justify-center">
              <div className="thinking-indicator" style={{ color: 'var(--text-muted)' }}>Loading graph...</div>
            </div>
          ) : graphData && graphData.nodes.length > 0 ? (
            <ForceGraph
              data={graphData}
              width={dimensions.width}
              height={dimensions.height}
              onNodeClick={selectNode}
              selectedNodeId={selectedNode?.id}
            />
          ) : (
            <div className="absolute inset-0 flex flex-col items-center justify-center" style={{ color: 'var(--text-muted)' }}>
              <BookOpen size={48} className="opacity-20 mb-4" />
              <p className="font-display text-lg">Your graph is empty</p>
              <p className="text-sm mt-1">Start reading and dropping thoughts to grow your graph.</p>
            </div>
          )}
        </div>

        {/* Detail panel */}
        {selectedNode && (
          <div
            className="w-80 border-l overflow-auto flex-shrink-0 animate-fade-in"
            style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)' }}
          >
            <div className="p-4">
              <div className="flex items-start justify-between mb-3">
                <div className="flex-1">
                  <span className="text-[10px] font-mono uppercase tracking-wider px-1.5 py-0.5 rounded"
                    style={{ background: 'var(--bg-surface)', color: 'var(--text-muted)' }}>
                    {selectedNode.node_type}
                  </span>
                  <h3 className="font-display font-semibold text-base mt-2" style={{ color: 'var(--text-primary)' }}>
                    {selectedNode.label}
                  </h3>
                </div>
                <button onClick={() => selectNode(null)} className="p-1" style={{ color: 'var(--text-muted)' }}>
                  <X size={16} />
                </button>
              </div>

              <p className="text-sm leading-relaxed mb-4" style={{ color: 'var(--text-secondary)' }}>
                {selectedNode.description}
              </p>

              {/* Metadata */}
              <div className="space-y-2 text-xs pb-4 border-b" style={{ borderColor: 'var(--border)', color: 'var(--text-muted)' }}>
                <div className="flex justify-between">
                  <span>Confidence</span>
                  <div className="flex items-center gap-2">
                    <div className="w-20 h-1.5 rounded-full overflow-hidden" style={{ background: 'var(--bg-surface)' }}>
                      <div className="h-full rounded-full" style={{
                        width: `${selectedNode.confidence_score * 100}%`,
                        background: selectedNode.confidence_score > 0.7 ? '#4a7c59' :
                          selectedNode.confidence_score > 0.4 ? '#b8860b' : '#a0522d'
                      }} />
                    </div>
                    <span>{(selectedNode.confidence_score * 100).toFixed(0)}%</span>
                  </div>
                </div>
                <div className="flex justify-between">
                  <span>Connections</span>
                  <span>{selectedNode.connection_count}</span>
                </div>
                {selectedNode.book_title && (
                  <div className="flex justify-between">
                    <span>From</span>
                    <span className="flex items-center gap-1">
                      <span className="w-2 h-2 rounded-full inline-block" style={{ background: selectedNode.book_color || '#6366f1' }} />
                      {selectedNode.book_title}
                    </span>
                  </div>
                )}
              </div>

              {/* Connected edges */}
              {graphData && (
                <div className="mt-4">
                  <h4 className="text-xs font-semibold mb-2" style={{ color: 'var(--text-muted)' }}>
                    <Link2 size={12} className="inline mr-1" /> Connections
                  </h4>
                  {graphData.edges
                    .filter(e => e.source === selectedNode.id || e.target === selectedNode.id)
                    .map(edge => {
                      const otherId = edge.source === selectedNode.id ? edge.target : edge.source;
                      const otherNode = graphData.nodes.find(n => n.id === otherId);
                      return (
                        <button
                          key={edge.id}
                          onClick={() => otherNode && selectNode(otherNode)}
                          className="w-full text-left flex items-center gap-2 px-2 py-1.5 rounded text-xs hover:bg-[var(--bg-tertiary)] transition-smooth"
                        >
                          <span className="font-mono px-1 py-0.5 rounded text-[10px]"
                            style={{ background: 'var(--bg-surface)', color: 'var(--text-muted)' }}>
                            {edge.relation_type.replace(/_/g, ' ')}
                          </span>
                          <span style={{ color: 'var(--text-primary)' }}>
                            {otherNode?.label || otherId.slice(0, 8)}
                          </span>
                        </button>
                      );
                    })}
                </div>
              )}

              {/* Actions */}
              <div className="mt-6 pt-4 border-t" style={{ borderColor: 'var(--border)' }}>
                <button
                  onClick={() => { deleteNode(selectedNode.id); selectNode(null); }}
                  className="flex items-center gap-1.5 text-xs px-3 py-1.5 rounded transition-smooth"
                  style={{ color: '#a0522d' }}
                >
                  <Trash2 size={12} /> Delete node
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
