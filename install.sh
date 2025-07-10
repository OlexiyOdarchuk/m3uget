#!/usr/bin/env bash
set -e

REPO_URL="https://github.com/OlexiyOdarchuk/m3uget.git"
INSTALL_DIR="$HOME/.local/src/m3uget"

# --- helpers -------------------------------------------------------
log() { printf "\e[1;34m%s\e[0m\n" "$*"; }
need() { command -v "$1" >/dev/null 2>&1; }
pkg_install() {
  if need pacman; then sudo pacman -Sy --noconfirm "$@"
  elif need apt-get;  then sudo apt-get update && sudo apt-get install -y "$@"
  elif need brew;     then brew install "$@"
  else printf "❌ Package manager not supported. Install %s manually.\n" "$*" && exit 1
  fi
}

# --- 1. Rust -------------------------------------------------------
if ! need cargo; then
  log "Installing Rust toolchain..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
fi

# --- 2. yt-dlp & ffmpeg -------------------------------------------
for pkg in yt-dlp ffmpeg; do
  if ! need "$pkg"; then
    log "Installing $pkg..."
    pkg_install "$pkg"
  fi
done

# --- 3. Clone / pull source ---------------------------------------
log "Fetching source..."
if [[ -d $INSTALL_DIR/.git ]]; then
  git -C "$INSTALL_DIR" pull --quiet
else
  git clone --depth 1 "$REPO_URL" "$INSTALL_DIR"
fi

# --- 4. Build ------------------------------------------------------
log "Building m3uget (release)..."
cargo build --release --manifest-path "$INSTALL_DIR/Cargo.toml"

BIN="$INSTALL_DIR/target/release/m3uget"
DEST="/usr/local/bin/m3uget"

sudo cp "$BIN" "$DEST"
sudo chmod +x "$DEST"

log "✅ Installed → $DEST"
echo "Run: m3uget --help"
