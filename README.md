# Nodum — The Knowledge-Connecting Book Reader

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=waving&height=220&color=0:1f2937,100:4f46e5&text=Nodum&fontColor=ffffff&fontAlignY=38&desc=Read%20deeply.%20Think%20clearly.%20Connect%20knowledge.&descAlignY=58&animation=fadeIn" alt="Nodum banner" />
</p>

<p align="center">
  <a href="https://git.io/typing-svg">
    <img src="https://readme-typing-svg.herokuapp.com?font=Inter&weight=500&size=22&duration=2800&pause=900&color=4F46E5&center=true&vCenter=true&width=900&lines=Read+books+with+focus;Capture+rough+thoughts+as+you+learn;Build+a+living+knowledge+graph;The+LLM+checks+understanding+%E2%80%94+you+do+the+reading" alt="Typing animation" />
  </a>
</p>

<p align="center">
  <strong>The LLM never reads the book — you do.</strong><br/>
  It verifies understanding, surfaces connections, and helps grow your personal knowledge graph.
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#tech-stack">Tech Stack</a> •
  <a href="#roadmap">Roadmap</a> •
  <a href="#contributing">Contributing</a>
</p>

---

## Badges

<p>
  <img src="https://img.shields.io/badge/Backend-Rust-000000?style=for-the-badge&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/Frontend-React-20232A?style=for-the-badge&logo=react" alt="React" />
  <img src="https://img.shields.io/badge/Language-TypeScript-3178C6?style=for-the-badge&logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/API-Axum-6B21A8?style=for-the-badge" alt="Axum" />
  <img src="https://img.shields.io/badge/Database-PostgreSQL-336791?style=for-the-badge&logo=postgresql&logoColor=white" alt="PostgreSQL" />
  <img src="https://img.shields.io/badge/Vector-pgvector-0F172A?style=for-the-badge" alt="pgvector" />
  <img src="https://img.shields.io/badge/Graph-D3.js-F9A03C?style=for-the-badge&logo=d3.js&logoColor=white" alt="D3.js" />
  <img src="https://img.shields.io/badge/LLM-Multi--Provider-111827?style=for-the-badge" alt="Multi Provider LLM" />
  <img src="https://img.shields.io/badge/License-MIT-16A34A?style=for-the-badge" alt="MIT License" />
</p>

<p>
  <img src="https://img.shields.io/badge/Status-Active%20Development-2563EB?style=flat-square" alt="Status" />
  <img src="https://img.shields.io/badge/PRs-Welcome-10B981?style=flat-square" alt="PRs Welcome" />
  <img src="https://img.shields.io/badge/Contributions-Welcome-0EA5E9?style=flat-square" alt="Contributions Welcome" />
</p>

---

## About

Nodum is a reader for PDFs, EPUBs, and web content that turns active reading into a connected learning system.

As you read, you write rough thoughts. Nodum’s LLM layer then:

- Validates your understanding against the current source context
- Finds meaningful links to concepts already in your graph
- Suggests a node you can confirm in one click

You get a living, searchable map of **your understanding** — built from what you think, not from generic summaries.

---

## Features

- **Understanding-first feedback** with fast, structured LLM responses
- **Live thought capture** in a focused reading sidebar
- **Personal knowledge graph** with visual node/edge exploration
- **Semantic node search** across your entire reading history
- **Multi-provider LLM support** (Claude, OpenAI, Gemini, Ollama)
- **Local-first privacy option** via Ollama
- **JWT auth + user sessions** for secure personalized workflows
- **Cross-document learning memory** to reinforce long-term retention

---

## Demo / Preview

<p align="center">
  <img src="frontend/public/nodum-icon.svg" width="90" alt="Nodum icon" />
</p>

> Add product screenshots or a short GIF here (e.g., `docs/preview.gif`) to showcase the reader, thinking dock, and graph view.

---

## Tech Stack

| Layer | Technologies |
|---|---|
| Backend | Rust, Axum |
| Frontend | React, TypeScript, Zustand |
| Database | PostgreSQL, pgvector |
| Visualization | D3.js |
| Reader Engine | PDF.js (frontend), Rust parser backend |
| LLM Layer | Claude, GPT-4o, Gemini, Ollama |
| DevOps | Docker, Docker Compose, Makefile |

---

## Installation

### Prerequisites

- Rust **1.78+**
- Node.js **20+**
- Docker (for PostgreSQL + pgvector)
- API key for at least one LLM provider

### Local Development

```bash
git clone https://github.com/nodum-app/nodum.git
cd nodum

make setup
# Update .env with your API keys
make dev
```

Open: **http://localhost:5173**

### Docker Compose

```bash
cp .env.example .env
# Update .env with your API keys
docker compose up --build
```

Open: **http://localhost:3000**

---

## Usage

```bash
# Run tests
make test

# Lint
make lint

# Reset database
make db-reset

# Open PostgreSQL shell
make db-shell

# Build production artifacts
make build
```

### Core API Endpoints

- `POST /api/auth/register` — Register account  
- `POST /api/auth/login` — Login + JWT  
- `GET /api/books` — List books  
- `POST /api/books` — Upload PDF/EPUB  
- `POST /api/thoughts` — Analyze rough thought  
- `GET /api/graph` — Fetch full knowledge graph  
- `POST /api/nodes` — Confirm proposed node  

---

## Project Structure

```text
nodum/
├── backend/
│   ├── src/
│   │   ├── api/
│   │   ├── db/
│   │   ├── llm/
│   │   ├── reader/
│   │   ├── graph/
│   │   └── main.rs
│   └── migrations/
├── frontend/
│   └── src/
│       ├── components/
│       ├── stores/
│       ├── api/
│       └── types/
├── docker-compose.yml
├── Makefile
└── .env.example
```

---

## Roadmap

- [x] PDF reader and thought capture flow
- [x] Three-part LLM response (accuracy, connections, node)
- [x] D3 force-directed graph
- [x] Multi-provider LLM integration
- [x] Semantic search across nodes
- [ ] EPUB rendering in-browser
- [ ] Browser extension for web articles
- [ ] Spaced repetition for weak nodes
- [ ] Obsidian Markdown export
- [ ] Mobile-optimized reading mode

---

## GitHub Stats

<p align="center">
  <img height="165" src="https://github-readme-stats.vercel.app/api?username=kishore-1404&show_icons=true&hide_border=true&theme=transparent" alt="GitHub stats" />
  <img height="165" src="https://github-readme-streak-stats.herokuapp.com/?user=kishore-1404&hide_border=true&theme=transparent" alt="GitHub streak" />
</p>

---

## Contributing

Contributions are welcome and appreciated.

1. Fork the repository  
2. Create a feature branch  
3. Commit your changes  
4. Open a pull request  

For highest impact, focus on:

- System prompt quality and evaluation
- PDF parsing edge cases
- Graph UX and interaction quality
- Mobile reading experience

---

## License

This project is licensed under the **MIT License**.  
See [LICENSE](LICENSE) for details.