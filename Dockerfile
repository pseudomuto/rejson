# Stage 1: Build the binary
FROM rust:1.91.1-alpine AS build
RUN apk --update --no-cache add ca-certificates=20250911-r0 musl-dev=1.2.5-r10
WORKDIR /app
COPY Cargo.toml README.md ./
COPY src/ ./src/
RUN cargo build --release

# Final stage: Make the binary available
FROM scratch
WORKDIR /app
COPY --from=build /app/target/release/rejson .
VOLUME [ "/files" ]
VOLUME [ "/keys" ]
ENV EJSON_KEYDIR=/keys
ENTRYPOINT [ "/app/rejson" ]
