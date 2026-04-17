use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================
// Database models
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub preferences: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LlmProvider {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub api_key_encrypted: Option<String>,
    pub model_name: String,
    pub base_url: Option<String>,
    pub is_default: bool,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Book {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub author: Option<String>,
    pub file_type: String,
    pub file_path: Option<String>,
    pub file_hash: Option<String>,
    pub cover_image_url: Option<String>,
    pub total_pages: Option<i32>,
    pub total_chapters: Option<i32>,
    pub metadata: serde_json::Value,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BookChunk {
    pub id: Uuid,
    pub book_id: Uuid,
    pub chunk_index: i32,
    pub page_number: Option<i32>,
    pub chapter_number: Option<i32>,
    pub chapter_title: Option<String>,
    pub content: String,
    pub token_count: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Node {
    pub id: Uuid,
    pub user_id: Uuid,
    pub book_id: Option<Uuid>,
    pub chunk_id: Option<Uuid>,
    pub label: String,
    pub description: String,
    pub user_note: Option<String>,
    pub accuracy_note: Option<String>,
    pub node_type: String,
    pub confidence_score: f32,
    pub times_confirmed: i32,
    pub times_corrected: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Edge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub source_node_id: Uuid,
    pub target_node_id: Uuid,
    pub relation_type: String,
    pub weight: f32,
    pub llm_generated: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReadingSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub book_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub last_page: Option<i32>,
    pub last_chunk_id: Option<Uuid>,
    pub nodes_created: i32,
    pub nodes_reinforced: i32,
    pub thoughts_submitted: i32,
    pub duration_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Thought {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_id: Option<Uuid>,
    pub book_id: Uuid,
    pub chunk_id: Option<Uuid>,
    pub raw_text: String,
    pub llm_response: Option<serde_json::Value>,
    pub accuracy_status: Option<String>,
    pub node_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub node_id: Uuid,
    pub next_review_at: DateTime<Utc>,
    pub interval_days: i32,
    pub ease_factor: f32,
    pub repetitions: i32,
    pub created_at: DateTime<Utc>,
}

// ============================================================
// API request/response types
// ============================================================

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublic,
}

#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub preferences: serde_json::Value,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            display_name: u.display_name,
            preferences: u.preferences,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SubmitThoughtRequest {
    pub book_id: Uuid,
    pub chunk_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub raw_text: String,
    pub page_context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThoughtResponse {
    pub accuracy_check: AccuracyCheck,
    pub connections: Vec<ConnectionProposal>,
    pub node_proposal: NodeProposal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccuracyCheck {
    pub status: String, // correct, partially_correct, incorrect, vague
    pub explanation: String,
    pub author_context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionProposal {
    pub existing_node_id: Uuid,
    pub existing_node_label: String,
    pub relation_type: String,
    pub explanation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeProposal {
    pub label: String,
    pub description: String,
    pub node_type: String,
    pub confidence_score: f32,
    pub suggested_connections: Vec<SuggestedConnection>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SuggestedConnection {
    pub target_label: String,
    pub target_node_id: Option<Uuid>,
    pub relation_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmNodeRequest {
    pub thought_id: Uuid,
    pub label: String,
    pub description: String,
    pub node_type: String,
    pub connections: Vec<ConfirmConnection>,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmConnection {
    pub target_node_id: Uuid,
    pub relation_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateManualEdgeRequest {
    pub source_node_id: Uuid,
    pub target_node_id: Uuid,
    pub relation_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNodeRequest {
    pub label: Option<String>,
    pub description: Option<String>,
    pub user_note: Option<String>,
    pub node_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Serialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub label: String,
    pub description: String,
    pub node_type: String,
    pub confidence_score: f32,
    pub book_id: Option<Uuid>,
    pub book_color: Option<String>,
    pub book_title: Option<String>,
    pub connection_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct GraphEdge {
    pub id: Uuid,
    pub source: Uuid,
    pub target: Uuid,
    pub relation_type: String,
    pub weight: f32,
    pub llm_generated: bool,
}

#[derive(Debug, Serialize)]
pub struct SessionSummary {
    pub session_id: Uuid,
    pub duration_seconds: i32,
    pub nodes_created: i32,
    pub nodes_reinforced: i32,
    pub thoughts_submitted: i32,
    pub weak_areas: Vec<String>, // labels of thin/isolated nodes
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<i32>,
    pub book_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigureProviderRequest {
    pub provider: String,
    pub api_key: Option<String>,
    pub model_name: String,
    pub base_url: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Serialize)]
pub struct BookUploadResponse {
    pub book: Book,
    pub chunks_created: usize,
}
