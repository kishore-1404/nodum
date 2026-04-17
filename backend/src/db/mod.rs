pub mod models;

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

use models::*;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    let migration_sql = include_str!("../../migrations/001_initial.sql");
    sqlx::raw_sql(migration_sql).execute(pool).await?;
    tracing::info!("Database migrations applied");
    Ok(())
}

// ============================================================
// User operations
// ============================================================

pub async fn create_user(pool: &PgPool, req: &RegisterRequest, password_hash: &str) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash, display_name) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(&req.email)
    .bind(password_hash)
    .bind(&req.display_name)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

pub async fn get_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

// ============================================================
// Book operations
// ============================================================

pub async fn create_book(
    pool: &PgPool, user_id: Uuid, title: &str, author: Option<&str>,
    file_type: &str, file_path: Option<&str>, file_hash: Option<&str>,
    total_pages: Option<i32>, color: &str,
) -> Result<Book> {
    let book = sqlx::query_as::<_, Book>(
        "INSERT INTO books (user_id, title, author, file_type, file_path, file_hash, total_pages, color)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"
    )
    .bind(user_id).bind(title).bind(author).bind(file_type)
    .bind(file_path).bind(file_hash).bind(total_pages).bind(color)
    .fetch_one(pool)
    .await?;
    Ok(book)
}

pub async fn get_user_books(pool: &PgPool, user_id: Uuid) -> Result<Vec<Book>> {
    let books = sqlx::query_as::<_, Book>(
        "SELECT * FROM books WHERE user_id = $1 ORDER BY updated_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(books)
}

pub async fn get_book(pool: &PgPool, book_id: Uuid, user_id: Uuid) -> Result<Option<Book>> {
    let book = sqlx::query_as::<_, Book>(
        "SELECT * FROM books WHERE id = $1 AND user_id = $2"
    )
    .bind(book_id).bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(book)
}

pub async fn delete_book(pool: &PgPool, book_id: Uuid, user_id: Uuid) -> Result<bool> {
    let result = sqlx::query("DELETE FROM books WHERE id = $1 AND user_id = $2")
        .bind(book_id).bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ============================================================
// Chunk operations
// ============================================================

pub async fn insert_chunks(pool: &PgPool, book_id: Uuid, chunks: &[(i32, Option<i32>, Option<String>, String)]) -> Result<Vec<BookChunk>> {
    let mut inserted = Vec::new();
    for (idx, page, chapter_title, content) in chunks {
        let token_count = content.split_whitespace().count() as i32;
        let chunk = sqlx::query_as::<_, BookChunk>(
            "INSERT INTO book_chunks (book_id, chunk_index, page_number, chapter_title, content, token_count)
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
        )
        .bind(book_id).bind(idx).bind(page).bind(chapter_title).bind(content).bind(token_count)
        .fetch_one(pool)
        .await?;
        inserted.push(chunk);
    }
    Ok(inserted)
}

pub async fn get_chunks_for_page(pool: &PgPool, book_id: Uuid, page: i32) -> Result<Vec<BookChunk>> {
    let chunks = sqlx::query_as::<_, BookChunk>(
        "SELECT * FROM book_chunks WHERE book_id = $1 AND page_number = $2 ORDER BY chunk_index"
    )
    .bind(book_id).bind(page)
    .fetch_all(pool)
    .await?;
    Ok(chunks)
}

pub async fn get_chunk_by_id(pool: &PgPool, chunk_id: Uuid) -> Result<Option<BookChunk>> {
    let chunk = sqlx::query_as::<_, BookChunk>("SELECT * FROM book_chunks WHERE id = $1")
        .bind(chunk_id)
        .fetch_optional(pool)
        .await?;
    Ok(chunk)
}

pub async fn get_book_chunks(pool: &PgPool, book_id: Uuid, offset: i64, limit: i64) -> Result<Vec<BookChunk>> {
    let chunks = sqlx::query_as::<_, BookChunk>(
        "SELECT * FROM book_chunks WHERE book_id = $1 ORDER BY chunk_index LIMIT $2 OFFSET $3"
    )
    .bind(book_id).bind(limit).bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(chunks)
}

// ============================================================
// Node operations
// ============================================================

pub async fn create_node(
    pool: &PgPool, user_id: Uuid, book_id: Option<Uuid>, chunk_id: Option<Uuid>,
    label: &str, description: &str, user_note: Option<&str>, accuracy_note: Option<&str>,
    node_type: &str, confidence_score: f32,
) -> Result<Node> {
    let node = sqlx::query_as::<_, Node>(
        "INSERT INTO nodes (user_id, book_id, chunk_id, label, description, user_note, accuracy_note, node_type, confidence_score)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *"
    )
    .bind(user_id).bind(book_id).bind(chunk_id).bind(label).bind(description)
    .bind(user_note).bind(accuracy_note).bind(node_type).bind(confidence_score)
    .fetch_one(pool)
    .await?;
    Ok(node)
}

pub async fn get_user_nodes(pool: &PgPool, user_id: Uuid, book_id: Option<Uuid>) -> Result<Vec<Node>> {
    if let Some(bid) = book_id {
        let nodes = sqlx::query_as::<_, Node>(
            "SELECT * FROM nodes WHERE user_id = $1 AND book_id = $2 ORDER BY created_at DESC"
        )
        .bind(user_id).bind(bid)
        .fetch_all(pool)
        .await?;
        Ok(nodes)
    } else {
        let nodes = sqlx::query_as::<_, Node>(
            "SELECT * FROM nodes WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        Ok(nodes)
    }
}

pub async fn get_node(pool: &PgPool, node_id: Uuid) -> Result<Option<Node>> {
    let node = sqlx::query_as::<_, Node>("SELECT * FROM nodes WHERE id = $1")
        .bind(node_id)
        .fetch_optional(pool)
        .await?;
    Ok(node)
}

pub async fn update_node(pool: &PgPool, node_id: Uuid, req: &UpdateNodeRequest) -> Result<Option<Node>> {
    let node = sqlx::query_as::<_, Node>(
        "UPDATE nodes SET
            label = COALESCE($2, label),
            description = COALESCE($3, description),
            user_note = COALESCE($4, user_note),
            node_type = COALESCE($5, node_type)
         WHERE id = $1 RETURNING *"
    )
    .bind(node_id)
    .bind(&req.label)
    .bind(&req.description)
    .bind(&req.user_note)
    .bind(&req.node_type)
    .fetch_optional(pool)
    .await?;
    Ok(node)
}

pub async fn delete_node(pool: &PgPool, node_id: Uuid, user_id: Uuid) -> Result<bool> {
    let result = sqlx::query("DELETE FROM nodes WHERE id = $1 AND user_id = $2")
        .bind(node_id).bind(user_id)
        .execute(pool).await?;
    Ok(result.rows_affected() > 0)
}

pub async fn increment_node_confirmed(pool: &PgPool, node_id: Uuid) -> Result<()> {
    sqlx::query("UPDATE nodes SET times_confirmed = times_confirmed + 1, confidence_score = LEAST(1.0, confidence_score + 0.05) WHERE id = $1")
        .bind(node_id).execute(pool).await?;
    Ok(())
}

pub async fn increment_node_corrected(pool: &PgPool, node_id: Uuid) -> Result<()> {
    sqlx::query("UPDATE nodes SET times_corrected = times_corrected + 1, confidence_score = GREATEST(0.0, confidence_score - 0.1) WHERE id = $1")
        .bind(node_id).execute(pool).await?;
    Ok(())
}

// ============================================================
// Edge operations
// ============================================================

pub async fn create_edge(
    pool: &PgPool, user_id: Uuid, source_id: Uuid, target_id: Uuid,
    relation_type: &str, llm_generated: bool, description: Option<&str>,
) -> Result<Edge> {
    let edge = sqlx::query_as::<_, Edge>(
        "INSERT INTO edges (user_id, source_node_id, target_node_id, relation_type, llm_generated, description)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (source_node_id, target_node_id, relation_type) DO UPDATE SET weight = edges.weight + 0.1
         RETURNING *"
    )
    .bind(user_id).bind(source_id).bind(target_id).bind(relation_type)
    .bind(llm_generated).bind(description)
    .fetch_one(pool)
    .await?;
    Ok(edge)
}

pub async fn get_user_edges(pool: &PgPool, user_id: Uuid) -> Result<Vec<Edge>> {
    let edges = sqlx::query_as::<_, Edge>(
        "SELECT * FROM edges WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(edges)
}

pub async fn delete_edge(pool: &PgPool, edge_id: Uuid, user_id: Uuid) -> Result<bool> {
    let result = sqlx::query("DELETE FROM edges WHERE id = $1 AND user_id = $2")
        .bind(edge_id).bind(user_id)
        .execute(pool).await?;
    Ok(result.rows_affected() > 0)
}

// ============================================================
// Graph data (combined nodes + edges)
// ============================================================

pub async fn get_graph_data(pool: &PgPool, user_id: Uuid, book_id: Option<Uuid>) -> Result<GraphData> {
    let nodes: Vec<GraphNode> = if let Some(bid) = book_id {
        sqlx::query_as::<_, GraphNode>(
            "SELECT n.id, n.label, n.description, n.node_type, n.confidence_score,
                    n.book_id, b.color as book_color, b.title as book_title,
                    COALESCE(ec.cnt, 0) as connection_count, n.created_at
             FROM nodes n
             LEFT JOIN books b ON n.book_id = b.id
             LEFT JOIN (
                SELECT node_id, COUNT(*) as cnt FROM (
                    SELECT source_node_id as node_id FROM edges WHERE user_id = $1
                    UNION ALL
                    SELECT target_node_id as node_id FROM edges WHERE user_id = $1
                ) sub GROUP BY node_id
             ) ec ON n.id = ec.node_id
             WHERE n.user_id = $1 AND n.book_id = $2"
        )
        .bind(user_id).bind(bid)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, GraphNode>(
            "SELECT n.id, n.label, n.description, n.node_type, n.confidence_score,
                    n.book_id, b.color as book_color, b.title as book_title,
                    COALESCE(ec.cnt, 0) as connection_count, n.created_at
             FROM nodes n
             LEFT JOIN books b ON n.book_id = b.id
             LEFT JOIN (
                SELECT node_id, COUNT(*) as cnt FROM (
                    SELECT source_node_id as node_id FROM edges WHERE user_id = $1
                    UNION ALL
                    SELECT target_node_id as node_id FROM edges WHERE user_id = $1
                ) sub GROUP BY node_id
             ) ec ON n.id = ec.node_id
             WHERE n.user_id = $1"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
    };

    let edges: Vec<GraphEdge> = sqlx::query_as::<_, GraphEdge>(
        "SELECT id, source_node_id as source, target_node_id as target,
                relation_type, weight, llm_generated
         FROM edges WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(GraphData { nodes, edges })
}

// ============================================================
// Session operations
// ============================================================

pub async fn create_session(pool: &PgPool, user_id: Uuid, book_id: Uuid) -> Result<ReadingSession> {
    let session = sqlx::query_as::<_, ReadingSession>(
        "INSERT INTO reading_sessions (user_id, book_id) VALUES ($1, $2) RETURNING *"
    )
    .bind(user_id).bind(book_id)
    .fetch_one(pool).await?;
    Ok(session)
}

pub async fn end_session(pool: &PgPool, session_id: Uuid) -> Result<Option<ReadingSession>> {
    let session = sqlx::query_as::<_, ReadingSession>(
        "UPDATE reading_sessions SET
            ended_at = NOW(),
            duration_seconds = EXTRACT(EPOCH FROM (NOW() - started_at))::INTEGER
         WHERE id = $1 RETURNING *"
    )
    .bind(session_id)
    .fetch_optional(pool).await?;
    Ok(session)
}

pub async fn update_session_stats(pool: &PgPool, session_id: Uuid, nodes_created: i32, nodes_reinforced: i32, thoughts: i32) -> Result<()> {
    sqlx::query(
        "UPDATE reading_sessions SET nodes_created = $2, nodes_reinforced = $3, thoughts_submitted = $4 WHERE id = $1"
    )
    .bind(session_id).bind(nodes_created).bind(nodes_reinforced).bind(thoughts)
    .execute(pool).await?;
    Ok(())
}

// ============================================================
// Thought operations
// ============================================================

pub async fn create_thought(
    pool: &PgPool, user_id: Uuid, session_id: Option<Uuid>, book_id: Uuid,
    chunk_id: Option<Uuid>, raw_text: &str,
) -> Result<Thought> {
    let thought = sqlx::query_as::<_, Thought>(
        "INSERT INTO thoughts (user_id, session_id, book_id, chunk_id, raw_text)
         VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(user_id).bind(session_id).bind(book_id).bind(chunk_id).bind(raw_text)
    .fetch_one(pool).await?;
    Ok(thought)
}

pub async fn update_thought_response(
    pool: &PgPool, thought_id: Uuid, response: &serde_json::Value,
    accuracy_status: &str, node_id: Option<Uuid>,
) -> Result<()> {
    sqlx::query(
        "UPDATE thoughts SET llm_response = $2, accuracy_status = $3, node_id = $4 WHERE id = $1"
    )
    .bind(thought_id).bind(response).bind(accuracy_status).bind(node_id)
    .execute(pool).await?;
    Ok(())
}

// ============================================================
// LLM Provider operations
// ============================================================

pub async fn upsert_llm_provider(
    pool: &PgPool, user_id: Uuid, req: &ConfigureProviderRequest,
) -> Result<LlmProvider> {
    let provider = sqlx::query_as::<_, LlmProvider>(
        "INSERT INTO llm_providers (user_id, provider, api_key_encrypted, model_name, base_url, is_default)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (user_id, provider, model_name) DO UPDATE SET
            api_key_encrypted = EXCLUDED.api_key_encrypted,
            base_url = EXCLUDED.base_url,
            is_default = EXCLUDED.is_default
         RETURNING *"
    )
    .bind(user_id).bind(&req.provider).bind(&req.api_key).bind(&req.model_name)
    .bind(&req.base_url).bind(req.is_default)
    .fetch_one(pool).await?;

    // If this is set as default, unset others
    if req.is_default {
        sqlx::query("UPDATE llm_providers SET is_default = false WHERE user_id = $1 AND id != $2")
            .bind(user_id).bind(provider.id)
            .execute(pool).await?;
    }
    Ok(provider)
}

pub async fn get_user_providers(pool: &PgPool, user_id: Uuid) -> Result<Vec<LlmProvider>> {
    let providers = sqlx::query_as::<_, LlmProvider>(
        "SELECT * FROM llm_providers WHERE user_id = $1 ORDER BY is_default DESC"
    )
    .bind(user_id)
    .fetch_all(pool).await?;
    Ok(providers)
}

pub async fn get_default_provider(pool: &PgPool, user_id: Uuid) -> Result<Option<LlmProvider>> {
    let provider = sqlx::query_as::<_, LlmProvider>(
        "SELECT * FROM llm_providers WHERE user_id = $1 AND is_default = true LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(pool).await?;
    Ok(provider)
}

// ============================================================
// Search (text-based, vector search done separately)
// ============================================================

pub async fn search_nodes_text(pool: &PgPool, user_id: Uuid, query: &str, limit: i32) -> Result<Vec<Node>> {
    let nodes = sqlx::query_as::<_, Node>(
        "SELECT * FROM nodes WHERE user_id = $1
         AND (label ILIKE '%' || $2 || '%' OR description ILIKE '%' || $2 || '%')
         ORDER BY similarity(label, $2) DESC
         LIMIT $3"
    )
    .bind(user_id).bind(query).bind(limit)
    .fetch_all(pool).await?;
    Ok(nodes)
}
