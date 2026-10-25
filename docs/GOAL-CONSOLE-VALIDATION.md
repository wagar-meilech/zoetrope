# Goal console validation — 25 October 2026

The customized native release is installed on this Apple Silicon Mac. Bare
`zoe` opens the fleet console. The [usage guide](GOAL-CONSOLE.md) describes the
controls; explicit project, file, session ID and inspect commands remain.

## Checks

- 290 native Rust tests passed; one manual resource probe is deliberately
  ignored by the ordinary test run.
- 176 portable library tests passed with default features disabled.
- Formatting, strict all-target/all-feature clippy, portable compilation and
  warnings-as-errors rustdoc passed.
- The separately locked browser frontend passed its WebAssembly build check.
- Five VS Code host tests passed, alongside the Rust document-launch tests
  included in the native suite.
- `cargo publish --dry-run --locked` packaged and compiled the standalone
  tarball successfully. Nothing was published.
- Repository spelling checks passed. Feature commit headers are normalized
  separately to satisfy the existing Conventional Commit lint without changing
  source files.

Native checks and the optimized release used Rust 1.88. The browser check used
Rust 1.90, required by its existing locked dependencies. These are local macOS
results; Linux and Windows test runs are not claimed here.

## Workflows and review

Public console and executable tests cover global history/search, stable
selection, criteria/evidence, exact recipients, native delivery states,
persistent questions and drafts, document opening, narrow terminals and
repeated replay/return. Five executable/PTY lifecycle cases are part of the
native suite. The installed release was also exercised in an owned terminal
against real local history, including criteria, documents and read-only replay.
An external criterion verification refreshed the selected bar from 4/6 to 5/6
without navigation.

Independent final review cleared all eight findings and follow-up regressions:
external state invalidation, secret masking/copy refusal, atomic paste limits,
durable save acknowledgments, cache stamp ordering, public Codex phase parsing,
local copy versus submission keys, and graph placement scaling. Extra cases
cover delayed copy preserving newer feedback, incomplete history retaining
valid edges, and repeated F2 answer saves staying in the editor.

The MD Reader bridge was installed locally and a real VS Code preview of the
exact selected specification was verified. Launcher and host tests also cover
literal paths, unavailable readers/files and concurrent requests.

## Resource observations

The optimized executable is about 5.9 MiB. Passive monitoring makes no model
calls and adds no Internet service or telemetry.

| Installed release observation | Average process CPU | RSS |
| --- | ---: | ---: |
| Fresh launch, Current work, documents pane | About 0.2% | 8.2–8.4 MiB |
| After history/search and full session replay, returned to fleet | About 0.2% | 23.4 MiB |

Each CPU observation compares cumulative process CPU time over roughly
16 seconds after startup. Replay is an explicit full-transcript operation;
these are short observations on this Mac, not memory ceilings or universal
performance guarantees.

A public debug-build fixture with 1,000/4,000 archived goals measured peak UI
polling at 25.2/108.5 ms and startup including discovery at 428/1,263 ms. Stable
rendered positions survived an unchanged refresh. Occupied-coordinate indexing
and batched geometry removed a measured quadratic placement stall. The fixture
has no timing-fragile CI threshold.

## Local Codex compatibility

Disposable sessions on installed Codex 0.164 verified shared observation,
exact-turn guidance/interruption, stale-turn rejection, native queue operations
and question replay/resolution. Queueing on idle work may start it immediately.
Ordinary Codex remains available, with its permission policy unchanged.

Some child/helper threads report that they cannot accept direct input. The
console explains this and never silently redirects feedback to a parent.
Second-client answer arbitration, competing answers and real command/permission
approval coexistence were not established by those probes. Unsupported live
actions fail visibly; historical requests alone never establish a currently
pending native question. Native approvals stay in Codex.
