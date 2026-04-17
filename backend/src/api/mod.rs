use axum::{
    extract::{Json, Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;
use std::sync::Arc;

use crate::db;
use crate::db::models::*;
use crate::graph;
use crate::llm;
use crate::llm::provider;
use crate::reader;
use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // Auth
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        // Books
        .route("/api/books", get(list_books).post(upload_book))
        .route("/api/books/:id", get(get_book).delete(delete_book))
        .route("/api/books/:id/chunks", get(get_book_chunks))
        .route("/api/books/:id/coverage", get(get_book_coverage))
        // Thoughts (the core loop)
        .route("/api/thoughts", post(submit_thought))
        // Nodes
        .route("/api/nodes", get(list_nodes).post(confirm_node))
        .route("/api/nodes/:id", get(get_node).put(update_node).delete(delete_node))
        .route("/api/nodes/search", post(search_nodes))
        // Edges
        .route("/api/edges", post(create_edge).get(list_edges))
        .route("/api/edges/:id", delete(delete_edge))
        // Graph
        .route("/api/graph", get(get_graph))
        .route("/api/graph/weak", get(get_weak_nodes))
        // Sessions
        .route("/api/sessions", post(start_session))
        .route("/api/sessions/:id/end", post(end_session))
        // LLM Providers
        .route("/api/providers", get(list_providers).post(configure_provider))
        // Health
        .route("/api/health", get(health_check))
}

// ============================================================
// Auth handlers
// ============================================================

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let password_hash = bcrypt::hash(&req.password, 10)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let user = db::create_user(&state.pool, &req, &password_hash).await?;
    let token = create_jwt(&state.jwt_secret, user.id)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = db::get_user_by_email(&state.pool, &req.email).await?
        .ok_or(AppError::NotFound("User not found".into()))?;

    if !bcrypt::verify(&req.password, &user.password_hash).unwrap_or(false) {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    let token = create_jwt(&state.jwt_secret, user.id)?;
    Ok(Json(AuthResponse { token, user: user.into() }))
}

// ============================================================
// Book handlers
// ============================================================

async fn list_books(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Vec<Book>>, AppError> {
    let books = db::get_user_books(&state.pool, user.id).await?;
    Ok(Json(books))
}

async fn upload_book(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<BookUploadResponse>, AppError> {
    let mut file_data: Option<(String, Vec<u8>)> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        if field.name() == Some("file") {
            let filename = field.file_name().unwrap_or("document").to_string();
            let data = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
            file_data = Some((filename, data.to_vec()));
        }
    }

    let (filename, data) = file_data.ok_or(AppError::BadRequest("No file provided".into()))?;

    // Determine file type
    let file_type = if filename.ends_with(".pdf") {
        "pdf"
    } else if filename.ends_with(".epub") {
        "epub"
    } else {
        return Err(AppError::BadRequest("Unsupported file type. Upload PDF or EPUB.".into()));
    };

    // Save file to disk
    let file_hash = format!("{:x}", sha2::Sha256::digest(&data));
    let file_path = format!("{}/{}_{}", state.upload_dir, file_hash, filename);
    tokio::fs::create_dir_all(&state.upload_dir).await.ok();
    tokio::fs::write(&file_path, &data).await
        .map_err(|e| AppError::Internal(format!("Failed to save file: {}", e)))?;

    // Parse document
    let tmp = tempfile::NamedTempFile::new().map_err(|e| AppError::Internal(e.to_string()))?;
    std::fs::write(tmp.path(), &data).map_err(|e| AppError::Internal(e.to_string()))?;

    let (title, chunks) = match file_type {
        "pdf" => reader::parse_pdf(tmp.path())?,
        "epub" => reader::parse_epub(tmp.path())?,
        _ => unreachable!(),
    };

    // Generate a random book color
    let colors = ["#6366f1", "#ec4899", "#f59e0b", "#10b981", "#3b82f6", "#8b5cf6", "#ef4444", "#14b8a6"];
    let color = colors[rand::random::<usize>() % colors.len()];

    let total_pages = chunks.last().and_then(|c| c.page_number);
    let book = db::create_book(
        &state.pool, user.id, &title, None, file_type,
        Some(&file_path), Some(&file_hash), total_pages, color,
    ).await?;

    // Insert chunks
    let chunk_data: Vec<(i32, Option<i32>, Option<String>, String)> = chunks.iter().map(|c| {
        (c.index, c.page_number, c.chapter_title.clone(), c.content.clone())
    }).collect();

    let inserted = db::insert_chunks(&state.pool, book.id, &chunk_data).await?;

    Ok(Json(BookUploadResponse {
        book,
        chunks_created: inserted.len(),
    }))
}

async fn get_book(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Book>, AppError> {
    let book = db::get_book(&state.pool, id, user.id).await?
        .ok_or(AppError::NotFound("Book not found".into()))?;
    Ok(Json(book))
}

async fn delete_book(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    db::delete_book(&state.pool, id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct ChunkQuery {
    offset: Option<i64>,
    limit: Option<i64>,
}

async fn get_book_chunks(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<ChunkQuery>,
) -> Result<Json<Vec<BookChunk>>, AppError> {
    let chunks = db::get_book_chunks(&state.pool, id, q.offset.unwrap_or(0), q.limit.unwrap_or(20)).await?;
    Ok(Json(chunks))
}

async fn get_book_coverage(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<graph::BookCoverage>, AppError> {
    let coverage = graph::get_book_coverage(&state.pool, user.id, id).await?;
    Ok(Json(coverage))
}

// ============================================================
// Thought handler (THE CORE LOOP)
// ============================================================

async fn submit_thought(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<SubmitThoughtRequest>,
) -> Result<Json<ThoughtResponseWrapper>, AppError> {
    // 1. Create thought record
    let thought = db::create_thought(
        &state.pool, user.id, req.session_id, req.book_id,
        req.chunk_id, &req.raw_text,
    ).await?;

    // 2. Get page context (from chunk or provided)
    let page_context = if let Some(ctx) = &req.page_context {
        ctx.clone()
    } else if let Some(chunk_id) = req.chunk_id {
        db::get_chunk_by_id(&state.pool, chunk_id).await?
            .map(|c| c.content)
            .unwrap_or_default()
    } else {
        String::new()
    };

    // 3. Get LLM provider for user
    let provider_config = db::get_default_provider(&state.pool, user.id).await?;
    let llm = resolve_provider(&state, &provider_config)?;

    // 4. Find related existing nodes via semantic search
    let existing_nodes = graph::find_similar_nodes(
        &state.pool, user.id, &req.raw_text, llm.as_ref(), 5,
    ).await.unwrap_or_default();

    // 5. Build prompt and call LLM
    let prompt = llm::build_prompt(&req.raw_text, &page_context, &existing_nodes);
    let raw_response = llm.complete(llm::SYSTEM_PROMPT, &prompt).await?;

    // 6. Parse structured response
    let response = llm::parse_thought_response(&raw_response)
        .map_err(|e| AppError::Internal(format!("Failed to parse LLM response: {}. Raw: {}", e, &raw_response[..200.min(raw_response.len())])))?;

    // 7. Update thought record
    let response_json = serde_json::to_value(&response)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    db::update_thought_response(
        &state.pool, thought.id, &response_json,
        &response.accuracy_check.status, None,
    ).await?;

    Ok(Json(ThoughtResponseWrapper {
        thought_id: thought.id,
        response,
    }))
}

#[derive(serde::Serialize)]
struct ThoughtResponseWrapper {
    thought_id: Uuid,
    response: ThoughtResponse,
}

// ============================================================
// Node handlers
// ============================================================

#[derive(Deserialize)]
struct NodeQuery {
    book_id: Option<Uuid>,
}

async fn list_nodes(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(q): Query<NodeQuery>,
) -> Result<Json<Vec<Node>>, AppError> {
    let nodes = db::get_user_nodes(&state.pool, user.id, q.book_id).await?;
    Ok(Json(nodes))
}

async fn confirm_node(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<ConfirmNodeRequest>,
) -> Result<Json<Node>, AppError> {
    // Create the node
    let node = db::create_node(
        &state.pool, user.id, None, None,
        &req.label, &req.description, None, None,
        &req.node_type, 0.5,
    ).await?;

    // Create edges for confirmed connections
    for conn in &req.connections {
        db::create_edge(
            &state.pool, user.id, node.id, conn.target_node_id,
            &conn.relation_type, true, None,
        ).await?;
    }

    // Generate and store embedding
    let provider_config = db::get_default_provider(&state.pool, user.id).await?;
    let llm = resolve_provider(&state, &provider_config)?;
    if let Ok(embedding) = llm.embed(&format!("{}: {}", req.label, req.description)).await {
        graph::store_node_embedding(&state.pool, node.id, &embedding).await.ok();
    }

    // Update thought with node reference
    db::update_thought_response(
        &state.pool, req.thought_id,
        &serde_json::json!({}), "confirmed", Some(node.id),
    ).await.ok();

    Ok(Json(node))
}

async fn get_node(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Node>, AppError> {
    let node = db::get_node(&state.pool, id).await?
        .ok_or(AppError::NotFound("Node not found".into()))?;
    Ok(Json(node))
}

async fn update_node(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateNodeRequest>,
) -> Result<Json<Node>, AppError> {
    let node = db::update_node(&state.pool, id, &req).await?
        .ok_or(AppError::NotFound("Node not found".into()))?;
    Ok(Json(node))
}

async fn delete_node(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    db::delete_node(&state.pool, id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn search_nodes(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<SearchRequest>,
) -> Result<Json<Vec<Node>>, AppError> {
    let nodes = db::search_nodes_text(&state.pool, user.id, &req.query, req.limit.unwrap_or(20)).await?;
    Ok(Json(nodes))
}

// ============================================================
// Edge handlers
// ============================================================

async fn create_edge(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<CreateManualEdgeRequest>,
) -> Result<Json<Edge>, AppError> {
    let edge = db::create_edge(
        &state.pool, user.id, req.source_node_id, req.target_node_id,
        &req.relation_type, false, req.description.as_deref(),
    ).await?;
    Ok(Json(edge))
}

async fn list_edges(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Vec<Edge>>, AppError> {
    let edges = db::get_user_edges(&state.pool, user.id).await?;
    Ok(Json(edges))
}

async fn delete_edge(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    db::delete_edge(&state.pool, id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================
// Graph handlers
// ============================================================

#[derive(Deserialize)]
struct GraphQuery {
    book_id: Option<Uuid>,
}

async fn get_graph(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(q): Query<GraphQuery>,
) -> Result<Json<GraphData>, AppError> {
    let graph = db::get_graph_data(&state.pool, user.id, q.book_id).await?;
    Ok(Json(graph))
}

async fn get_weak_nodes(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Vec<Node>>, AppError> {
    let nodes = graph::get_weak_nodes(&state.pool, user.id, 1).await?;
    Ok(Json(nodes))
}

// ============================================================
// Session handlers
// ============================================================

async fn start_session(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<ReadingSession>, AppError> {
    let book_id: Uuid = serde_json::from_value(req["book_id"].clone())
        .map_err(|_| AppError::BadRequest("book_id required".into()))?;
    let session = db::create_session(&state.pool, user.id, book_id).await?;
    Ok(Json(session))
}

async fn end_session(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ReadingSession>, AppError> {
    let session = db::end_session(&state.pool, id).await?
        .ok_or(AppError::NotFound("Session not found".into()))?;
    Ok(Json(session))
}

// ============================================================
// Provider handlers
// ============================================================

async fn list_providers(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Vec<LlmProvider>>, AppError> {
    let providers = db::get_user_providers(&state.pool, user.id).await?;
    Ok(Json(providers))
}

async fn configure_provider(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<ConfigureProviderRequest>,
) -> Result<Json<LlmProvider>, AppError> {
    let provider = db::upsert_llm_provider(&state.pool, user.id, &req).await?;
    Ok(Json(provider))
}

// ============================================================
// Health check
// ============================================================

async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_ok = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
    Json(serde_json::json!({
        "status": if db_ok { "healthy" } else { "degraded" },
        "database": db_ok,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// ============================================================
// Auth middleware / extractor
// ============================================================

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub struct AuthUser {
    pub id: Uuid,
}

#[axum::async_trait]
impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let token = auth_header.strip_prefix("Bearer ").unwrap_or(auth_header);

        if token.is_empty() {
            return Err(AppError::Unauthorized("Missing authorization token".into()));
        }

        let claims = decode_jwt(&state.jwt_secret, token)?;
        Ok(AuthUser { id: claims.sub })
    }
}

// ============================================================
// JWT helpers
// ============================================================

use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Claims {
    sub: Uuid,
    exp: usize,
}

fn create_jwt(secret: &str, user_id: Uuid) -> Result<String, AppError> {
    let expiry = chrono::Utc::now() + chrono::Duration::hours(72);
    let claims = Claims { sub: user_id, exp: expiry.timestamp() as usize };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(e.to_string()))
}

fn decode_jwt(secret: &str, token: &str) -> Result<Claims, AppError> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))
}

// ============================================================
// Provider resolution helper
// ============================================================

fn resolve_provider(
    state: &AppState,
    user_provider: &Option<crate::db::models::LlmProvider>,
) -> Result<Box<dyn llm::LlmProvider>, AppError> {
    if let Some(up) = user_provider {
        let config = provider::ProviderConfig {
            api_key: up.api_key_encrypted.clone(),
            model_name: up.model_name.clone(),
            base_url: up.base_url.clone(),
            max_tokens: 2048,
            temperature: 0.3,
        };
        provider::create_provider(&up.provider, config)
            .map_err(|e| AppError::Internal(e.to_string()))
    } else {
        // Fall back to server-configured Claude
        let key = state.config.claude_api_key.clone()
            .or(state.config.openai_api_key.clone())
            .ok_or(AppError::BadRequest("No LLM provider configured. Add an API key in Settings.".into()))?;

        if state.config.claude_api_key.is_some() {
            Ok(Box::new(crate::llm::claude::ClaudeProvider::new(
                key, "claude-sonnet-4-20250514".into(), 2048, 0.3,
            )))
        } else {
            Ok(Box::new(crate::llm::openai::OpenAIProvider::new(
                key, "gpt-4o".into(), None, 2048, 0.3,
            )))
        }
    }
}

// ============================================================
// Error handling
// ============================================================

use sha2::Digest;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Internal(String),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Internal(format!("Database error: {}", e))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
            }
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
