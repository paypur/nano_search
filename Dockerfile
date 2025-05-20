# https://www.docker.com/blog/simplify-your-deployments-using-the-rust-official-image/
FROM rust:1.87.0 AS builder
WORKDIR /usr/src/nano_search/
COPY . .
RUN RUST_LOG=info cargo install --path .

# discard tooling
FROM archlinux:base
RUN pacman -Syu --noconfirm
COPY --from=builder /usr/local/cargo/bin/nano_search /usr/local/bin/nano_search

WORKDIR /data
CMD ["nano_search"]