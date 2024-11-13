FROM debian:bookworm-slim
ENV SHELL /bin/bash
ARG user_id=1000
ARG group_id=1000

# install packages
RUN apt-get update --fix-missing && DEBIAN_FRONTEND=noninteractive apt-get install --assume-yes --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    lsb-release \
    ca-certificates \
    curl \
    git-core \
    gnupg \
    nano \
    nodejs \
    npm \
    openssh-client \
    procps \
    python3 \
    python3-selenium \
    python3-watchdog \
    sudo \
    && rm -rf /var/lib/apt/lists/*

# install latest binaryen 116 (September 2023)
RUN curl -fsSL https://github.com/WebAssembly/binaryen/releases/download/version_116/binaryen-version_116-x86_64-linux.tar.gz | \
    tar xvz --strip-components 1 --directory /usr

# Create a developer user
RUN groupadd --gid $group_id developer && useradd --uid $user_id --gid $group_id -m developer
# Add developer user to the sudo file
RUN echo developer ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/developer && \
    chmod 0440 /etc/sudoers.d/developer
# Switch the developer user
USER developer

# install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --target wasm32-unknown-unknown
ENV PATH=/home/developer/.cargo/bin:$PATH
RUN cargo install --git https://github.com/rustwasm/wasm-bindgen.git  --tag 0.2.91 wasm-bindgen-cli
