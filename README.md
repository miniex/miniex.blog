# miniex.blog

A personal blog built with Rust (Axum) + Askama + Tailwind CSS + SQLite.

Supports Korean, Japanese, and English with a markdown (MDX) based post system, comments, and a guestbook.

## Features

- **Post System** — Three categories: Blog, Review, Diary. Written in MDX (YAML front matter + Markdown) with auto-generated TOC, reading time estimation, and series support
- **i18n** — Korean, Japanese, English. Language determined by filename suffix (`slug.ko.mdx`). Detection order: Cookie → Accept-Language → default (en). Language fallback for post listings (shows available translation when preferred language is missing)
- **Comments & Guestbook** — SQLite-backed. Password-protected edit and delete
- **Search** — `/api/search` endpoint. Searches title, description, and tags. Open with `Ctrl+K` or `/`
- **Dark Mode** — DaisyUI pastel/pastel-dark themes. Persisted in localStorage. Flash-free on route change via blocking inline script
- **LaTeX Math** — Inline (`$...$`) and block (`$$...$$`) math rendering via KaTeX
- **Code Blocks** — Syntax highlighting via Highlight.js with copy-to-clipboard button
- **Graph Rendering** — `graph` fenced code block for mathematical function plotting via function-plot. Supports `point2d` and `transform2d` plot types with interactive zoom/pan
- **Chart Rendering** — `chart` fenced code block for bar, line, pie, doughnut, and radar charts via Chart.js
- **3D Plot Rendering** — `plot3d` fenced code block for 3D surfaces, vector fields, and scatter plots via Plotly.js
- **Sort Toggle** — Ascending/descending sort on all list pages (blog, review, diary, series, guestbook) with htmx partial updates
- **Series** — Group related posts into a series with prev/next navigation, status tracking (Ongoing/Completed), and per-language navigation chains
- **Resume** — Dynamic resume page with hierarchical TOC, collapsible sections, and print-to-PDF optimization
- **SEO** — JSON-LD structured data, Open Graph tags, canonical URLs, hreflang alternate links, meta keywords, trailing slash redirect (301), XML sitemap with series pages
- **Performance** — Gzip/Brotli compression, Cache-Control headers for static assets, font preload, preconnect hints, deferred scripts
- **Security Headers** — Strict-Transport-Security (HSTS), X-Content-Type-Options, X-Frame-Options, Referrer-Policy
- **Accessibility** — ARIA labels, keyboard navigation, skip-to-content link, passive event listeners, prefers-reduced-motion support
- **Atom Feed** — `/feed.xml` (20 recent posts)
- **Sitemap** — `/sitemap.xml` (dynamically generated, includes series pages)
- **Robots.txt** — `/robots.txt`
- **Custom 404** — Error page with navigation links to main sections

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust, Axum, Tokio |
| Templates | Askama |
| Styling | Tailwind CSS 3, DaisyUI, Phosphor Icons |
| Fonts | Nunito, Gowun Dodum (KO), Zen Maru Gothic (JA), JetBrains Mono |
| Frontend | HTMX, Highlight.js, KaTeX, function-plot, Chart.js, Plotly.js |
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
├── i18n.rs          # Translations (80+ keys x 3 languages)
└── templates.rs     # Template definitions

templates/
├── _base.html       # Layout (header, footer, search modal)
├── _components.html # post_card macro
├── index.html       # Homepage (recent posts)
├── blog.html        # Blog list (category filter, sort, pagination)
├── review.html      # Review list (category filter, sort, pagination)
├── diary.html       # Diary list (category filter, sort, pagination)
├── post.html        # Post detail (comments, TOC, series nav)
├── series.html      # Series list (sort by updated_at)
├── series_detail.html # Series detail (timeline, sort)
├── guestbook.html   # Guestbook (sort)
├── resume.html      # Resume (hierarchical TOC, print)
└── error.html       # 404 with navigation links

assets/
├── js/
│   ├── search.js          # Search modal (Ctrl+K, language filter)
│   ├── code-highlight.js  # Syntax highlighting + copy button
│   ├── graph-render.js    # Graph, chart, plot3d rendering
│   ├── post-toc.js        # Post TOC (scroll tracking)
│   ├── resume-toc.js      # Resume TOC (collapsible h2 sections)
│   └── resume-print.js    # Print-to-PDF optimization
├── styles/
│   ├── tailwind.input.css # Tailwind source
│   ├── tailwind.output.css# Compiled output
│   ├── global.css         # Custom styles (tables, code blocks, graphs, hero animations)
│   └── print.css          # Print media styles
├── favicon/               # Sakura flower icons
└── robots.txt             # Crawler rules

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
series_order: 1
series_description: "A series about..."
series_status: "ongoing"
---
```

## Routes

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Homepage |
| GET | `/blog` | Blog listing |
| GET | `/review` | Review listing |
| GET | `/diary` | Diary listing |
| GET | `/series` | Series listing |
| GET | `/series/:name` | Series detail |
| GET | `/post/:slug` | Post detail |
| GET | `/guestbook` | Guestbook |
| GET | `/feed.xml` | Atom feed |
| GET | `/sitemap.xml` | Sitemap |
| GET | `/robots.txt` | Robots.txt |
| GET | `/api/search` | Search API |
| GET | `/api/set-lang` | Set language cookie |
| GET/POST/PUT/DELETE | `/api/comments/*` | Comments CRUD |
| GET/POST/PUT/DELETE | `/api/guestbook/*` | Guestbook CRUD |

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
./scripts/dev.sh     # Runs Tailwind watch + cargo watch in tmux (auto port cleanup on exit)
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

**The following are personal to this site and must not be reused:**
- All content under `contents/` (blog posts, reviews, diary entries, resume)
- Author name and profile (`Han Damin`, `miniex`)
- Domain names (`miniex.blog`, `miniex.info`, `daminstudio.com`)
- IndexNow API key (`assets/indexnow-key.txt`)
- Resume secret tag (`RESUME_TAG`)
- GitLab/GitHub repository URLs (`Cargo.toml`, `.gitlab-ci.yml`)
- Deploy infrastructure (server paths, container/network names)
- Favicon and profile images
