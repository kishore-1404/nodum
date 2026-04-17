use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::Node;
use crate::llm::provider::LlmProvider;

/// Find semantically similar nodes to a given text
pub async fn find_similar_nodes(
    pool: &PgPool,
    user_id: Uuid,
    text: &str,
    llm: &dyn LlmProvider,
    limit: i32,
) -> Result<Vec<Node>> {
    // Generate embedding for the query text
    let embedding = llm.embed(text).await?;
    let embedding_str = format_embedding(&embedding);

    let nodes = sqlx::query_as::<_, Node>(&format!(
        "SELECT * FROM nodes
         WHERE user_id = $1 AND embedding IS NOT NULL
         ORDER BY embedding <=> '{}'
         LIMIT $2",
        embedding_str
    ))
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(nodes)
}

/// Store embedding for a node
pub async fn store_node_embedding(
    pool: &PgPool,
    node_id: Uuid,
    embedding: &[f32],
) -> Result<()> {
    let embedding_str = format_embedding(embedding);
    sqlx::query(&format!(
        "UPDATE nodes SET embedding = '{}' WHERE id = $1",
        embedding_str
    ))
    .bind(node_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Store embedding for a book chunk
pub async fn store_chunk_embedding(
    pool: &PgPool,
    chunk_id: Uuid,
    embedding: &[f32],
) -> Result<()> {
    let embedding_str = format_embedding(embedding);
    sqlx::query(&format!(
        "UPDATE book_chunks SET content_embedding = '{}' WHERE id = $1",
        embedding_str
    ))
    .bind(chunk_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Find near-duplicate nodes (for merge suggestions)
pub async fn find_duplicates(
    pool: &PgPool,
    user_id: Uuid,
    embedding: &[f32],
    threshold: f32,
) -> Result<Vec<(Uuid, String, f32)>> {
    let embedding_str = format_embedding(embedding);

    let rows: Vec<(Uuid, String, f32)> = sqlx::query_as(&format!(
        "SELECT id, label, (1 - (embedding <=> '{}'))::REAL as similarity
         FROM nodes
         WHERE user_id = $1 AND embedding IS NOT NULL
           AND (1 - (embedding <=> '{}')) > $2
         ORDER BY similarity DESC
         LIMIT 5",
        embedding_str, embedding_str
    ))
    .bind(user_id)
    .bind(threshold)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Format embedding vector for pgvector
fn format_embedding(embedding: &[f32]) -> String {
    let values: Vec<String> = embedding.iter().map(|v| format!("{:.6}", v)).collect();
    format!("[{}]", values.join(","))
}

/// Calculate "fog of war" — nodes with few connections
pub async fn get_weak_nodes(pool: &PgPool, user_id: Uuid, max_connections: i64) -> Result<Vec<Node>> {
    let nodes = sqlx::query_as::<_, Node>(
        "SELECT n.* FROM nodes n
         LEFT JOIN edges e ON (n.id = e.source_node_id OR n.id = e.target_node_id)
         WHERE n.user_id = $1
         GROUP BY n.id
         HAVING COUNT(e.id) <= $2
         ORDER BY n.confidence_score ASC"
    )
    .bind(user_id)
    .bind(max_connections)
    .fetch_all(pool)
    .await?;
    Ok(nodes)
}

/// Get connection statistics per book
pub async fn get_book_coverage(pool: &PgPool, user_id: Uuid, book_id: Uuid) -> Result<BookCoverage> {
    let total_chunks: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM book_chunks WHERE book_id = $1"
    ).bind(book_id).fetch_one(pool).await?;

    let covered_chunks: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT chunk_id) FROM nodes WHERE user_id = $1 AND book_id = $2 AND chunk_id IS NOT NULL"
    ).bind(user_id).bind(book_id).fetch_one(pool).await?;

    let total_nodes: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM nodes WHERE user_id = $1 AND book_id = $2"
    ).bind(user_id).bind(book_id).fetch_one(pool).await?;

    let avg_confidence: (Option<f32>,) = sqlx::query_as(
        "SELECT AVG(confidence_score)::REAL FROM nodes WHERE user_id = $1 AND book_id = $2"
    ).bind(user_id).bind(book_id).fetch_one(pool).await?;

    Ok(BookCoverage {
        total_chunks: total_chunks.0,
        covered_chunks: covered_chunks.0,
        coverage_percent: if total_chunks.0 > 0 { (covered_chunks.0 as f64 / total_chunks.0 as f64 * 100.0) } else { 0.0 },
        total_nodes: total_nodes.0,
        avg_confidence: avg_confidence.0.unwrap_or(0.0),
    })
}

#[derive(Debug, serde::Serialize)]
pub struct BookCoverage {
    pub total_chunks: i64,
    pub covered_chunks: i64,
    pub coverage_percent: f64,
    pub total_nodes: i64,
    pub avg_confidence: f32,
}
