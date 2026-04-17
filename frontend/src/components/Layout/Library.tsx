import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Upload, BookOpen, Trash2, Search, Plus } from 'lucide-react';
import { useLibraryStore } from '../../stores';
import { clsx } from 'clsx';

export function Library() {
  const { books, loading, error, fetchBooks, uploadBook, deleteBook } = useLibraryStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [uploading, setUploading] = useState(false);
  const [dragOver, setDragOver] = useState(false);
  const fileInput = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();

  useEffect(() => { fetchBooks(); }, []);

  const handleUpload = async (file: File) => {
    setUploading(true);
    try {
      const book = await uploadBook(file);
      navigate(`/read/${book.id}`);
    } catch { /* error is in store */ }
    setUploading(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    const file = e.dataTransfer.files[0];
    if (file && (file.name.endsWith('.pdf') || file.name.endsWith('.epub'))) {
      handleUpload(file);
    }
  };

  const filtered = books.filter(b =>
    b.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
    (b.author || '').toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="p-8 max-w-6xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="font-display text-3xl font-bold" style={{ color: 'var(--text-primary)' }}>
            Your Library
          </h1>
          <p className="mt-1 text-sm" style={{ color: 'var(--text-muted)' }}>
            {books.length} {books.length === 1 ? 'book' : 'books'} in your knowledge base
          </p>
        </div>

        <button
          onClick={() => fileInput.current?.click()}
          className="flex items-center gap-2 px-5 py-2.5 rounded-lg font-medium text-sm text-white transition-smooth focus-ring"
          style={{ background: 'var(--accent)' }}
        >
          <Plus size={18} />
          Add Book
        </button>
        <input
          ref={fileInput}
          type="file"
          accept=".pdf,.epub"
          className="hidden"
          onChange={e => { const f = e.target.files?.[0]; if (f) handleUpload(f); }}
        />
      </div>

      {/* Search */}
      <div className="relative mb-6">
        <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2" style={{ color: 'var(--text-muted)' }} />
        <input
          type="text"
          placeholder="Search your books..."
          value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)}
          className="w-full pl-10 pr-4 py-2.5 rounded-lg border text-sm focus-ring transition-smooth"
          style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)', color: 'var(--text-primary)' }}
        />
      </div>

      {/* Drop zone */}
      <div
        onDragOver={e => { e.preventDefault(); setDragOver(true); }}
        onDragLeave={() => setDragOver(false)}
        onDrop={handleDrop}
        className={clsx(
          'border-2 border-dashed rounded-xl p-8 text-center mb-8 transition-smooth',
          dragOver ? 'border-[var(--accent)] bg-[var(--accent)]/5' : '',
          uploading && 'opacity-50 pointer-events-none'
        )}
        style={{ borderColor: dragOver ? 'var(--accent)' : 'var(--border)' }}
      >
        <Upload size={32} className="mx-auto mb-3" style={{ color: 'var(--text-muted)' }} />
        <p className="text-sm font-medium" style={{ color: 'var(--text-secondary)' }}>
          {uploading ? 'Processing document...' : 'Drop a PDF or EPUB here'}
        </p>
        <p className="text-xs mt-1" style={{ color: 'var(--text-muted)' }}>
          or click "Add Book" to browse
        </p>
      </div>

      {error && (
        <div className="mb-6 px-4 py-3 rounded-lg text-sm" style={{ background: 'rgba(160,82,45,0.1)', color: '#a0522d' }}>
          {error}
        </div>
      )}

      {/* Book grid */}
      {loading && books.length === 0 ? (
        <div className="text-center py-20" style={{ color: 'var(--text-muted)' }}>
          <div className="thinking-indicator inline-block">Loading your library...</div>
        </div>
      ) : filtered.length === 0 ? (
        <div className="text-center py-20" style={{ color: 'var(--text-muted)' }}>
          <BookOpen size={48} className="mx-auto mb-4 opacity-30" />
          <p className="font-display text-lg">{searchQuery ? 'No matches found' : 'Your library is empty'}</p>
          <p className="text-sm mt-1">Upload your first book to start building your knowledge graph.</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {filtered.map(book => (
            <button
              key={book.id}
              onClick={() => navigate(`/read/${book.id}`)}
              className="group text-left p-5 rounded-xl border transition-smooth focus-ring hover:border-[var(--accent)]"
              style={{ background: 'var(--bg-secondary)', borderColor: 'var(--border)' }}
            >
              <div className="flex items-start gap-3">
                <div
                  className="w-3 h-12 rounded-full flex-shrink-0 mt-0.5"
                  style={{ background: book.color }}
                />
                <div className="flex-1 min-w-0">
                  <h3 className="font-display font-semibold text-base leading-snug truncate" style={{ color: 'var(--text-primary)' }}>
                    {book.title}
                  </h3>
                  {book.author && (
                    <p className="text-xs mt-0.5 truncate" style={{ color: 'var(--text-muted)' }}>
                      {book.author}
                    </p>
                  )}
                  <div className="flex items-center gap-3 mt-3 text-xs" style={{ color: 'var(--text-muted)' }}>
                    <span className="uppercase tracking-wider font-mono">{book.file_type}</span>
                    {book.total_pages && <span>{book.total_pages} pages</span>}
                  </div>
                </div>
                <button
                  onClick={(e) => { e.stopPropagation(); deleteBook(book.id); }}
                  className="opacity-0 group-hover:opacity-100 p-1.5 rounded transition-smooth"
                  style={{ color: 'var(--text-muted)' }}
                  title="Remove book"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
