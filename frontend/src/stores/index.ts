import { create } from 'zustand';
import { api } from '../api/client';
import type {
  User, Book, BookChunk, GraphData, GraphNode,
  KnowledgeNode, ThoughtResponse, ReadingSession, BookCoverage
} from '../types';

// ─── Auth Store ─────────────────────────────────────────
interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  loading: boolean;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string, name: string) => Promise<void>;
  logout: () => void;
  checkAuth: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  isAuthenticated: !!localStorage.getItem('nodum_token'),
  loading: false,

  login: async (email, password) => {
    set({ loading: true });
    try {
      const { user } = await api.login(email, password);
      set({ user, isAuthenticated: true, loading: false });
    } catch (e) {
      set({ loading: false });
      throw e;
    }
  },

  register: async (email, password, name) => {
    set({ loading: true });
    try {
      const { user } = await api.register(email, password, name);
      set({ user, isAuthenticated: true, loading: false });
    } catch (e) {
      set({ loading: false });
      throw e;
    }
  },

  logout: () => {
    api.logout();
    set({ user: null, isAuthenticated: false });
  },

  checkAuth: () => {
    const token = api.getToken();
    set({ isAuthenticated: !!token });
  },
}));

// ─── Library Store (Books) ──────────────────────────────
interface LibraryState {
  books: Book[];
  loading: boolean;
  error: string | null;
  fetchBooks: () => Promise<void>;
  uploadBook: (file: File) => Promise<Book>;
  deleteBook: (id: string) => Promise<void>;
}

export const useLibraryStore = create<LibraryState>((set, get) => ({
  books: [],
  loading: false,
  error: null,

  fetchBooks: async () => {
    set({ loading: true, error: null });
    try {
      const books = await api.getBooks();
      set({ books, loading: false });
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  uploadBook: async (file) => {
    set({ loading: true, error: null });
    try {
      const { book } = await api.uploadBook(file);
      set({ books: [book, ...get().books], loading: false });
      return book;
    } catch (e: any) {
      set({ error: e.message, loading: false });
      throw e;
    }
  },

  deleteBook: async (id) => {
    await api.deleteBook(id);
    set({ books: get().books.filter(b => b.id !== id) });
  },
}));

// ─── Reader Store ───────────────────────────────────────
interface ReaderState {
  currentBook: Book | null;
  chunks: BookChunk[];
  currentChunkIndex: number;
  session: ReadingSession | null;
  coverage: BookCoverage | null;
  loading: boolean;

  openBook: (bookId: string) => Promise<void>;
  loadChunks: (offset?: number) => Promise<void>;
  setChunkIndex: (index: number) => void;
  nextChunk: () => void;
  prevChunk: () => void;
  startSession: () => Promise<void>;
  endSession: () => Promise<ReadingSession | null>;
  fetchCoverage: () => Promise<void>;
}

export const useReaderStore = create<ReaderState>((set, get) => ({
  currentBook: null,
  chunks: [],
  currentChunkIndex: 0,
  session: null,
  coverage: null,
  loading: false,

  openBook: async (bookId) => {
    set({ loading: true });
    const book = await api.getBook(bookId);
    const chunks = await api.getBookChunks(bookId, 0, 50);
    set({ currentBook: book, chunks, currentChunkIndex: 0, loading: false });
  },

  loadChunks: async (offset = 0) => {
    const { currentBook } = get();
    if (!currentBook) return;
    const newChunks = await api.getBookChunks(currentBook.id, offset, 50);
    set(state => ({
      chunks: offset === 0 ? newChunks : [...state.chunks, ...newChunks],
    }));
  },

  setChunkIndex: (index) => set({ currentChunkIndex: index }),
  nextChunk: () => set(s => ({
    currentChunkIndex: Math.min(s.currentChunkIndex + 1, s.chunks.length - 1)
  })),
  prevChunk: () => set(s => ({
    currentChunkIndex: Math.max(s.currentChunkIndex - 1, 0)
  })),

  startSession: async () => {
    const { currentBook } = get();
    if (!currentBook) return;
    const session = await api.startSession(currentBook.id);
    set({ session });
  },

  endSession: async () => {
    const { session } = get();
    if (!session) return null;
    const ended = await api.endSession(session.id);
    set({ session: null });
    return ended;
  },

  fetchCoverage: async () => {
    const { currentBook } = get();
    if (!currentBook) return;
    const coverage = await api.getBookCoverage(currentBook.id);
    set({ coverage });
  },
}));

// ─── Thinking Dock Store ────────────────────────────────
interface DockState {
  isProcessing: boolean;
  lastResponse: ThoughtResponse | null;
  sessionNodes: GraphNode[];
  history: Array<{ thought: string; response: ThoughtResponse; timestamp: number }>;
  error: string | null;

  submitThought: (text: string) => Promise<ThoughtResponse>;
  confirmNode: (thoughtId: string, label: string, description: string, nodeType: string, connections: any[]) => Promise<void>;
  clearError: () => void;
}

export const useDockStore = create<DockState>((set, get) => ({
  isProcessing: false,
  lastResponse: null,
  sessionNodes: [],
  history: [],
  error: null,

  submitThought: async (text) => {
    const reader = useReaderStore.getState();
    if (!reader.currentBook) throw new Error('No book open');

    const chunk = reader.chunks[reader.currentChunkIndex];
    set({ isProcessing: true, error: null });

    try {
      const response = await api.submitThought(
        reader.currentBook.id,
        text,
        chunk?.id,
        reader.session?.id,
        chunk?.content
      );
      set(state => ({
        isProcessing: false,
        lastResponse: response,
        history: [...state.history, { thought: text, response, timestamp: Date.now() }],
      }));
      return response;
    } catch (e: any) {
      set({ isProcessing: false, error: e.message });
      throw e;
    }
  },

  confirmNode: async (thoughtId, label, description, nodeType, connections) => {
    const node = await api.confirmNode(thoughtId, label, description, nodeType, connections);
    // Refresh graph
    useGraphStore.getState().fetchGraph();
  },

  clearError: () => set({ error: null }),
}));

// ─── Graph Store ────────────────────────────────────────
interface GraphState {
  graphData: GraphData | null;
  selectedNode: GraphNode | null;
  filterBookId: string | null;
  loading: boolean;

  fetchGraph: (bookId?: string) => Promise<void>;
  selectNode: (node: GraphNode | null) => void;
  setFilter: (bookId: string | null) => void;
  createEdge: (sourceId: string, targetId: string, relationType: string) => Promise<void>;
  deleteNode: (id: string) => Promise<void>;
}

export const useGraphStore = create<GraphState>((set, get) => ({
  graphData: null,
  selectedNode: null,
  filterBookId: null,
  loading: false,

  fetchGraph: async (bookId) => {
    set({ loading: true });
    try {
      const data = await api.getGraph(bookId || get().filterBookId || undefined);
      set({ graphData: data, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  selectNode: (node) => set({ selectedNode: node }),
  setFilter: (bookId) => {
    set({ filterBookId: bookId });
    get().fetchGraph(bookId || undefined);
  },

  createEdge: async (sourceId, targetId, relationType) => {
    await api.createEdge(sourceId, targetId, relationType);
    get().fetchGraph();
  },

  deleteNode: async (id) => {
    await api.deleteNode(id);
    get().fetchGraph();
  },
}));

// ─── UI Store ───────────────────────────────────────────
interface UIState {
  sidebarOpen: boolean;
  graphFullscreen: boolean;
  activeView: 'library' | 'reader' | 'graph' | 'settings';
  theme: 'dark' | 'light';

  toggleSidebar: () => void;
  toggleGraphFullscreen: () => void;
  setView: (view: UIState['activeView']) => void;
  setTheme: (theme: 'dark' | 'light') => void;
}

export const useUIStore = create<UIState>((set) => ({
  sidebarOpen: true,
  graphFullscreen: false,
  activeView: 'library',
  theme: (localStorage.getItem('nodum_theme') as 'dark' | 'light') || 'dark',

  toggleSidebar: () => set(s => ({ sidebarOpen: !s.sidebarOpen })),
  toggleGraphFullscreen: () => set(s => ({ graphFullscreen: !s.graphFullscreen })),
  setView: (view) => set({ activeView: view }),
  setTheme: (theme) => {
    localStorage.setItem('nodum_theme', theme);
    set({ theme });
  },
}));
