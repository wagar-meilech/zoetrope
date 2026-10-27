#!/bin/sh
# Build this checkout and install its compiled command, without changing Git state.
set -eu

bin_dir=
while [ "$#" -gt 0 ]; do
    case "$1" in
        --bin-dir)
            [ "$#" -ge 2 ] && [ -n "$2" ] || { printf '%s\n' '--bin-dir requires a directory' >&2; exit 2; }
            bin_dir=$2
            shift 2
            ;;
        --help|-h)
            printf '%s\n' 'Usage: scripts/install.sh [--bin-dir DIR]' 'Default destination: $HOME/.local/bin'
            exit 0
            ;;
        *) printf 'Unknown option: %s\n' "$1" >&2; exit 2 ;;
    esac
done
if [ -z "$bin_dir" ]; then
    : "${HOME:?Set HOME or pass --bin-dir DIR}"
    bin_dir=$HOME/.local/bin
fi
checkout=$(CDPATH= cd "$(dirname "$0")/.." && pwd -P)
target_dir=${CARGO_TARGET_DIR:-"$checkout/target"}
case "$target_dir" in /*) ;; *) target_dir=$checkout/$target_dir ;; esac
(
    cd "$checkout"
    cargo build --release --locked --bin zoe --target-dir "$target_dir"
)
mkdir -p "$bin_dir"
bin_dir=$(CDPATH= cd "$bin_dir" && pwd -P)
for command in zoe zoe-update; do
    if [ -d "$bin_dir/$command" ]; then
        printf 'Cannot replace a directory: %s\n' "$bin_dir/$command" >&2
        exit 1
    fi
done
staging=$(mktemp -d "$bin_dir/.zoe-install.XXXXXX")
trap 'rm -rf "$staging"' 0
trap 'exit 1' HUP INT TERM
cp "$target_dir/release/zoe" "$staging/zoe"
shell_quote() {
    printf "'"
    printf '%s' "$1" | sed "s/'/'\\\\''/g"
    printf "'"
}
{
    printf '%s\n' '#!/bin/sh' 'set -eu'
    printf 'checkout=%s\n' "$(shell_quote "$checkout")"
    printf 'bin_dir=%s\n' "$(shell_quote "$bin_dir")"
    cat <<'UPDATER'
if [ ! -x "$checkout/scripts/install.sh" ] || [ ! -f "$checkout/Cargo.toml" ]; then
    printf 'Source checkout is missing: %s\nRun scripts/install.sh from its new location to reinstall into %s.\n' "$checkout" "$bin_dir" >&2
    exit 1
fi
exec "$checkout/scripts/install.sh" --bin-dir "$bin_dir" "$@"
UPDATER
} > "$staging/zoe-update"
chmod 755 "$staging/zoe" "$staging/zoe-update"
mv -f "$staging/zoe-update" "$bin_dir/zoe-update"
mv -f "$staging/zoe" "$bin_dir/zoe"
printf 'Installed %s\nUpdate this checkout with %s\n' "$bin_dir/zoe" "$bin_dir/zoe-update"
