# https://www.docker.com/blog/simplify-your-deployments-using-the-rust-official-image/
FROM rust:1.87.0 AS builder
WORKDIR /usr/src/nano_search/
COPY . .
RUN RUST_LOG=info cargo install --path .

# discard tooling
FROM debian:buster-slim
RUN apt-get update &amp;amp;amp; apt-get install -y extra-runtime-dependencies &amp;amp;amp; rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/nano_search /usr/local/bin/nano_search
CMD ["nano_search"]