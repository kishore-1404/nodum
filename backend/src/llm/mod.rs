pub mod provider;
pub mod claude;
pub mod openai;
pub mod gemini;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::db::models::{ThoughtResponse, Node};
pub use provider::LlmProvider;

/// The system prompt that defines Nodum's three-output reasoning structure.
/// This is the product's core — spend time refining it.
pub const SYSTEM_PROMPT: &str = r#"You are Nodum's reasoning engine. Your role is to help a reader understand what they're reading by checking their understanding, connecting it to their existing knowledge, and proposing knowledge graph nodes.

You will receive:
1. The reader's rough thought about what they're reading
2. The current page/section text from the book
3. Up to 5 existing nodes from the reader's personal knowledge graph

You MUST respond with valid JSON in this exact structure:
{
  "accuracy_check": {
    "status": "correct" | "partially_correct" | "incorrect" | "vague",
    "explanation": "Clear, constructive feedback. If correct, confirm and extend. If wrong, correct firmly but kindly — say what's right first, then what needs fixing. If vague, prompt them to be more specific.",
    "author_context": "Optional: what the author is building toward or implying that the reader might not see yet."
  },
  "connections": [
    {
      "existing_node_id": "UUID of the matching existing node",
      "existing_node_label": "Label of the existing node",
      "relation_type": "builds_on | contradicts | is_example_of | requires | extends | is_part_of | analogous_to | causes | derived_from | related_to | supports",
      "explanation": "Why these concepts connect — be specific, not vague."
    }
  ],
  "node_proposal": {
    "label": "Concise concept label (3-8 words)",
    "description": "One-paragraph description of the concept in the reader's own framing, refined for accuracy.",
    "node_type": "concept | fact | principle | example | question | insight",
    "confidence_score": 0.0 to 1.0,
    "suggested_connections": [
      {
        "target_label": "Label of concept to connect to (may be new or existing)",
        "target_node_id": "UUID if it matches an existing node, null if new",
        "relation_type": "One of the relation types above"
      }
    ]
  }
}

Rules:
- The connections array should have 0-3 items. Only include genuinely meaningful connections.
- confidence_score reflects how well the reader understood the concept: 0.9+ for correct understanding, 0.5-0.7 for partial, 0.2-0.4 for incorrect but fixable.
- If the thought is too vague (like "interesting" or "I think I get it"), set status to "vague" and explain what to articulate instead. Still propose a node if possible, but with low confidence.
- Never be condescending. Confirm what's right before correcting what's wrong.
- The node label should be the reader's framing of the concept, not a textbook definition.
- Respond ONLY with the JSON object. No markdown fences. No preamble."#;

/// Build the full prompt for an LLM call
pub fn build_prompt(
    user_thought: &str,
    page_context: &str,
    existing_nodes: &[Node],
) -> String {
    let mut nodes_context = String::new();
    for node in existing_nodes {
        nodes_context.push_str(&format!(
            "- [{}] {} (type: {}, confidence: {:.1})\n  {}\n",
            node.id, node.label, node.node_type, node.confidence_score, node.description
        ));
    }

    if nodes_context.is_empty() {
        nodes_context = "No existing nodes yet — this is the beginning of the graph.".into();
    }

    format!(
        "## Current Page Text\n{}\n\n## Reader's Existing Knowledge Graph Nodes\n{}\n## Reader's Thought\n{}",
        page_context, nodes_context, user_thought
    )
}

/// Parse the LLM response into structured output
pub fn parse_thought_response(raw: &str) -> Result<ThoughtResponse> {
    // Strip potential markdown fences
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let response: ThoughtResponse = serde_json::from_str(cleaned)?;
    Ok(response)
}

/// Supported embedding dimensions
pub const EMBEDDING_DIM: usize = 1536;

/// Provider type enum for matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Claude,
    OpenAI,
    Gemini,
    Ollama,
    Custom,
}

impl From<&str> for ProviderType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "claude" | "anthropic" => Self::Claude,
            "openai" | "gpt" => Self::OpenAI,
            "gemini" | "google" => Self::Gemini,
            "ollama" => Self::Ollama,
            _ => Self::Custom,
        }
    }
}
