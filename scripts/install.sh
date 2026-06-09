#!/usr/bin/env bash
# theta installer.
set -u

REPO_OWNER="tamarillo-ai"
REPO_NAME="theta"
REPO_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}"
RELEASES_URL="${REPO_URL}/releases"

VERSION="${THETA_VERSION:-latest}"
INSTALL_DIR="${THETA_INSTALL_DIR:-$HOME/.local/bin}"
FROM_SOURCE="false"

for arg in "$@"; do
    case "$arg" in
        --from-source) FROM_SOURCE="true" ;;
        *) echo "unknown argument: $arg" >&2; exit 64 ;;
    esac
done

TMP=""
cleanup() {
    if [ -n "$TMP" ] && [ -d "$TMP" ]; then
        rm -rf "$TMP"
    fi
}
trap cleanup EXIT

say()  { echo "$*"; }
err()  { echo "error: $*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || err "need '$1' (command not found)"; }

download() {
    local url="$1" out="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$out"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$out" "$url"
    else
        err "need curl or wget"
    fi
}

verify_sha256() {
    local file="$1" checksum_file="$2"
    if command -v shasum >/dev/null 2>&1; then
        (cd "$(dirname "$file")" && shasum -a 256 -c "$(basename "$checksum_file")" >/dev/null 2>&1)
    elif command -v sha256sum >/dev/null 2>&1; then
        (cd "$(dirname "$file")" && sha256sum -c "$(basename "$checksum_file")" >/dev/null 2>&1)
    else
        say "warning: no shasum or sha256sum command, skipping verification"
        return 0
    fi
}

detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"
    case "$os/$arch" in
        Linux/x86_64)              echo "x86_64-unknown-linux-gnu" ;;
        Linux/aarch64|Linux/arm64) echo "aarch64-unknown-linux-gnu" ;;
        Darwin/x86_64)             echo "x86_64-apple-darwin" ;;
        Darwin/arm64)              echo "aarch64-apple-darwin" ;;
        *) echo "" ;;
    esac
}

install_from_source() {
    say "installing theta from source via cargo install"
    need cargo
    need git
    TMP="$(mktemp -d)"
    local ref="main"
    if [ "$VERSION" != "latest" ]; then
        ref="$VERSION"
    fi
    git clone --quiet --depth 1 --branch "$ref" "${REPO_URL}.git" "$TMP/theta" \
        || err "git clone failed"
    cargo install --locked --path "$TMP/theta/crates/theta" --root "$TMP/cargo" \
        || err "cargo install failed"
    mkdir -p "$INSTALL_DIR" || err "could not create $INSTALL_DIR"
    install -m 0755 "$TMP/cargo/bin/theta" "$INSTALL_DIR/theta" \
        || err "could not install to $INSTALL_DIR"
    say "installed theta to $INSTALL_DIR/theta"
}

install_prebuilt() {
    local target="$1"
    local archive="theta-${target}.tar.gz"
    local checksum_file="${archive}.sha256"
    local base
    if [ "$VERSION" = "latest" ]; then
        base="${RELEASES_URL}/latest/download"
    else
        base="${RELEASES_URL}/download/${VERSION}"
    fi

    TMP="$(mktemp -d)"

    say "downloading ${archive}"
    if ! download "$base/$archive" "$TMP/$archive"; then
        return 1
    fi

    if download "$base/$checksum_file" "$TMP/$checksum_file" 2>/dev/null; then
        say "verifying checksum"
        verify_sha256 "$TMP/$archive" "$TMP/$checksum_file" \
            || err "checksum verification failed"
    else
        say "warning: no checksum published, skipping verification"
    fi

    tar -xzf "$TMP/$archive" -C "$TMP" || err "extraction failed"

    mkdir -p "$INSTALL_DIR" || err "could not create $INSTALL_DIR"
    install -m 0755 "$TMP/theta" "$INSTALL_DIR/theta" \
        || err "could not install to $INSTALL_DIR"
    say "installed theta to $INSTALL_DIR/theta"
}

path_check() {
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *) say "note: $INSTALL_DIR is not on your PATH" ;;
    esac
}

main() {
    if [ "$FROM_SOURCE" = "true" ]; then
        install_from_source
        path_check
        return
    fi

    local target
    target="$(detect_target)"
    if [ -z "$target" ]; then
        say "platform $(uname -s)/$(uname -m) has no prebuilt binary, falling back to cargo install"
        install_from_source
        path_check
        return
    fi

    if ! install_prebuilt "$target"; then
        say "prebuilt download failed, falling back to cargo install"
        install_from_source
    fi
    path_check
}

main "$@"
