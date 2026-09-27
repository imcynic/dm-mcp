# Build dm-mcp
FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release && strip target/release/dm-mcp

# Runtime: dm-mcp + BYOND Linux toolchain
FROM debian:bookworm-slim
ARG BYOND_MAJOR=516
ARG BYOND_MINOR=1688
RUN dpkg --add-architecture i386 \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates curl unzip \
        libc6:i386 zlib1g:i386 libstdc++6:i386 libgcc-s1:i386 libssl3:i386 libcurl4:i386 \
    && curl -fsSL "http://www.byond.com/download/build/${BYOND_MAJOR}/${BYOND_MAJOR}.${BYOND_MINOR}_byond_linux.zip" -o /tmp/byond.zip \
    && unzip -q /tmp/byond.zip -d /opt \
    && rm /tmp/byond.zip \
    && apt-get purge -y curl unzip && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*
ENV PATH="/opt/byond/bin:${PATH}" \
    LD_LIBRARY_PATH="/opt/byond/bin" \
    BYOND_SYSTEM="/opt/byond"
COPY --from=build /src/target/release/dm-mcp /usr/local/bin/dm-mcp
WORKDIR /workspace
ENTRYPOINT ["dm-mcp"]
