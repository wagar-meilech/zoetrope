# Work graph final review fixes

27 October 2026. Isolated branch `feat/work-graph-review-fixes`, initially based
on integration `1478ba6`. Approved behavior and test seams are recorded in
`docs/WORK-GRAPH-SPEC.md`, `docs/WORK-GRAPH-PLAN.md` and the WG-3 public source
contract. The independent Standards/Spec findings are retained in
`work-graph-review.md`; this file records their fixes and validation.

## Bounded regular-file reads

The public executable/external-file regression
`nonregular_sources_retain_last_good_without_blocking_tracking_commands` covers
an initially selected Markdown FIFO and a valid imported regular file replaced
with a FIFO. It checks stale diagnostics, preservation of task identity, values
and provenance, and usable status/disable commands. A ten-second subprocess
boundary kills a deadlocked child and fails the test; it is not a performance
threshold. No FIFO writer is required to unblock the importer.

RED:

```sh
CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/fixes cargo test --locked --offline --test tracking nonregular_sources_retain_last_good_without_blocking_tracking_commands -- --exact --nocapture
```

Before the fix, this failed with `tracking enable deadlocked` when the subprocess
guard killed the blocked executable after ten seconds. After the fix the same
command passed, including the imported-file replacement case and last-good
assertions.

The importer now rejects nonregular/oversized metadata before opening. Unix uses
`OpenOptionsExt` with the platform's `libc::O_NONBLOCK`, then validates the opened
descriptor's type/size before any read. A regular-file-to-FIFO replacement between
stat and open therefore cannot block. Ordinary regular-file reads retain the
existing byte limit on every platform. The tiny direct `libc` edge is optional,
Unix-targeted and enabled only by `native`; no dependency versions changed.
The automatic inventory worker and CLI share this source-refresh path.

## Unique explicit ticket IDs

The public executable/files regression
`selected_ticket_ids_are_unique_and_colliding_edits_retain_the_last_good_namespace`
checks two selected ticket files declaring the same ID without any dependency
references. It then checks recovery to unique IDs with identical titles, an
invalid collision after a good snapshot, preservation of source values and field
ownership alongside a supplemental report, stale/errors, recovery, valid source
removal, and independent tables reusing the same raw ID.

RED:

```sh
CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/fixes cargo test --locked --offline --test tracking selected_ticket_ids_are_unique_and_colliding_edits_retain_the_last_good_namespace -- --exact --nocapture
```

Before the fix, this failed on the initial expected stale flag: both duplicates
were accepted as fresh. The same command passed after the fix.

Enable, explicit sync and inventory refresh now validate declared IDs across the
selected ticket namespace before publishing it. A collision restores that
namespace's previous tasks, fingerprints, retirement observations and field
authority, marks it stale and records the duplicate ID. This avoids publishing a
partially edited or ambiguously named ticket plan. Independent table namespaces
remain separate; labels never merge tasks.

## Typed visual cues and compiler compatibility

Work cards, progress, legend and focused inspector status cues now receive
`Option<WorkStatus>`. Fleet/runtime cues receive `RuntimeStatus` and the archive
flag independently of their presentation labels. Existing public render/style
coverage verifies colors, glyphs, focused blockers and overview selection; the
raw unknown native phrase `reported done` still renders as unknown/magenta and
never counts as reported completion. No private wording/helper tests were added.

The root's Rust 1.88 strict-Clippy check found `nonminimal_bool` in the auto color
preference expression at the integration baseline. It now uses
`var_os("NO_COLOR").is_none_or(|value| value.is_empty())`, accepted by both supported
check toolchains without an allow suppression.

## Checks

The two RED/GREEN slices above passed. Strict Clippy passed on both compilers:

```sh
CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/fixes cargo +1.99.0 clippy --locked --offline --all-targets -- -D warnings
CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/sources-msrv cargo +1.88.0 clippy --locked --offline --all-targets -- -D warnings
```

The excluded browser workspace also passed with its existing lockfile:

```sh
CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/final-wasm cargo check --locked --offline --manifest-path web/wasm/Cargo.toml --target wasm32-unknown-unknown
```

Installed `stable` reports Rust 1.99.0 and has the wasm standard library; the
separately installed `1.99.0` alias does not have that target. An initial browser
check through the latter failed for the missing standard library; using the
already installed stable target passed. Both root and browser workspace lock
updates were performed offline; only the root package's direct `libc` edge changed.

An initial parallel targeted run passed all eight PTY/runtime tests and 68 UI
tests with one manual probe ignored, but the existing save-backpressure test
`feedback_stays_in_the_composer_when_the_save_queue_is_full_then_saves_on_exit`
failed its immediate `queue` text assertion while displaying the retained draft
and `Saving draft… waiting for local storage`. Its exact isolated rerun passed.
The fixture review found three contention tests chose the first `.lock` in the
state directory, which can also contain `tracking.lock` and other goal locks.
They now select the serialized public goal record by exact session identity and
hold its sibling lock. Each fixture explicitly enables an unrelated tracking
binding so the tracking lock is present; existing draft-retention, queue and
flush assertions are unchanged. Seeding the extra lock alone did not reproduce
the original failure deterministically, so this is recorded as a fixture
correction rather than a claimed deterministic RED/GREEN slice.

Both serial targeted runs passed: 18 tracking tests, 69 UI tests and eight
PTY/runtime tests, with the one manual resource probe ignored:

```sh
CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/fixes cargo test --locked --offline --test tracking --test fleet_ui --test fleet_runtime -- --test-threads=1
CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/sources-msrv cargo +1.88.0 test --locked --offline --test tracking --test fleet_ui --test fleet_runtime -- --test-threads=1
```

After correcting the contention fixtures, normal parallel UI suites passed on
both compilers, 69 tests each with one manual probe ignored. Strict Clippy on both
compilers was rerun and passed with that final test setup:

```sh
CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/fixes cargo test --locked --offline --test fleet_ui
CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/sources-msrv cargo +1.88.0 test --locked --offline --test fleet_ui
```

Portable compatibility passed: 176 library and 13 public Console tests:

```sh
CARGO_TARGET_DIR=/tmp/zoetrope-wg-build/final-portable cargo test --locked --offline --no-default-features --lib --test console
```

Formatting and `git diff --check` pass. Root/WG-6 owns final full-suite, package,
rustdoc, resource, installation and live-observation validation after integration.
