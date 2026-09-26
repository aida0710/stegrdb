FROM rust:1.98-slim-bookworm
WORKDIR /usr/src/app
COPY . .
ENV RUST_BACKTRACE=1
CMD ["cargo", "test", "--workspace", "--locked"]
