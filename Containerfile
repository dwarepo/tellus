## Builder
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=demo
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /demo

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

## Final image
FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /demo

# Copy our build
COPY --from=builder /demo/target/x86_64-unknown-linux-musl/release/graph ./
# Add test networks
ADD ./test_networks.csv /demo/test_networks.csv
# Use an unprivileged user.
USER demo:demo

CMD ["/demo/graph", "WOB1", "Trumpf", "bandwidth"]
