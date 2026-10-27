# Local installation implementation — 27 October 2026

Implements [INSTALLATION-SPEC](../INSTALLATION-SPEC.md), issue #14, through the
approved installer/updater/executable subprocess seam. Based on integration
`69e78b2`; current integration is merged before completion. No app CLI, Rust
runtime dependency, service, Git operation or implicit source update is added.

## Interface and behavior

- `./scripts/install.sh` defaults to `$HOME/.local/bin`.
- `./scripts/install.sh --bin-dir DIR` chooses an explicit destination;
  `--help` documents the option. Spaces and apostrophes in source/destination
  paths work.
- Build command: `cargo build --release --locked --bin zoe --target-dir DIR`,
  run from the physical checkout directory. The default target is its `target/`;
  `CARGO_TARGET_DIR` remains supported, with relative values based at the checkout.
- Both command artifacts are staged inside the destination after successful
  build, permissions set, then renamed. Existing command directories are rejected
  before either file changes. Build failures leave both installed commands intact.
- Installed `zoe` is the compiled artifact itself. Its normal launch invokes no
  Cargo or background service.
- Generated `zoe-update` records the physical checkout and destination using shell
  quoting, and invokes that checkout’s installer. No Git pull, branch change,
  reset, upstream crate installation or checkout deletion occurs. The updater
  fails with reinstall guidance if its source checkout has moved/disappeared.
- Only the README installation section changes. Root integrates and performs the
  actual connected-Linux installation from the persistent checkout, avoiding a
  temporary-worktree dependency. macOS portability is designed with POSIX shell
  and common utilities; execution evidence here is Linux only.

## Vertical TDD evidence

All tests are public processes against isolated fixtures. Cargo is replaced at
its external executable boundary with a fixture build tool which requires a
locked release `zoe` build and creates versioned executable candidates. No
application internals are mocked.

Each command uses `CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/local-install`:

```sh
cargo test --locked --test local_install installing_to_the_user_bin_runs_the_release_without_cargo_at_launch -- --exact
```

RED: the installer process could not open the missing helper. GREEN: a default
install runs its installed command with an empty PATH, without needing Cargo.

```sh
cargo test --locked --test local_install updating_twice_from_another_directory_preserves_the_running_command -- --exact
```

RED: installation produced no updater. GREEN: two updater runs from another
project install the new version from the same checkout. Paths contain spaces and
apostrophes. A held old command completes with its original version, and an open
old artifact still reads the original bytes, verifying rename replacement rather
than truncating the live file.

```sh
cargo test --locked --test local_install a_moved_checkout_explains_reinstallation_without_breaking_the_installed_command -- --exact
```

RED: missing source produced only the shell’s exec error. GREEN: the updater
explains reinstallation, the installed app still runs, and reinstalling from the
moved checkout restores updates.

```sh
cargo test --locked --test local_install an_existing_command_directory_is_rejected_before_installed_files_change -- --exact
```

RED: the installer reported success after moving the executable into an existing
`zoe/` directory and replacing its sibling updater. GREEN: it rejects the invalid
target before replacing either installed artifact.

Additional public regression:
`a_failed_update_leaves_both_installed_commands_usable_and_unchanged` was already
green with the build-before-replace implementation; no artificial RED was
introduced. It forces Cargo exit27, verifies both old artifacts are unchanged,
runs the old command, then removes the failure and successfully updates.

## Acceptance checks

Run after all helper/test/document edits:

```sh
CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/local-install cargo test --locked --test local_install
CARGO_TARGET_DIR=/tmp/zoetrope-sem-build/local-install cargo clippy --locked --all-targets --all-features -- -D warnings
cargo fmt --all --check
sh -n scripts/install.sh
./scripts/install.sh --help
git diff --check
```

All five installer/updater tests passed on the connected Linux environment.
Strict Rust 1.99 all-target/all-feature Clippy, formatting, shell syntax/help and
whitespace checks passed. The handoff reports the exact checked commit after
rechecking its tree. Full semantic/native regression validation is owned by
integration and need not repeat for shell-only installation work. Actual release-build/install evidence is recorded separately
by root on the persistent connected-Linux checkout.


## Persistent connected-environment installation

The user explicitly selected the connected environment after clarification that
execution is Linux x86_64. Installed from the permanent project checkout at
`dfd777d49c8bf0579df77f49f0f78fdc3a19b0eb`, using `./scripts/install.sh`.
No installation on the earlier Mac is claimed.

- Installed compiled `zoe` and checkout-pinned `zoe-update` in `~/.local/bin`,
  already present on this environment's PATH. No shell startup files were changed.
- `zoe --version` prints `zoe 0.2.0` even with an empty PATH. The installed
  executable matches the built release SHA-256:
  `a8d49f82c3345d7384863f1696b531cadc7a960c8e1f68743c3db1f70bf34001`.
- The installed release opened the fleet in an owned PTY and exited normally.
  Its compact CLI context returned the three synthetic work items. Only an
  isolated disposable fixture was changed; no live session was steered.
- Running `zoe-update` from `/tmp` succeeded, rebuilt the permanent checkout
  with locked dependencies, and replaced the installed executable with a new
  inode while retaining identical release bytes. Rechecking `zoe --version`
  passed. Existing process/open-artifact preservation is covered by the public
  installer regression above.

Use `zoe` to launch and `zoe-update` after editing this checkout. The update
command uses the current checkout contents, including local changes; it does
not fetch a remote branch. Leave the checkout in place, or reinstall after
moving it. Updates need Cargo/Rust; normal launch does not.

## Independent review

Reviewed immutable `69e78b2...dfd777d`; the prior semantic application review
remains clear at `856c565`. The installer introduced no application-source or
Cargo dependency changes.

### Standards

Zero hard violations and zero useful smell findings. The reviewer checked the
portable helper, locked build-before-stage, same-filesystem replacement,
quoted persistent updater, moved-checkout guidance, public process tests and
repository documentation. No remaining findings.

### Spec

Zero confirmed missing requirements, scope creep or incorrect behavior. The
reviewer checked all five public regressions and the installation/toolchain/PATH
instructions. Actual installation was a coordinating-agent action subsequent to
review and is recorded above. No remaining findings.

Summary: Standards 0 findings; Spec 0 findings.
