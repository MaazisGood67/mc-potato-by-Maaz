#!/bin/bash
set -e

echo "🥔 MaazX Potato Edition — Setting up dev environment..."

# ── Rust toolchain ──────────────────────────────────────────────
echo "📦 Installing Rust components..."
rustup component add clippy rustfmt rust-analyzer
rustup target add x86_64-unknown-linux-gnu

# ── Fork rustcraft from GitHub ───────────────────────────────────
if [ ! -d "rustcraft-src" ]; then
  echo "🍴 Forking EduRenesto/rustcraft..."
  git clone https://github.com/EduRenesto/rustcraft.git rustcraft-src
  echo "✅ Cloned rustcraft source"
fi

# ── Install system deps for OpenGL/windowing ─────────────────────
echo "🖥  Installing OpenGL + window system deps..."
sudo apt-get update -qq
sudo apt-get install -y -qq \
  libgl1-mesa-dev \
  libglu1-mesa-dev \
  libx11-dev \
  libxi-dev \
  libxrandr-dev \
  libxcursor-dev \
  libasound2-dev \
  pkg-config \
  cmake \
  build-essential \
  libssl-dev \
  mesa-utils

# ── Node for the HTML launcher ────────────────────────────────────
echo "🌐 Setting up launcher dev server..."
npm install -g live-server 2>/dev/null || true

# ── Done ──────────────────────────────────────────────────────────
echo ""
echo "✅ MaazX dev environment ready!"
echo ""
echo "  📁 rustcraft-src/  ← base engine to modify"
echo "  📁 engine/         ← our potato-optimized fork"
echo "  📁 launcher/       ← HTML offline launcher"
echo ""
echo "  🚀 cargo run --release    (in engine/)"
echo "  🌐 live-server launcher/  (launcher UI)"
echo ""
echo "  🤖 GPT-6 Sol is your coding agent in the sidebar"
echo "  📋 See docs/CLAUDE_DIRECTOR.md for the full plan"
echo ""
echo "🥔 Let's build something no one has done before."
