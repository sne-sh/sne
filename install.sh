#!/bin/sh
# Installs sne on Linux and macOS without root.
#
#   curl -fsSL https://raw.githubusercontent.com/sne-sh/sne/main/install.sh | sh
#
# Optional settings, as environment variables:
#   SNE_VERSION         release tag to install, for example 0.1.0-beta.4 (default: latest)
#   SNE_INSTALL_DIR     where the sne binary goes (default: ~/.local/bin)
#   SNE_NO_PROFILE      set to 1 to leave your shell profile alone
#   SNE_NO_NOTIFY       set to 1 to not be notified when a new sne version is out

set -eu

fail() {
    echo "install.sh: $*" >&2
    exit 1
}

have() {
    command -v "$1" >/dev/null 2>&1
}

# download URL FILE
download() {
    if have curl; then
        curl -fsSL "$1" -o "$2"
    elif have wget; then
        wget -q "$1" -O "$2"
    else
        fail "curl or wget is needed"
    fi
}

sha256_of() {
    if have sha256sum; then
        sha256sum "$1" | cut -d' ' -f1
    elif have shasum; then
        shasum -a 256 "$1" | cut -d' ' -f1
    else
        fail "sha256sum or shasum is needed to verify the download"
    fi
}

# Adds the setup block to FILE once. $1 = file, $2 = block.
add_to_profile() {
    if [ -f "$1" ] && grep -q 'sne env' "$1"; then
        return 0
    fi
    mkdir -p "$(dirname "$1")"
    printf '\n%s\n' "$2" >>"$1"
    echo "Added the sne setup lines to $1"
}

main() {
    repo="sne-sh/sne"
    base="${SNE_DOWNLOAD_BASE:-https://github.com/$repo/releases/download}"
    base="${base%/}"
    install_dir="${SNE_INSTALL_DIR:-$HOME/.local/bin}"

    case "$(uname -m)" in
        x86_64 | amd64) cpu="x86_64" ;;
        arm64 | aarch64) cpu="aarch64" ;;
        *) fail "sne has no build for this processor ($(uname -m))" ;;
    esac
    case "$(uname -s)" in
        Linux)
            # The musl build is fully static, so it runs on every distribution.
            target="$cpu-unknown-linux-musl"
            ;;
        Darwin)
            # A shell running under Rosetta reports x86_64 on Apple silicon.
            if [ "$(sysctl -n sysctl.proc_translated 2>/dev/null || echo 0)" = "1" ]; then
                cpu="aarch64"
            fi
            target="$cpu-apple-darwin"
            ;;
        *) fail "this script supports Linux and macOS; on Windows use install.ps1" ;;
    esac

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT

    tag="${SNE_VERSION:-latest}"
    if [ "$tag" = "latest" ]; then
        download "https://api.github.com/repos/$repo/releases/latest" "$tmp/latest.json"
        tag="$(sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$tmp/latest.json" | head -n 1)"
        [ -n "$tag" ] || fail "could not find the latest sne release"
    fi

    asset="sne-$tag-$target.tar.gz"
    echo "Downloading sne $tag ($target)..."
    download "$base/$tag/$asset" "$tmp/$asset"
    download "$base/$tag/SHA256SUMS.txt" "$tmp/SHA256SUMS.txt"

    expected="$(awk -v name="$asset" '{ file = $2; sub(/^\*/, "", file); if (file == name) { print $1; exit } }' "$tmp/SHA256SUMS.txt")"
    [ -n "$expected" ] || fail "$asset is not listed in SHA256SUMS.txt"
    actual="$(sha256_of "$tmp/$asset")"
    [ "$actual" = "$expected" ] || fail "checksum mismatch for $asset; nothing was installed"

    tar -xzf "$tmp/$asset" -C "$tmp"
    [ -f "$tmp/sne-$tag-$target/sne" ] || fail "the sne binary was not found inside $asset"
    mkdir -p "$install_dir"
    # Copy then rename, so a running sne is replaced safely.
    cp "$tmp/sne-$tag-$target/sne" "$install_dir/.sne.new"
    chmod 755 "$install_dir/.sne.new"
    mv -f "$install_dir/.sne.new" "$install_dir/sne"
    echo "Installed $install_dir/sne"

    # Run it once before touching any profile: the system can refuse to start a program.
    "$install_dir/sne" --version >/dev/null 2>&1 ||
        fail "sne was installed but does not run on this system. Your shell profile was not changed. Try: $install_dir/sne --version"

    if [ "${SNE_NO_PROFILE:-0}" != "1" ]; then
        posix_block="# sne (Node.js version manager)
if [ -x \"$install_dir/sne\" ]; then
    case \":\$PATH:\" in *\":$install_dir:\"*) ;; *) export PATH=\"$install_dir:\$PATH\" ;; esac
    eval \"\$(sne env --shell SHELLNAME --use-on-cd)\"
fi"
        case "$(basename "${SHELL:-sh}")" in
            zsh)
                add_to_profile "${ZDOTDIR:-$HOME}/.zshrc" "$(echo "$posix_block" | sed 's/SHELLNAME/zsh/')"
                ;;
            fish)
                add_to_profile "${XDG_CONFIG_HOME:-$HOME/.config}/fish/conf.d/sne.fish" "# sne (Node.js version manager)
if test -x \"$install_dir/sne\"
    fish_add_path \"$install_dir\"
    sne env --shell fish --use-on-cd | source
end"
                ;;
            bash)
                # macOS terminals start login shells, which read .bash_profile and not .bashrc.
                if [ "$(uname -s)" = "Darwin" ]; then rc="$HOME/.bash_profile"; else rc="$HOME/.bashrc"; fi
                add_to_profile "$rc" "$(echo "$posix_block" | sed 's/SHELLNAME/bash/')"
                ;;
            *)
                echo "Your shell was not recognised. Add $install_dir to PATH and see the README for the setup line."
                ;;
        esac
    fi

    # On a desktop, sne turns new version notifications on by itself at the first `sne install`.
    if [ "${SNE_NO_NOTIFY:-0}" = "1" ]; then
        "$install_dir/sne" notify off >/dev/null 2>&1 || true
    fi

    echo
    echo "Done. Open a new terminal, then run:  sne install lts"
}

main "$@"
