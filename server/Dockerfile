# The API image. Built in CI by deploy-api.yml, never on Render (TS §3).
# The build context is the repository root:
#
#   docker build -f server/Dockerfile --build-arg GIT_SHA=$(git rev-parse HEAD) -t offcut-api:local .
#
# Both stages are the same Debian release, so the binary finds the C library
# it was linked against. Each image is pinned by digest; the tag is there for
# a person to read. When rust-toolchain.toml changes, change the first one.

# --- Build ---------------------------------------------------------------------
FROM rust:1.99.0-slim-trixie@sha256:24e632c09342c20abf8312cf4f61430a911c01ed3a5e4c02b87292b1c39c5273 AS build

WORKDIR /build

# The queries are checked against server/.sqlx/, so no database is needed here.
ENV SQLX_OFFLINE=true

# Cargo needs every workspace member to read the workspace, also the ones this
# image does not build.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
COPY server/ server/

RUN cargo build --release --locked -p offcut-api

# --- Runtime -------------------------------------------------------------------
FROM debian:13.7-slim@sha256:a29215f6a35e51e22adffa17f89e9d2ef06214e64a2bad10d765c46aea49f11f

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin offcut

COPY --from=build /build/target/release/offcut-api /usr/local/bin/offcut-api

USER 10001

# The commit this image was built from; /api/v1/healthz reports it (D-14).
# Declared last, and only in this stage: a new commit with unchanged Rust
# sources then reuses every layer above.
ARG GIT_SHA=dev
ENV GIT_SHA=$GIT_SHA

# The port comes from PORT, which Render sets. Every other variable of
# v1implementation.md §3.3 is entered in Render; none is in this image.
CMD ["offcut-api"]
