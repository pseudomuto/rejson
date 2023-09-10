# Stage 1: Build the binary
FROM rust:alpine3.18 as build
RUN apk --update --no-cache add ca-certificates=20230506-r0 musl-dev=1.2.4-r1
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
