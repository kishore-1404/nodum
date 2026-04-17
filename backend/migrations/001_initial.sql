-- Nodum: Complete Database Schema
-- Requires: PostgreSQL 15+ with pgvector extension

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "vector";

-- ============================================================
-- USERS
-- ============================================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    preferences JSONB NOT NULL DEFAULT '{
        "note_style": "inline",
        "theme": "dark",
        "default_llm_provider": "claude"
    }'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- LLM PROVIDER CONFIGURATION
-- ============================================================
CREATE TABLE llm_providers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL CHECK (provider IN ('claude', 'openai', 'gemini', 'ollama', 'custom')),
    api_key_encrypted TEXT, -- NULL for ollama/local
    model_name VARCHAR(100) NOT NULL,
    base_url TEXT, -- For custom/ollama endpoints
    is_default BOOLEAN NOT NULL DEFAULT false,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, provider, model_name)
);

-- ============================================================
-- BOOKS / DOCUMENTS
-- ============================================================
CREATE TABLE books (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    author VARCHAR(500),
    file_type VARCHAR(20) NOT NULL CHECK (file_type IN ('pdf', 'epub', 'url', 'markdown')),
    file_path TEXT, -- S3 key or local path
    file_hash VARCHAR(64), -- SHA256 for dedup
    cover_image_url TEXT,
    total_pages INTEGER,
    total_chapters INTEGER,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    color VARCHAR(7) NOT NULL DEFAULT '#6366f1', -- Graph node color for this book
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_books_user ON books(user_id);

-- ============================================================
-- BOOK CHUNKS (parsed content segments)
-- ============================================================
CREATE TABLE book_chunks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    page_number INTEGER,
    chapter_number INTEGER,
    chapter_title VARCHAR(500),
    content TEXT NOT NULL,
    content_embedding vector(1536), -- for semantic search within the book
    token_count INTEGER NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chunks_book ON book_chunks(book_id, chunk_index);
CREATE INDEX idx_chunks_embedding ON book_chunks USING ivfflat (content_embedding vector_cosine_ops) WITH (lists = 100);

-- ============================================================
-- KNOWLEDGE GRAPH NODES
-- ============================================================
CREATE TABLE nodes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id UUID REFERENCES books(id) ON DELETE SET NULL,
    chunk_id UUID REFERENCES book_chunks(id) ON DELETE SET NULL,
    label VARCHAR(300) NOT NULL,
    description TEXT NOT NULL,
    user_note TEXT, -- The user's original rough thought
    accuracy_note TEXT, -- LLM's accuracy assessment
    node_type VARCHAR(50) NOT NULL DEFAULT 'concept' CHECK (
        node_type IN ('concept', 'fact', 'principle', 'example', 'question', 'prior_knowledge', 'insight')
    ),
    confidence_score REAL NOT NULL DEFAULT 0.5 CHECK (confidence_score BETWEEN 0.0 AND 1.0),
    times_confirmed INTEGER NOT NULL DEFAULT 0,
    times_corrected INTEGER NOT NULL DEFAULT 0,
    embedding vector(1536),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_reviewed_at TIMESTAMPTZ
);

CREATE INDEX idx_nodes_user ON nodes(user_id);
CREATE INDEX idx_nodes_book ON nodes(book_id);
CREATE INDEX idx_nodes_embedding ON nodes USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX idx_nodes_label_trgm ON nodes USING gin (label gin_trgm_ops);

-- Enable trigram for fuzzy text search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ============================================================
-- KNOWLEDGE GRAPH EDGES
-- ============================================================
CREATE TABLE edges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    source_node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    target_node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    relation_type VARCHAR(50) NOT NULL CHECK (
        relation_type IN (
            'builds_on', 'contradicts', 'is_example_of',
            'requires', 'extends', 'is_part_of',
            'analogous_to', 'causes', 'derived_from',
            'related_to', 'supports'
        )
    ),
    weight REAL NOT NULL DEFAULT 1.0,
    llm_generated BOOLEAN NOT NULL DEFAULT true,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(source_node_id, target_node_id, relation_type)
);

CREATE INDEX idx_edges_source ON edges(source_node_id);
CREATE INDEX idx_edges_target ON edges(target_node_id);
CREATE INDEX idx_edges_user ON edges(user_id);

-- ============================================================
-- READING SESSIONS
-- ============================================================
CREATE TABLE reading_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    last_page INTEGER,
    last_chunk_id UUID REFERENCES book_chunks(id),
    nodes_created INTEGER NOT NULL DEFAULT 0,
    nodes_reinforced INTEGER NOT NULL DEFAULT 0,
    thoughts_submitted INTEGER NOT NULL DEFAULT 0,
    duration_seconds INTEGER
);

CREATE INDEX idx_sessions_user_book ON reading_sessions(user_id, book_id);

-- ============================================================
-- THOUGHTS (raw user inputs)
-- ============================================================
CREATE TABLE thoughts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id UUID REFERENCES reading_sessions(id) ON DELETE SET NULL,
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    chunk_id UUID REFERENCES book_chunks(id),
    raw_text TEXT NOT NULL,
    llm_response JSONB, -- Full structured response
    accuracy_status VARCHAR(20) CHECK (accuracy_status IN ('correct', 'partially_correct', 'incorrect', 'vague')),
    node_id UUID REFERENCES nodes(id) ON DELETE SET NULL, -- resulting node if confirmed
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_thoughts_session ON thoughts(session_id);
CREATE INDEX idx_thoughts_book ON thoughts(book_id);

-- ============================================================
-- SPACED REPETITION QUEUE
-- ============================================================
CREATE TABLE review_queue (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    next_review_at TIMESTAMPTZ NOT NULL,
    interval_days INTEGER NOT NULL DEFAULT 1,
    ease_factor REAL NOT NULL DEFAULT 2.5,
    repetitions INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, node_id)
);

CREATE INDEX idx_review_due ON review_queue(user_id, next_review_at);

-- ============================================================
-- HELPER FUNCTIONS
-- ============================================================

-- Auto-update updated_at on row modification
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_books_updated_at BEFORE UPDATE ON books
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_nodes_updated_at BEFORE UPDATE ON nodes
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Find similar nodes by embedding (used for deduplication)
CREATE OR REPLACE FUNCTION find_similar_nodes(
    p_user_id UUID,
    p_embedding vector(1536),
    p_threshold REAL DEFAULT 0.92,
    p_limit INTEGER DEFAULT 5
)
RETURNS TABLE(node_id UUID, label VARCHAR, similarity REAL) AS $$
BEGIN
    RETURN QUERY
    SELECT n.id, n.label, (1 - (n.embedding <=> p_embedding))::REAL as sim
    FROM nodes n
    WHERE n.user_id = p_user_id
      AND n.embedding IS NOT NULL
      AND (1 - (n.embedding <=> p_embedding)) > p_threshold
    ORDER BY sim DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- Get graph stats for a user
CREATE OR REPLACE FUNCTION get_graph_stats(p_user_id UUID)
RETURNS TABLE(
    total_nodes BIGINT,
    total_edges BIGINT,
    total_books BIGINT,
    avg_confidence REAL,
    weakest_nodes BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        (SELECT COUNT(*) FROM nodes WHERE user_id = p_user_id),
        (SELECT COUNT(*) FROM edges WHERE user_id = p_user_id),
        (SELECT COUNT(*) FROM books WHERE user_id = p_user_id),
        (SELECT COALESCE(AVG(confidence_score), 0)::REAL FROM nodes WHERE user_id = p_user_id),
        (SELECT COUNT(*) FROM nodes n
         LEFT JOIN edges e ON n.id = e.source_node_id OR n.id = e.target_node_id
         WHERE n.user_id = p_user_id
         GROUP BY n.id
         HAVING COUNT(e.id) <= 1);
END;
$$ LANGUAGE plpgsql;
