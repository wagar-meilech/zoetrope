# ZOE-5 implementation

Worktree: `/Users/melikhov/Repos/zoetrope-worktrees/zoe-5`, branch `feat/zoe-5-preview`.
Approved seams: native document launch/external executables and the extension's public activation/URI handler against an external VS Code host adapter. No private helper tests.

Cycles 1–2 initially used `CARGO_TARGET_DIR=/Users/melikhov/Repos/zoetrope/target`. After the root identified worktree artifact collisions, all later commands use the isolated `CARGO_TARGET_DIR=/Users/melikhov/Repos/zoetrope/target/zoe-5` and the configured Rust 1.88 override. Final validation reruns the earlier cases there too.

## Vertical TDD evidence

1. Exact selected path, including spaces, Unicode, shell metacharacters, `#`, `+`, `%`: `cargo test --locked --test document_launch requests_preview_for_the_exact_selected_file_without_interpreting_its_name`.
   - RED: unresolved public `zoetrope::documents` API (exit 101).
   - GREEN: actual Python recording executables received one preview URI and independently decoded the exact file path; outcome says requested (1 passed).
2. Missing files and directories: `cargo test --locked --test document_launch unavailable_documents_report_a_clear_error_without_launching_an_editor`.
   - RED: missing document incorrectly returned `PreviewRequested` (exit 101).
   - GREEN: clear unavailable/not-regular-file errors, no external executable dispatch (1 passed).
3. Unavailable bridge fallback: `cargo test --locked --test document_launch an_unavailable_bridge_requests_the_selected_file_in_the_plain_editor`.
   - RED: absent `EditorRequested` public outcome (exit 101).
   - GREEN: extension list without bridge causes direct `code --reuse-window <absolute-selected-path>`, clear reason, requested-file outcome (1 passed).
4. Failed URI dispatch: `cargo test --locked --test document_launch failed_uri_dispatch_falls_back_to_the_selected_file_instead_of_claiming_a_preview`.
   - RED: URI opener exit 1 returned error without fallback (exit 101).
   - GREEN: real recording opener exit 1 causes plain-editor fallback with failure reason and exact file (1 passed).
5. Markdown-only dispatch: `cargo test --locked --test document_launch non_markdown_files_cannot_be_dispatched_as_reader_documents`.
   - RED: existing `.command` file was dispatched as preview (exit 101).
   - GREEN: `.md`/`.markdown` selection required before external dispatch (1 passed).
6. Extension URI entry point: `DYLD_LIBRARY_PATH=/opt/homebrew/Cellar/llhttp/9.3.0/lib node --test editors/vscode/test/preview.test.cjs`.
   - RED: bridge package/entry point did not exist (exit 1).
   - GREEN: packaged activation registers URI handler; exact real file content/path reaches MD Reader through the external host adapter (1 passed).
7. Reject non-file/remote/non-Markdown URI targets: same Node command with `--test-name-pattern='bridge rejects'` before the test filename.
   - RED: invalid command URI reached document open and rejected with an uncaught external-host error (exit 1).
   - GREEN: clear visible errors; no document or reader command dispatched (1 passed).
8. Missing MD Reader/command fallback: same Node command with `--test-name-pattern='unavailable MD Reader'`.
   - RED: no information explaining the unavailable reader (exit 1).
   - GREEN: exact plain document stays open; clear information; no false preview claim (1 passed).
9. Concurrent URI requests: same Node command with `--test-name-pattern='concurrent preview'`.
   - Initial async real-file adapter happened to serialize IO, hiding the race. Made the external host's file-open promise resolve immediately from real synchronous file reads, a valid host schedule.
   - RED: both MD Reader invocations received the second document (exit 1).
   - GREEN: serialized public URI requests each reach their own file (1 passed).
10. Reader activation/command failures: same Node command with `--test-name-pattern='activation and command failures'`.
    - RED: reader activation failure left no selected document open (exit 1).
    - GREEN: both failures leave exact plain document open with a clear explanation (1 passed).
11. Host URI fidelity: official [Microsoft URI implementation](https://github.com/microsoft/vscode-uri/blob/main/src/uri.ts) shows that `Uri.parse` percent-decodes `query` once. Updated the external host adapter to reproduce that public behavior.
    - RED: existing exact-file entry-point test (`--test-name-pattern='exact selected document'`) no longer reached MD Reader because `URLSearchParams` double-decoded the nested file URI (exit 1).
    - GREEN: consume the already-decoded single `uri=` query value and parse the nested URI once; all five public bridge cases pass, including `#`, `+`, `%`, and `&` names.

## External facts

- Installed VS Code CLI 1.142.0 has no public `--command`; uses argument-based file opening and `--list-extensions`.
- MD Reader is `AnandShah.mdreader` 0.0.8, command `mdreader.fullRender`.
- Official APIs: [commands](https://code.visualstudio.com/api/extension-guides/command), [manifest](https://code.visualstudio.com/api/references/extension-manifest), [CLI and extension URI format](https://code.visualstudio.com/docs/configure/command-line).
- Installed Homebrew Node 25.6.0 needs an existing older llhttp library. Run extension checks with `DYLD_LIBRARY_PATH=/opt/homebrew/Cellar/llhttp/9.3.0/lib node ...`; no global repair made.

No extension installation, GUI opening or release build performed in this ticket. The integration step must install the packaged bridge and verify a real preview on this Mac.

## Handoff and validation

- Feature commit `ceb363a` (`feat(api): request selected documents in vscode md reader`).
- Final head `6e0d0c8` adds API documentation requiring the blocking launch call to run off the UI thread, only for explicit open. Working tree is clean.
- After merging integration, reran all 5 launcher tests, all 5 bridge tests and formatting successfully. No affected product code arrived from integration.
- Merged latest integration `c44dc38` (resource-light implementation requirement) into this branch as `f3e758a`; only project/spec instructions changed. The bridge has no dependencies, service or watcher, activates only on URI, and extension listing happens only on explicit native open.
- `cargo test --locked`: 207 existing library + 9 existing binary + 5 native launcher tests passed in the isolated target directory.
- `cargo fmt --check`: passed.
- `cargo check --locked --no-default-features`: passed; native document IO is absent from portable builds.
- `node --test editors/vscode/test/preview.test.cjs` with the process-local llhttp loader path: all 5 tests passed.
- `node --check editors/vscode/extension.cjs`: passed.
- `python3 editors/vscode/package_vsix.py --output .local/research-artifacts/tmp/zoetrope-preview-0.1.0.vsix` and `python3 -m zipfile -t ...`: generated and inspected successfully. The packaged bridge was not installed.
- `cargo clippy --locked --all-targets -- -D warnings`: blocked by unchanged baseline warnings at `src/ui/panel.rs:99` (`uninlined_format_args`) and `src/state/session.rs:997` (`needless_borrows_for_generic_args`). Reported to the root integrator rather than editing other tickets' files.
- Rust 1.88 currently has only the host target installed; no wasm build attempted. Portable check passed; integration owns browser/release checks and real preview verification.
