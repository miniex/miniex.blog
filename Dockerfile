# Build stage
FROM rust:1.85.1 as builder
WORKDIR /usr/src/app

# Install Node.js, Bun, and other necessary tools
RUN apt-get update && apt-get install -y curl unzip && \
    curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && \
    apt-get install -y nodejs && \
    curl -fsSL https://bun.sh/install | bash && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.bun/bin:${PATH}"

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY package.json bun.lockb* ./
RUN bun install

# Build application
COPY . .
RUN bunx tailwindcss -i ./assets/styles/tailwind.input.css -o ./assets/styles/tailwind.output.css --minify
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
ENV TZ=UTC
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    sqlite3 \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/blog .
COPY --from=builder /usr/src/app/assets ./assets
COPY --from=builder /usr/src/app/templates ./templates
COPY --from=builder /usr/src/app/contents ./contents

# Create data directory for SQLite database with proper permissions
RUN mkdir -p /app/data && \
    chmod 755 /app/data && \
    chown -R root:root /app/data
ENV DATABASE_URL=sqlite:/app/data/blog.db

EXPOSE 80
CMD ["./blog"]
