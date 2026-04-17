import type {
  Book, BookChunk, BookCoverage, GraphData, KnowledgeNode,
  KnowledgeEdge, LlmProviderConfig, ReadingSession, ThoughtResponse, User
} from '../types';

const API_BASE = '/api';

class ApiClient {
  private token: string | null = null;

  setToken(token: string | null) {
    this.token = token;
    if (token) localStorage.setItem('nodum_token', token);
    else localStorage.removeItem('nodum_token');
  }

  getToken(): string | null {
    if (!this.token) this.token = localStorage.getItem('nodum_token');
    return this.token;
  }

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const headers: Record<string, string> = {
      ...(options.headers as Record<string, string> || {}),
    };
    const token = this.getToken();
    if (token) headers['Authorization'] = `Bearer ${token}`;
    if (!(options.body instanceof FormData)) {
      headers['Content-Type'] = 'application/json';
    }

    const response = await fetch(`${API_BASE}${path}`, { ...options, headers });

    if (response.status === 401) {
      this.setToken(null);
      window.location.href = '/login';
      throw new Error('Unauthorized');
    }

    if (!response.ok) {
      const err = await response.json().catch(() => ({ error: 'Request failed' }));
      throw new Error(err.error || `HTTP ${response.status}`);
    }

    if (response.status === 204) return undefined as T;
    return response.json();
  }

  // ─── Auth ─────────────────────────────────────────────
  async register(email: string, password: string, display_name: string) {
    const data = await this.request<{ token: string; user: User }>('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ email, password, display_name }),
    });
    this.setToken(data.token);
    return data;
  }

  async login(email: string, password: string) {
    const data = await this.request<{ token: string; user: User }>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    });
    this.setToken(data.token);
    return data;
  }

  logout() {
    this.setToken(null);
  }

  // ─── Books ────────────────────────────────────────────
  async getBooks(): Promise<Book[]> {
    return this.request('/books');
  }

  async getBook(id: string): Promise<Book> {
    return this.request(`/books/${id}`);
  }

  async uploadBook(file: File): Promise<{ book: Book; chunks_created: number }> {
    const formData = new FormData();
    formData.append('file', file);
    return this.request('/books', { method: 'POST', body: formData });
  }

  async deleteBook(id: string): Promise<void> {
    return this.request(`/books/${id}`, { method: 'DELETE' });
  }

  async getBookChunks(bookId: string, offset = 0, limit = 20): Promise<BookChunk[]> {
    return this.request(`/books/${bookId}/chunks?offset=${offset}&limit=${limit}`);
  }

  async getBookCoverage(bookId: string): Promise<BookCoverage> {
    return this.request(`/books/${bookId}/coverage`);
  }

  // ─── Thoughts (Core Loop) ────────────────────────────
  async submitThought(
    bookId: string,
    rawText: string,
    chunkId?: string,
    sessionId?: string,
    pageContext?: string
  ): Promise<ThoughtResponse> {
    return this.request('/thoughts', {
      method: 'POST',
      body: JSON.stringify({
        book_id: bookId,
        raw_text: rawText,
        chunk_id: chunkId,
        session_id: sessionId,
        page_context: pageContext,
      }),
    });
  }

  // ─── Nodes ────────────────────────────────────────────
  async getNodes(bookId?: string): Promise<KnowledgeNode[]> {
    const params = bookId ? `?book_id=${bookId}` : '';
    return this.request(`/nodes${params}`);
  }

  async confirmNode(
    thoughtId: string,
    label: string,
    description: string,
    nodeType: string,
    connections: Array<{ target_node_id: string; relation_type: string }>
  ): Promise<KnowledgeNode> {
    return this.request('/nodes', {
      method: 'POST',
      body: JSON.stringify({
        thought_id: thoughtId,
        label,
        description,
        node_type: nodeType,
        connections,
      }),
    });
  }

  async updateNode(id: string, updates: Partial<KnowledgeNode>): Promise<KnowledgeNode> {
    return this.request(`/nodes/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    });
  }

  async deleteNode(id: string): Promise<void> {
    return this.request(`/nodes/${id}`, { method: 'DELETE' });
  }

  async searchNodes(query: string, bookId?: string): Promise<KnowledgeNode[]> {
    return this.request('/nodes/search', {
      method: 'POST',
      body: JSON.stringify({ query, book_id: bookId, limit: 20 }),
    });
  }

  // ─── Edges ────────────────────────────────────────────
  async createEdge(
    sourceId: string, targetId: string, relationType: string, description?: string
  ): Promise<KnowledgeEdge> {
    return this.request('/edges', {
      method: 'POST',
      body: JSON.stringify({
        source_node_id: sourceId,
        target_node_id: targetId,
        relation_type: relationType,
        description,
      }),
    });
  }

  async deleteEdge(id: string): Promise<void> {
    return this.request(`/edges/${id}`, { method: 'DELETE' });
  }

  // ─── Graph ────────────────────────────────────────────
  async getGraph(bookId?: string): Promise<GraphData> {
    const params = bookId ? `?book_id=${bookId}` : '';
    return this.request(`/graph${params}`);
  }

  async getWeakNodes(): Promise<KnowledgeNode[]> {
    return this.request('/graph/weak');
  }

  // ─── Sessions ─────────────────────────────────────────
  async startSession(bookId: string): Promise<ReadingSession> {
    return this.request('/sessions', {
      method: 'POST',
      body: JSON.stringify({ book_id: bookId }),
    });
  }

  async endSession(sessionId: string): Promise<ReadingSession> {
    return this.request(`/sessions/${sessionId}/end`, { method: 'POST' });
  }

  // ─── LLM Providers ───────────────────────────────────
  async getProviders(): Promise<LlmProviderConfig[]> {
    return this.request('/providers');
  }

  async configureProvider(config: {
    provider: string;
    api_key?: string;
    model_name: string;
    base_url?: string;
    is_default: boolean;
  }): Promise<LlmProviderConfig> {
    return this.request('/providers', {
      method: 'POST',
      body: JSON.stringify(config),
    });
  }

  // ─── Health ───────────────────────────────────────────
  async healthCheck() {
    return this.request<{ status: string; database: boolean }>('/health');
  }
}

export const api = new ApiClient();
