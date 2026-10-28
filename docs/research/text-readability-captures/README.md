# Structured text terminal previews

Synthetic data captured from the actual release executable in an owned disposable
tmux server. These are terminal cells rendered with Pillow, not browser mockups.
The app has no Pillow/tmux runtime dependency. Fixtures isolate HOME/CODEX_HOME,
use `NO_COLOR=1` with explicit `--color always`, and send no real native input.
Source identity and final visual/resource verification are recorded in
[the readability handoff](../../TEXT-READABILITY.md).

| Terminal | Work / readable | Work / overview | Commands / structured text |
| --- | --- | --- | --- |
| 150×42 | [PNG](work-150x42-readable.png) | [PNG](work-150x42-overview.png) | [PNG](work-150x42-commands.png) |
| 106×48 | [PNG](work-106x48-readable.png) | [PNG](work-106x48-overview.png) | [PNG](work-106x48-commands.png) |
| 60×28 | [PNG](work-60x28-readable.png) | [PNG](work-60x28-overview.png) | [PNG](work-60x28-commands.png) |

Paired `.txt` files preserve plain cells; `.ansi.txt` preserve actual terminal
styles. Source timestamps are deliberately unknown in this synthetic fixture.
Reported tasks and the zero verified-outcome count remain separate.

Reproduce after building the release:

```sh
python3 docs/research/probes/wg-5-terminal-capture.py target/release/zoe /tmp/zoe-readable-preview --readability
```
