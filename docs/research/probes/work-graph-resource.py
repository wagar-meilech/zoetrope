"""Measure the real release Work graph using an isolated structured source."""
import datetime
import fcntl
import json
import os
import pathlib
import pty
import re
import select
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time
import unicodedata

program = str(pathlib.Path(sys.argv[1]).resolve())
count = int(sys.argv[2]) if len(sys.argv) > 2 else 12
dense = "dense" in sys.argv[3:]
rich = "rich" in sys.argv[3:]
commands = "commands" in sys.argv[3:]
if commands and not rich:
    raise SystemExit("commands sample requires rich source fixture")
if not 1 <= count <= 512:
    raise SystemExit("task count must be 1..512")
with tempfile.TemporaryDirectory(prefix="zoe-work-resource-") as temporary:
    root = pathlib.Path(temporary)
    project = root / "project"
    project.mkdir()
    codex = root / "codex"
    state = root / "state"
    session = "resource-goal"
    now = datetime.datetime.now(datetime.timezone.utc)
    sessions = codex / "sessions" / now.strftime("%Y/%m/%d")
    sessions.mkdir(parents=True)
    records = [
        {"type": "session_meta", "payload": {"id": session, "session_id": session,
         "cwd": str(project), "source": "cli", "thread_source": "user"}},
        {"timestamp": now.isoformat(), "type": "response_item", "payload": {
         "type": "message", "role": "user", "content": [{"type": "input_text",
         "text": f"Work graph resource fixture with {count} tasks"}]}},
    ]
    message = ""
    if rich:
        message = "## Source context\nRead src/parser.rs:42 and **keep scope focused**.\n\n" + ("- [x] Updated README.md\n- [ ] Check ./tests/parser.rs\n\n```rust\nlet parser = read_source(\"src/parser.rs\");\n    validate(parser);\n```\n\n" * 50)
        records.append({"timestamp":now.isoformat(),"type":"response_item","payload":{"type":"message","role":"assistant","channel":"commentary","content":[{"type":"output_text","text":message}]}})
    (sessions / "rollout-resource.jsonl").write_text("".join(json.dumps(r)+"\n" for r in records))
    sources = []
    edge_count = 0
    chunk_size = 32 if dense else count
    for first in range(0, count, chunk_size):
        stop = min(first+chunk_size, count)
        lines = ["## Tasks", "", "| ID | Task | Status | Owner | Depends on | Purpose | Next |",
                 "| --- | --- | --- | --- | --- | --- | --- |"]
        for index in range(first, stop):
            status = "done" if index < count // 2 else "active" if index == count // 2 else "planned"
            dependencies = [other for other in range(first, stop) if other != index] if dense else ([] if index < 4 else [index-4])
            dependency = ", ".join(f"T{other:03}" for other in dependencies) or "None"
            edge_count += len(dependencies)
            lines.append(f"| T{index:03} | Deliver checkpoint {index:03} | {status} | main | {dependency} | Validate bounded local work | Inspect the next checkpoint |")
        source = project / f"WORK-{first:03}.md"
        source.write_text("\n".join(lines)+"\n")
        sources.append(source)
    environment = os.environ.copy()
    environment.update(CODEX_HOME=str(codex), HOME=str(root / "home"),
                       ZOETROPE_STATE_DIR=str(state), TERM="xterm-256color", NO_COLOR="1")
    pathlib.Path(environment["HOME"]).mkdir()
    setup = [program, "tracking", "enable", "--project", str(project), "--provider", "codex",
             "--session", session, "--root", str(state)]
    for source in sources:
        setup.extend(["--source", source.name+"#Tasks"])
    subprocess.run(setup, env=environment, check=True, capture_output=True)
    ready_read, ready_write = os.pipe()
    pid, terminal = pty.fork()
    if pid == 0:
        os.close(ready_write)
        os.read(ready_read, 1)
        os.close(ready_read)
        os.chdir(project)
        os.execve(program, [program, "--color", "always"], environment)
    os.close(ready_read)
    fcntl.ioctl(terminal, termios.TIOCSWINSZ, struct.pack("HHHH", 42, 150, 0, 0))
    os.write(ready_write, b"1")
    os.close(ready_write)
    captured = bytearray()
    reaped = False

    def drain(seconds):
        until = time.monotonic() + seconds
        while time.monotonic() < until:
            if select.select([terminal], [], [], min(0.05, max(0, until-time.monotonic())))[0]:
                try:
                    chunk = os.read(terminal, 65536)
                except OSError:
                    return
                if not chunk:
                    return
                captured.extend(chunk)
                if len(captured) > 4 * 1024 * 1024:
                    raise RuntimeError("unexpected terminal output flood")

    def screen_text():
        # Reconstruct cells: a terminal redraw sends changed characters only.
        cells = {}
        row = column = 0
        for part in re.split(r"(\x1b\[[0-?]*[ -/]*[@-~])", captured.decode("utf-8", "replace")):
            control = re.fullmatch(r"\x1b\[([0-?]*)[ -/]*([@-~])", part)
            if control:
                raw, operation = control.groups()
                if raw.startswith("?"):
                    if raw == "?1049" and operation in "hl":
                        cells.clear()
                    continue
                if any(ch not in "0123456789;" for ch in raw):
                    continue
                values = [int(value or "0") for value in raw.split(";")]
                amount = values[0] or 1
                if operation in "Hf":
                    row, column = amount-1, ((values[1] or 1)-1 if len(values)>1 else 0)
                elif operation == "A": row = max(0, row-amount)
                elif operation == "B": row += amount
                elif operation == "C": column += amount
                elif operation == "D": column = max(0, column-amount)
                elif operation == "G": column = amount-1
                elif operation == "d": row = amount-1
                elif operation == "J" and values[0] in (2, 3): cells.clear()
                elif operation == "K":
                    cells = {position:value for position,value in cells.items()
                             if position[0] != row or (values[0] == 0 and position[1]<column)
                             or (values[0] == 1 and position[1]>column)}
                continue
            for character in part:
                if character == "\r": column = 0
                elif character == "\n": row += 1
                elif character >= " ":
                    cells[row,column] = character
                    column += 0 if unicodedata.combining(character) else (2 if unicodedata.east_asian_width(character) in "WF" else 1)
        return "\n".join("".join(cells.get((r,c)," ") for c in range(150)) for r in range(42))

    def sample():
        raw = pathlib.Path(f"/proc/{pid}/stat").read_text()
        fields = raw[raw.rfind(")")+2:].split()
        memory = {}
        for line in pathlib.Path(f"/proc/{pid}/status").read_text().splitlines():
            if line.startswith(("VmRSS:", "VmHWM:")):
                key, value, unit = line.split()
                memory[key.rstrip(":")] = int(value)
        return time.monotonic(), int(fields[11])+int(fields[12]), memory

    try:
        until = time.monotonic()+10
        while "Selected:" not in screen_text() and time.monotonic() < until:
            drain(0.05)
        if "Selected:" not in screen_text():
            raise RuntimeError("fixture goal did not become selectable")
        opened = time.monotonic()
        os.write(terminal, b"w")
        until = opened+10
        while "Work graph" not in screen_text().splitlines()[0] and time.monotonic() < until:
            drain(0.05)
        if "Work graph" not in screen_text().splitlines()[0]:
            raise RuntimeError("work graph did not open")
        open_seconds = time.monotonic()-opened
        if commands:
            os.write(terminal, b"x")
            until = time.monotonic()+10
            while "Commands and public activity" not in screen_text() and time.monotonic() < until:
                drain(0.05)
            if "Commands and public activity" not in screen_text():
                raise RuntimeError("commands view did not open")
        drain(3)
        before = sample()
        start_bytes = len(captured)
        drain(10)
        after = sample()
        elapsed = after[0]-before[0]
        result = {"fixture": "isolated structured local source and real release executable",
                  "tasks": count, "sources": len(sources), "dense_within_sources": dense,
                  "structured_message_bytes": len(message.encode()), "commands_view": commands,
                  "declared_edges": edge_count,
                  "source_bytes": sum(path.stat().st_size for path in sources),
                  "open_seconds_observed": round(open_seconds, 3),
                  "measurement_seconds": round(elapsed, 3),
                  "cpu_percent_one_core": round((after[1]-before[1])/os.sysconf("SC_CLK_TCK")/elapsed*100, 3),
                  "rss_kib": after[2].get("VmRSS"), "peak_rss_kib": after[2].get("VmHWM"),
                  "idle_terminal_bytes": len(captured)-start_bytes,
                  "explicit_color_override": b"38;" in captured or b"[33m" in captured}
        os.write(terminal, b"q")
        drain(0.1)
        until = time.monotonic()+5
        while time.monotonic() < until:
            finished, status = os.waitpid(pid, os.WNOHANG)
            if finished:
                reaped = True
                result["exit_code"] = os.waitstatus_to_exitcode(status)
                break
            drain(0.05)
        if not reaped:
            raise RuntimeError("viewer did not exit after q")
        print(json.dumps(result, indent=2))
    finally:
        if not reaped:
            try:
                os.kill(pid, signal.SIGKILL)
                os.waitpid(pid, 0)
            except ProcessLookupError:
                pass
        os.close(terminal)
