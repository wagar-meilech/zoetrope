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
            printf '%s\n' 'Usage: scripts/install.sh [--bin-dir DIR]' 'Default destination: $HOME/.local/bin' 'Skill destination: ${CODEX_HOME:-$HOME/.codex}/skills/zoetrope-tracking'
            exit 0
            ;;
        *) printf 'Unknown option: %s\n' "$1" >&2; exit 2 ;;
    esac
done
if [ -z "$bin_dir" ]; then
    : "${HOME:?Set HOME or pass --bin-dir DIR}"
    bin_dir=$HOME/.local/bin
fi
if [ -n "${CODEX_HOME:-}" ]; then
    codex_home=$CODEX_HOME
else
    : "${HOME:?Set HOME or CODEX_HOME for the user skill destination}"
    codex_home=$HOME/.codex
fi
mkdir -p "$codex_home"
codex_home=$(CDPATH= cd "$codex_home" && pwd -P)
checkout=$(CDPATH= cd "$(dirname "$0")/.." && pwd -P)
[ -f "$checkout/skills/zoetrope-tracking/SKILL.md" ] || { printf '%s\n' 'Packaged zoetrope-tracking skill is missing' >&2; exit 1; }
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
skills_home=$codex_home/skills
skill_dir=$skills_home/zoetrope-tracking
if [ -L "$skill_dir" ] || { [ -e "$skill_dir" ] && [ ! -d "$skill_dir" ]; }; then
    printf 'Cannot replace skill path: %s\n' "$skill_dir" >&2
    exit 1
fi
mkdir -p "$skills_home"
staging=$(mktemp -d "$bin_dir/.zoe-install.XXXXXX")
skill_staging=
cleanup() {
    if [ -n "$skill_staging" ]; then
        if [ -d "$skill_staging/previous" ] && [ ! -e "$skill_dir" ]; then
            mv "$skill_staging/previous" "$skill_dir"
        fi
        rm -rf "$skill_staging"
    fi
    rm -rf "$staging"
}
trap cleanup 0
trap 'exit 1' HUP INT TERM
skill_staging=$(mktemp -d "$skills_home/.zoe-skill.XXXXXX")
mkdir "$skill_staging/new"
# Stage packaged files first, then preserve user additions without following
# existing file symlinks or overwriting the new package with an older version.
cp -R "$checkout/skills/zoetrope-tracking/." "$skill_staging/new/"
if [ -d "$skill_dir" ]; then
    cp -R -n "$skill_dir/." "$skill_staging/new/"
fi
cp "$target_dir/release/zoe" "$staging/zoe"
shell_quote() {
    printf "'"
    printf '%s' "$1" | sed "s/'/'\\\\''/g"
    printf "'"
}
{
    printf '%s\n' '#!/bin/sh' 'set -eu'
    printf 'checkout=%s\n' "$(shell_quote "$checkout")"
    printf 'export CODEX_HOME=%s\n' "$(shell_quote "$codex_home")"
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
if [ -d "$skill_dir" ]; then
    mv "$skill_dir" "$skill_staging/previous"
fi
mv "$skill_staging/new" "$skill_dir"
mv -f "$staging/zoe-update" "$bin_dir/zoe-update"
mv -f "$staging/zoe" "$bin_dir/zoe"
printf 'Installed %s\nInstalled skill %s\nUpdate this checkout with %s\n' "$bin_dir/zoe" "$skill_dir" "$bin_dir/zoe-update"
