# miniex.blog

A personal blog built with Rust (Axum) + Askama + Tailwind CSS + SQLite.

Supports Korean, Japanese, and English with a markdown (MDX) based post system, comments, and a guestbook.

## Features

- **Post System** — Three categories: Blog, Review, Diary. Written in MDX (YAML front matter + Markdown) with auto-generated TOC, reading time estimation, and series support
- **i18n** — Korean, Japanese, English. Language determined by filename suffix (`slug.ko.mdx`). Detection order: Cookie → Accept-Language → default (en)
- **Comments & Guestbook** — SQLite-backed. Password-protected edit and delete
- **Search** — `/api/search` endpoint. Searches title, description, and tags. Open with Ctrl+K
- **Dark Mode** — DaisyUI pastel/pastel-dark themes. Persisted in localStorage
- **LaTeX Math** — Inline (`$...$`) and block (`$$...$$`) math rendering via KaTeX
- **Atom Feed** — `/feed.xml`
- **Sitemap** — `/sitemap.xml` (dynamically generated)
- **Robots.txt** — `/robots.txt`
- **Series** — Group related posts into a series with prev/next navigation

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust, Axum, Tokio |
| Templates | Askama |
| Styling | Tailwind CSS 3, DaisyUI, Phosphor Icons |
| Frontend | HTMX, Highlight.js, KaTeX, Vanilla JS |
| Database | SQLite (sqlx) |
| Build | Cargo, Bun |
| Deploy | Docker, GitLab CI/CD |
| Dev Env | Nix Flake |

## Project Structure

```
src/
├── main.rs          # Router, handlers, entrypoint
├── lib.rs           # Shared types (AppState, SharedState)
├── db.rs            # SQLite CRUD (comments, guestbook)
├── post.rs          # MDX loading, markdown parsing, TOC generation
├── post/de.rs       # DateTime serialization
├── filters.rs       # Askama template filters
├── i18n.rs          # Translations (50+ keys x 3 languages)
└── templates.rs     # Template definitions

templates/
├── _base.html       # Layout (header, footer, search modal)
├── _components.html # post_card macro
├── index.html       # Homepage
├── blog.html        # Blog list
├── review.html      # Review list
├── diary.html       # Diary list
├── post.html        # Post detail (comments, TOC)
├── series.html      # Series list
├── series_detail.html
├── guestbook.html   # Guestbook
├── resume.html      # Resume
└── error.html       # 404

assets/
├── js/              # search, code-highlight, post-toc, resume-*
├── styles/          # tailwind input/output, global.css, print.css
├── favicon/         # Sakura flower icons
└── robots.txt       # Crawler rules

contents/
├── blog/            # Blog posts (*.mdx)
├── review/          # Review posts
└── diary/           # Diary entries
```

## Content Format

Create MDX files under `contents/{blog,review,diary}/`. Language is determined by the filename suffix.

```
contents/blog/my-post.ko.mdx
contents/blog/my-post.ja.mdx
contents/blog/my-post.en.mdx
```

Front matter example:

```yaml
---
title: "Post Title"
description: "A short description"
author: "miniex"
tags: ["rust", "web"]
created_at: "2025/01/15 12:00"
updated_at: "2025/01/16 12:00"
series: "My Series"
---
```

## Getting Started

### Prerequisites

- Rust (stable)
- Bun
- SQLite

### With Nix

```bash
nix develop          # Enter devShell
bun install          # Install node dependencies
bun dev &            # Tailwind CSS watch
cargo run            # Start server (localhost:3000)
```

### Without Nix

```bash
# Build Tailwind CSS
bun install
bun run build

# Build & run Rust server
cargo run
```

### Development (tmux)

```bash
./scripts/dev.sh     # Runs Tailwind watch + cargo watch in tmux
```

### Docker

```bash
docker compose up --build
# localhost:1380
```

### Nix Build

```bash
nix build            # result/bin/blog
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level | — |
| `DATABASE_URL` | SQLite database path | `sqlite:./data/blog.db` |
| `RESUME_TAG` | Resume route path | — |
| `RESUME_TITLE` | Resume page title | — |

## License

Code is licensed under MIT OR Apache-2.0.

All content under `contents/` is copyrighted by the author. All rights reserved.
