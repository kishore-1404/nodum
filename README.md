# Nodum — The Knowledge-Connecting Book Reader

<p align="center">
  <img src="frontend/public/nodum-icon.svg" width="80" alt="Nodum" />
</p>

<p align="center">
  <strong>Read books. Drop rough thoughts. Build a living knowledge graph.</strong><br/>
  <em>The LLM never reads the book — you do. It only checks your understanding.</em>
</p>

<p align="center">
  <a href="#quickstart">Quickstart</a> •
  <a href="#how-it-works">How It Works</a> •
  <a href="#architecture">Architecture</a> •
  <a href="#llm-providers">LLM Providers</a> •
  <a href="#contributing">Contributing</a>
</p>

---

## What Is This?

Nodum is a document reader (PDF, EPUB, web articles) with a live personal knowledge graph. As you read, you type rough thoughts into a sidebar. The LLM does three things:

1. **Checks your understanding** against the source text
2. **Surfaces connections** to concepts already in your graph
3. **Proposes a graph node** you can confirm with one click

The result is a visual, searchable map of *your* understanding — not the book's content, not the LLM's summary, but what *you* actually grasped, cross-referenced across every book you've ever read in the app.

## Quickstart

### Prerequisites

- **Rust** 1.78+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Node.js** 20+ (`nvm install 20`)
- **Docker** (for PostgreSQL with pgvector)
- An API key for at least one LLM provider (Claude, OpenAI, or Gemini)

### Setup

```bash
git clone https://github.com/nodum-app/nodum.git
cd nodum

# Install dependencies and create .env
make setup

# Edit .env — add your LLM API key
nano .env

# Start everything (DB + backend + frontend)
make dev
```

Open **http://localhost:5173** in your browser.

### With Docker Compose (alternative)

```bash
cp .env.example .env
# Edit .env with your API keys
docker compose up --build
```

Open **http://localhost:3000**.

## How It Works

### The Core Loop

```
You read a page
  → Something clicks (or doesn't)
  → You type a rough thought: "I think F=ma means heavier things
    need more force for the same acceleration?"
  → The LLM fires (2-3 seconds)
  → You get back:
      ✓ Accuracy check: "Correct — and the relationship is linear"
      🔗 Connections: Links to your existing [Newton's Laws] node
      📌 Node proposal: "F=ma — Newton's Second Law"
  → One click to add it to your graph
```

### The Key Insight

The LLM never sees the whole book. It only sees:
- Your rough thought (~50-200 tokens)
- The current page text (~600 tokens)
- Your 5 most relevant existing graph nodes (~300 tokens)

Total per request: ~1,200 tokens. Cheap, fast, and — most importantly — keeps the learning in *your* brain.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Frontend (React)                  │
│  ┌──────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │  Reader   │  │ Thinking Dock│  │  Force Graph  │ │
│  │ (PDF.js)  │  │ (LLM I/O)   │  │   (D3.js)     │ │
│  └──────────┘  └──────────────┘  └───────────────┘ │
└────────────────────────┬────────────────────────────┘
                         │ REST API
┌────────────────────────┴────────────────────────────┐
│               Backend (Rust / Axum)                  │
│  ┌──────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │  Reader   │  │ LLM Provider │  │   Graph       │ │
│  │  Engine   │  │  Abstraction │  │   Engine      │ │
│  │ (PDF/EPUB)│  │ Claude|GPT|  │  │ (Semantic     │ │
│  │          │  │ Gemini|Ollama│  │  Search)      │ │
│  └──────────┘  └──────────────┘  └───────────────┘ │
└────────────────────────┬────────────────────────────┘
                         │
          ┌──────────────┴──────────────┐
          │   PostgreSQL + pgvector      │
          │  ┌─────────┐ ┌───────────┐  │
          │  │  Graph   │ │  Vector   │  │
          │  │  Tables  │ │  Index    │  │
          │  └─────────┘ └───────────┘  │
          └─────────────────────────────┘
```

### Tech Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| Backend | **Rust** (Axum) | Performance, safety, single binary deployment |
| Frontend | **React** + TypeScript | Component model, ecosystem, D3.js integration |
| Database | **PostgreSQL** + pgvector | Graph queries + vector search in one DB |
| Graph Viz | **D3.js** force layout | Unmatched power for interactive graphs |
| PDF | pdf-extract (Rust) + PDF.js (frontend) | Native parsing + browser rendering |
| LLM | Multi-provider abstraction | Claude, GPT-4o, Gemini, Ollama |

## LLM Providers

Nodum supports multiple LLM providers. Configure them in **Settings** or via environment variables.

| Provider | Chat | Embeddings | Notes |
|----------|------|------------|-------|
| **Claude** (Anthropic) | ✅ Default | ❌ (use OpenAI) | Best reasoning quality |
| **OpenAI** (GPT-4o) | ✅ | ✅ Native | Best embedding support |
| **Gemini** (Google) | ✅ | ✅ Native | Good price/performance |
| **Ollama** (Local) | ✅ | ✅ | Free, private, offline |

### Using Your Own API Keys

Users configure API keys through the Settings page. Keys are stored encrypted in the database. Server-level keys in `.env` serve as fallbacks.

### Ollama (Fully Local)

For complete privacy — no data leaves your machine:

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh
ollama pull llama3.1

# In Nodum Settings, configure:
# Provider: Ollama
# Model: llama3.1
# Base URL: http://localhost:11434
```

## Project Structure

```
nodum/
├── backend/                 # Rust backend
│   ├── src/
│   │   ├── main.rs          # Server bootstrap
│   │   ├── config.rs        # Environment config
│   │   ├── api/mod.rs       # REST API routes + auth
│   │   ├── db/              # Database models + queries
│   │   ├── llm/             # Multi-provider LLM abstraction
│   │   │   ├── provider.rs  # Provider trait
│   │   │   ├── claude.rs    # Anthropic Claude
│   │   │   ├── openai.rs    # OpenAI / Ollama
│   │   │   └── gemini.rs    # Google Gemini
│   │   ├── reader/          # PDF/EPUB parsing + chunking
│   │   └── graph/           # Semantic search + graph ops
│   └── migrations/          # PostgreSQL schema
├── frontend/                # React + TypeScript
│   └── src/
│       ├── components/
│       │   ├── Reader/      # Document reading view
│       │   ├── ThinkingDock/ # Core thought input + LLM response
│       │   ├── Graph/       # D3.js force graph visualization
│       │   ├── Layout/      # App shell, library, login
│       │   └── Settings/    # LLM provider configuration
│       ├── stores/          # Zustand state management
│       ├── api/             # API client
│       └── types/           # TypeScript types
├── docker-compose.yml       # Full stack deployment
├── Makefile                 # Dev commands
└── .env.example             # Configuration template
```

## API Reference

### Auth
- `POST /api/auth/register` — Create account
- `POST /api/auth/login` — Get JWT token

### Books
- `GET /api/books` — List user's books
- `POST /api/books` — Upload PDF/EPUB (multipart)
- `GET /api/books/:id/chunks` — Get parsed text chunks
- `GET /api/books/:id/coverage` — Understanding coverage stats

### Core Loop
- `POST /api/thoughts` — Submit a rough thought → get accuracy check + connections + node proposal

### Knowledge Graph
- `POST /api/nodes` — Confirm a proposed node
- `GET /api/graph` — Full graph data (nodes + edges)
- `GET /api/graph/weak` — Weakly-connected nodes (fog of war)
- `POST /api/nodes/search` — Semantic search across all nodes
- `POST /api/edges` — Create manual connection

### Sessions
- `POST /api/sessions` — Start reading session
- `POST /api/sessions/:id/end` — End session with summary

### Settings
- `GET /api/providers` — List configured LLM providers
- `POST /api/providers` — Add/update provider config

## Development

```bash
# Run tests
make test

# Lint
make lint

# Reset database
make db-reset

# Open database shell
make db-shell

# Build for production
make build
```

## The System Prompt

The quality of Nodum's responses depends entirely on the system prompt. It lives in `backend/src/llm/mod.rs` as `SYSTEM_PROMPT`. Key design decisions:

- Structured JSON output (not freeform text)
- Confirms what's right before correcting what's wrong
- Never condescending
- Node labels use the reader's framing, not textbook definitions
- Confidence scoring reflects understanding quality

**If you're contributing, this is the single most impactful thing to improve.**

## Roadmap

- [x] PDF reader with clean typography
- [x] Thinking dock with LLM processing
- [x] Three-part response (accuracy/connections/node)
- [x] D3.js force-directed graph
- [x] Multi-LLM support (Claude, GPT, Gemini, Ollama)
- [x] Semantic search across nodes
- [x] JWT authentication
- [ ] EPUB rendering in browser
- [ ] Browser extension for web articles
- [ ] Cross-book connection detection (proactive)
- [ ] Spaced repetition for weak nodes
- [ ] Fog of war understanding map
- [ ] Export to Obsidian Markdown
- [ ] Collaborative graphs
- [ ] Mobile responsive reading mode
- [ ] Audio-to-thought (speech input)

## Contributing

Contributions welcome. The highest-impact areas:

1. **System prompt refinement** — Test against diverse reader thoughts
2. **PDF parsing edge cases** — Multi-column, scanned docs, footnotes
3. **Graph UX** — Clustering, filtering, level-of-detail rendering
4. **Mobile experience** — Responsive reader + floating dock

## License

MIT — see [LICENSE](LICENSE).

---

<p align="center">
  <em>"The product's core bet: the LLM never reads the book — you do."</em>
</p>
