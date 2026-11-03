# Official multi-architecture Rust image, pinned by manifest digest.
FROM rust:1.90.0-trixie@sha256:e227f20ec42af3ea9a3c9c1dd1b2012aa15f12279b5e9d5fb890ca1c2bb5726c AS toolchain
RUN apt-get update && apt-get install -y --no-install-recommends qemu-system-arm seabios python3 ca-certificates && rm -rf /var/lib/apt/lists/*
RUN rustup target add aarch64-unknown-none-softfloat && rustup component add rustfmt clippy
WORKDIR /rai
FROM toolchain AS demo
COPY . .
RUN python3 -B scripts/portable.py build
# The VNC listener is inside the container. Bind published host ports to loopback.
EXPOSE 5900
ENTRYPOINT ["python3", "-B", "scripts/portable.py"]
CMD ["verify"]
