export interface User {
  id: string;
  email: string;
  display_name: string;
  preferences: UserPreferences;
}

export interface UserPreferences {
  note_style: 'inline' | 'after_section' | 'chapter_end';
  theme: 'dark' | 'light';
  default_llm_provider: string;
}

export interface Book {
  id: string;
  user_id: string;
  title: string;
  author: string | null;
  file_type: 'pdf' | 'epub' | 'url' | 'markdown';
  file_path: string | null;
  cover_image_url: string | null;
  total_pages: number | null;
  total_chapters: number | null;
  metadata: Record<string, unknown>;
  color: string;
  created_at: string;
  updated_at: string;
}

export interface BookChunk {
  id: string;
  book_id: string;
  chunk_index: number;
  page_number: number | null;
  chapter_number: number | null;
  chapter_title: string | null;
  content: string;
  token_count: number;
  metadata: Record<string, unknown>;
  created_at: string;
}

export interface KnowledgeNode {
  id: string;
  user_id: string;
  book_id: string | null;
  chunk_id: string | null;
  label: string;
  description: string;
  user_note: string | null;
  accuracy_note: string | null;
  node_type: NodeType;
  confidence_score: number;
  times_confirmed: number;
  times_corrected: number;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
  last_reviewed_at: string | null;
}

export type NodeType = 'concept' | 'fact' | 'principle' | 'example' | 'question' | 'prior_knowledge' | 'insight';

export interface KnowledgeEdge {
  id: string;
  source: string;
  target: string;
  relation_type: RelationType;
  weight: number;
  llm_generated: boolean;
}

export type RelationType =
  | 'builds_on' | 'contradicts' | 'is_example_of'
  | 'requires' | 'extends' | 'is_part_of'
  | 'analogous_to' | 'causes' | 'derived_from'
  | 'related_to' | 'supports';

export interface GraphData {
  nodes: GraphNode[];
  edges: KnowledgeEdge[];
}

export interface GraphNode {
  id: string;
  label: string;
  description: string;
  node_type: NodeType;
  confidence_score: number;
  book_id: string | null;
  book_color: string | null;
  book_title: string | null;
  connection_count: number;
  created_at: string;
}

export interface ThoughtResponse {
  thought_id: string;
  response: {
    accuracy_check: {
      status: 'correct' | 'partially_correct' | 'incorrect' | 'vague';
      explanation: string;
      author_context: string | null;
    };
    connections: Array<{
      existing_node_id: string;
      existing_node_label: string;
      relation_type: RelationType;
      explanation: string;
    }>;
    node_proposal: {
      label: string;
      description: string;
      node_type: NodeType;
      confidence_score: number;
      suggested_connections: Array<{
        target_label: string;
        target_node_id: string | null;
        relation_type: RelationType;
      }>;
    };
  };
}

export interface ReadingSession {
  id: string;
  user_id: string;
  book_id: string;
  started_at: string;
  ended_at: string | null;
  last_page: number | null;
  nodes_created: number;
  nodes_reinforced: number;
  thoughts_submitted: number;
  duration_seconds: number | null;
}

export interface LlmProviderConfig {
  id: string;
  provider: string;
  model_name: string;
  base_url: string | null;
  is_default: boolean;
}

export interface BookCoverage {
  total_chunks: number;
  covered_chunks: number;
  coverage_percent: number;
  total_nodes: number;
  avg_confidence: number;
}

export interface SessionSummary {
  session_id: string;
  duration_seconds: number;
  nodes_created: number;
  nodes_reinforced: number;
  thoughts_submitted: number;
  weak_areas: string[];
}
