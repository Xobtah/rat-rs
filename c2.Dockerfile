#################################################################################
## Builder
#################################################################################
FROM rust:latest AS builder
WORKDIR /app
COPY ./ .
RUN cargo build --manifest-path=c2/Cargo.toml --release

#################################################################################
## Final image
#################################################################################
FROM debian:bullseye

# Create unprivileged user
ENV USER=c2
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /app

# Copy our build
COPY --from=builder /app/c2/target/release/c2 ./

# Use an unprivileged user
USER c2:c2

CMD ["/app/c2"]
EXPOSE 443
