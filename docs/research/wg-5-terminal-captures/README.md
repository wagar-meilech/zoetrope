# Synthetic Work graph terminal captures

Captured from the final installed release, application revision `2a65e315`,
on 28 October 2026 Europe/Amsterdam (27 October UTC), using an owned disposable
tmux server and synthetic local reports. These are captured terminal cells rendered as PNG with Pillow and
DejaVu Sans Mono, not browser designs. All six views were visually inspected during WG-5; final installed-release QA
rechecked narrow readable and wide/tall overview views. The capture helper
isolates both HOME and CODEX_HOME.

The 12-task fixture includes two fork/merge branches, cancelled and unknown work,
5 reported done tasks and 0 of 5 verified outcomes. Its child environment uses
`TERM=tmux-256color`, `NO_COLOR=1` and `--color always`; the paired raw ANSI files
retain actual terminal color escapes. Captured row counts match the stated sizes.

| Terminal and view | Image | Plain cells | Actual terminal styles |
| --- | --- | --- | --- |
| 150×42 readable | [PNG](work-150x42-readable.png) | [Text](work-150x42-readable.txt) | [ANSI](work-150x42-readable.ansi.txt) |
| 150×42 overview | [PNG](work-150x42-overview.png) | [Text](work-150x42-overview.txt) | [ANSI](work-150x42-overview.ansi.txt) |
| 106×48 readable | [PNG](work-106x48-readable.png) | [Text](work-106x48-readable.txt) | [ANSI](work-106x48-readable.ansi.txt) |
| 106×48 overview | [PNG](work-106x48-overview.png) | [Text](work-106x48-overview.txt) | [ANSI](work-106x48-overview.ansi.txt) |
| 60×28 readable | [PNG](work-60x28-readable.png) | [Text](work-60x28-readable.txt) | [ANSI](work-60x28-readable.ansi.txt) |
| 60×28 overview | [PNG](work-60x28-overview.png) | [Text](work-60x28-overview.txt) | [ANSI](work-60x28-overview.ansi.txt) |

See [implementation and RED/GREEN evidence](../wg-5-implementation.md) and the
[reproduction script](../probes/wg-5-terminal-capture.py). The script requires
local tmux/Pillow solely for QA; the Rust app has no new runtime dependency.
