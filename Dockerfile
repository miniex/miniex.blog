# Build stage
FROM rust:latest as builder
WORKDIR /usr/src/app

RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs

RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:${PATH}"

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY package.json bun.lockb* ./
RUN bun install

COPY . .

RUN bunx tailwindcss -i ./assets/styles/tailwind.input.css -o ./assets/styles/tailwind.output.css --minify
RUN cargo build --release

# Runtime stage
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive

ENV TZ=UTC
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/blog .
COPY --from=builder /usr/src/app/assets ./assets
COPY --from=builder /usr/src/app/templates ./templates
COPY --from=builder /usr/src/app/contents ./contents

EXPOSE 80

CMD ["blog"]
