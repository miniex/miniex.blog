# miniex.blog

A personal blog built with Rust (Axum) + Askama + Tailwind CSS + SQLite.

Supports Korean, Japanese, and English with a markdown (MDX) based post system, comments, and a guestbook.

## Features

- **Post System** — Three categories: Blog, Review, Diary. Written in MDX (YAML front matter + Markdown) with auto-generated TOC, reading time estimation, and series support
- **i18n** — Korean, Japanese, English. Language determined by filename suffix (`slug.ko.mdx`). Detection order: Cookie → Accept-Language → default (en). Language fallback for post listings (shows available translation when preferred language is missing)
- **Comments & Guestbook** — SQLite-backed. Argon2 password hashing with transparent migration from legacy hashes
- **Search** — `/api/search` endpoint. Searches title, description, and tags. Open with `Ctrl+K` or `/`
- **Dark Mode** — DaisyUI pastel/pastel-dark themes. Persisted in localStorage. Flash-free on route change via blocking inline script
- **LaTeX Math** — Inline (`$...$`) and block (`$$...$$`) math rendering via KaTeX
- **Code Blocks** — Syntax highlighting via Highlight.js with copy-to-clipboard button
- **Graph Rendering** — `graph` fenced code block for mathematical function plotting via function-plot with interactive zoom/pan
- **Chart Rendering** — `chart` fenced code block for bar, line, pie, doughnut, and radar charts via Chart.js
- **Plot Rendering** — `plot3d` fenced code block with multiple visualization types via Plotly.js (see [Visualization DSL](#visualization-dsl) below)
- **Sort Toggle** — Ascending/descending sort on all list pages (blog, review, diary, series, guestbook) with htmx partial updates
- **Series** — Group related posts into a series with prev/next navigation, status tracking (Ongoing/Completed), and per-language navigation chains
- **Resume** — Dynamic resume page with hierarchical TOC, collapsible sections, and print-to-PDF optimization
- **SEO** — JSON-LD structured data, Open Graph tags, canonical URLs, hreflang alternate links, meta keywords, trailing slash redirect (301), XML sitemap with series pages
- **Performance** — Gzip/Brotli compression, Cache-Control headers for static assets, font preload, preconnect hints, deferred scripts, ETag conditional responses for feed/sitemap, image lazy loading
- **Security Headers** — Strict-Transport-Security (HSTS), X-Content-Type-Options, X-Frame-Options, Referrer-Policy, Content-Security-Policy
- **Rate Limiting** — tower_governor based rate limiting on write API endpoints (2/sec, burst 5)
- **Accessibility** — ARIA labels, keyboard navigation, skip-to-content link, passive event listeners, prefers-reduced-motion support
- **Atom Feed** — `/feed.xml` (20 recent posts, ETag support)
- **Sitemap** — `/sitemap.xml` (dynamically generated, includes series pages, ETag support)
- **Robots.txt** — `/robots.txt`
- **Custom 404** — Error page with navigation links to main sections

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust, Axum, Tokio, tower_governor |
| Templates | Askama |
| Styling | Tailwind CSS 3, DaisyUI, Phosphor Icons |
| Fonts | Nunito, Gowun Dodum (KO), Zen Maru Gothic (JA), JetBrains Mono |
| Frontend | HTMX, Highlight.js, KaTeX, function-plot, Chart.js, Plotly.js |
| Database | SQLite (sqlx), argon2 |
| Build | Cargo, Bun |
| Deploy | Docker, GitLab CI/CD |
| Dev Env | Nix Flake |

## Project Structure

```
src/
├── main.rs          # Entrypoint (server startup)
├── lib.rs           # Shared types (AppState, SharedState, constants)
├── router.rs        # Router assembly, middleware, live reload
├── handlers.rs      # Module declarations (handlers/)
├── handlers/
│   ├── pages.rs     # Page handlers (index, blog, review, diary, series, post, resume, guestbook, error)
│   ├── api.rs       # API handlers (search, language, comments, guestbook CRUD)
│   └── feed.rs      # Feed handlers (Atom feed, sitemap with ETag)
├── error.rs         # AppError type (NotFound, Database, Internal)
├── db.rs            # SQLite CRUD (comments, guestbook, argon2 hashing)
├── post.rs          # MDX loading, markdown parsing, TOC generation, image lazy loading
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
│   ├── graph-render.js    # Graph, chart, plot3d rendering (animated transforms)
│   ├── post-toc.js        # Post TOC (scroll tracking)
│   ├── resume-toc.js      # Resume TOC (collapsible h2 sections)
│   └── resume-print.js    # Print-to-PDF optimization
├── styles/
│   ├── tailwind.input.css  # Tailwind entry (imports + directives)
│   ├── tailwind.output.css # Compiled output (single bundle)
│   ├── base.css            # Base resets, selection, scrollbar, nav
│   ├── code.css            # Tables, code blocks, copy button, dark mode
│   ├── katex.css           # KaTeX math isolation
│   ├── charts.css          # Plotly, Chart.js, D3 graph styling
│   ├── hero.css            # Hero animations, reduced motion
│   ├── animations.css      # Page transitions, staggered animations
│   └── print.css           # Print media styles
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

## Visualization DSL

Visualizations are embedded in MDX posts using fenced code blocks. Three renderers are available:

| Block | Renderer | Use case |
|-------|----------|----------|
| `` ```graph `` | function-plot | 2D math functions (`y = f(x)`) |
| `` ```chart `` | Chart.js | Bar, line, pie, doughnut, radar charts |
| `` ```plot3d `` | Plotly.js | Vectors, points, transforms, 3D surfaces/scatter |

### `graph` — Function Plot

```
title: Quadratic
x: -5, 5
y: -2, 30
fn: x^2 | | x^2
fn: 2*x + 1 | steelblue | 2x + 1
```

- `fn`: expression `| color (optional) | KaTeX label (optional)`
- `x`, `y`: axis range
- `xlabel`, `ylabel`: axis labels

### `chart` — Chart.js

```
type: bar
title: Monthly Sales
labels: Jan, Feb, Mar, Apr
dataset: Revenue | 10, 25, 15, 30
dataset: Cost | 5, 10, 8, 12 | tomato
```

- `type`: `line`, `bar`, `pie`, `doughnut`, `radar`
- `dataset`: `label | values | color (optional)`

### `plot3d` — Plotly Types

All `plot3d` blocks require a `type` field. Available types:

#### `vector2d` — 2D Vector Arrows

```
type: vector2d
title: Basis vectors
x: -1, 4
y: -1, 4
vec: 1, 0 | | \hat{i}
vec: 0, 1 | | \hat{j}
vec: 2, 3 | | \vec{v} = 2\hat{i} + 3\hat{j}
```

- `vec`: `vx, vy | color (optional) | KaTeX label (optional)`
- Renders arrows from origin with arrowheads

#### `point2d` — 2D Scatter Points

```
type: point2d
title: Lattice points
x: -4, 6
y: -6, 6
point: 0, 0 | | \vec{0}
point: 2, 3 | | \vec{v}
point: 1, 2 | | \vec{w}
```

- `point`: `px, py | color (optional) | KaTeX label (optional)`

#### `transform2d` — Animated 2D Linear Transform

```
type: transform2d
title: 90° rotation
matrix: 0, -1, 1, 0
grid: -2, 2
step: 1
```

- `matrix`: `a, b, c, d` — the 2x2 matrix `[[a,b],[c,d]]`
- `grid`: range for grid points
- `step`: grid spacing
- Renders an animated 2D grid that deforms from the original to the transformed state
- Auto-plays on scroll into view (IntersectionObserver), ping-pong loop with 8x fast reverse
- Pause/play button; pauses when scrolled off-screen

#### `compose2d` — Animated Two-Step Composition

```
type: compose2d
title: SR composition — rotate then shear
matrix1: 0, -1, 1, 0
matrix2: 1, 1, 0, 1
grid: -2, 2
step: 1
```

- `matrix1`: first transform (applied first)
- `matrix2`: second transform (applied to result of matrix1)
- Two-phase animation: original → after M₁ (pause) → after M₂·M₁
- Same auto-play/loop behavior as `transform2d`

#### `vector3d` — 3D Vector Arrows

```
type: vector3d
vec: 2, 1, 0 | | \vec{v}
vec: 0, 2, 1 | | \vec{w}
```

- `vec`: `vx, vy, vz | color (optional) | KaTeX label (optional)`
- Renders 3D lines with cone arrowheads

#### `scatter3d` — 3D Scatter Plot

```
type: scatter3d
dataset: Group A | 1,2,3; 4,5,6; 7,8,9
dataset: Group B | 2,3,1; 5,6,4 | tomato
```

- `dataset`: `label | x,y,z points separated by ; | color (optional)`

#### `surface` — 3D Surface Plot

```
type: surface
title: Paraboloid
x: -3, 3
y: -3, 3
fn: x^2 + y^2
```

- `fn`: expression in `x` and `y`
- Supports: `sin`, `cos`, `tan`, `sqrt`, `exp`, `log`, `abs`, `pow`, `PI`

### Shared DSL Features

- Colors: CSS named colors (`steelblue`, `tomato`) or hex (`#c9899e`)
- Labels with `|` delimiters: `data | color | KaTeX label`
- Empty color field inherits from the pastel palette: `data | | label`
- All plots support `title` field
- Light/dark theme auto-switch on `data-theme` change
- Responsive: mobile-optimized dimensions and font sizes

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
