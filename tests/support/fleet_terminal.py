"""Drive the real zoe executable through an owned terminal (stdlib only)."""
import fcntl
import json
import os
import pathlib
import pty
import re
import select
import shlex
import signal
import struct
import subprocess
import sys
import termios
import time
import unicodedata

executable, fixture_root, scenario = sys.argv[1:]
fixture = pathlib.Path(fixture_root)
peer = None
command_environment = os.environ.copy()
if scenario in ("tracking-cli", "criteria-cli"):
    fake_bin = fixture / "fake-bin"
    fake_bin.mkdir(exist_ok=True)
    fake = fake_bin / "zoe"
    fake.write_text("#!/bin/sh\necho incompatible-zoe >&2\nexit 72\n")
    fake.chmod(0o755)
    command_environment["PATH"] = str(fake_bin) + os.pathsep + command_environment.get("PATH", "")
if scenario in ("replay", "mouse-replay"):
    socket_path = fixture / "codex/app-server-control/app-server-control.sock"
    socket_path.parent.mkdir(parents=True, exist_ok=True)
    peer = subprocess.Popen(
        [sys.executable, str(pathlib.Path(__file__).with_name("codex_fleet_peer.py")), str(socket_path), "observe"],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    ready = pathlib.Path(str(socket_path) + ".ready")
    deadline = time.monotonic() + 5
    while not ready.exists() and time.monotonic() < deadline:
        if peer.poll() is not None:
            raise RuntimeError(peer.communicate()[1].decode())
        time.sleep(0.01)
    if not ready.exists():
        peer.terminate()
        peer.wait()
        raise RuntimeError("protocol peer did not become ready")
start_read, start_write = os.pipe()
pid, terminal = pty.fork()
if pid == 0:
    os.close(start_write)
    os.read(start_read, 1)
    os.close(start_read)
    os.chdir(fixture if scenario in ("tracking-cli", "criteria-cli") else fixture / "project")
    os.environ.update(command_environment)
    os.environ["CODEX_HOME"] = str(fixture / "codex")
    os.environ["ZOETROPE_STATE_DIR"] = str(fixture / "state")
    os.environ["TERM"] = "xterm-256color"
    arguments = ["--project"] if scenario == "project" else []
    os.execv(executable, [executable, *arguments])

os.close(start_read)
fcntl.ioctl(terminal, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 160, 0, 0))
os.write(start_write, b"1")
os.close(start_write)
data = bytearray()
status = None


def receive(timeout=0.05):
    if select.select([terminal], [], [], timeout)[0]:
        try:
            chunk = os.read(terminal, 65536)
        except OSError:
            return False
        data.extend(chunk)
        if len(data) > 2 * 1024 * 1024:
            raise RuntimeError("unexpected terminal output flood")
        return bool(chunk)
    return True


def visible():
    text = data.decode("utf-8", "replace")
    return re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", text)


def screen_text():
    # Ratatui emits changed cells only, including skips over unchanged letters.
    # Reconstruct the owned terminal's visible cells for content assertions.
    cells = {}
    row = column = 0
    for part in re.split(r"(\x1b\[[0-?]*[ -/]*[@-~])", data.decode("utf-8", "replace")):
        control = re.fullmatch(r"\x1b\[([0-?]*)[ -/]*([@-~])", part)
        if control:
            raw, operation = control.groups()
            if raw.startswith("?"):
                if raw == "?1049" and operation in "hl":
                    cells.clear()
                continue
            if any(character not in "0123456789;" for character in raw):
                continue
            values = [int(value or "0") for value in raw.split(";")]
            amount = values[0] or 1
            if operation in "Hf":
                row, column = amount - 1, (values[1] or 1) - 1 if len(values) > 1 else 0
            elif operation == "A": row = max(0, row - amount)
            elif operation == "B": row += amount
            elif operation == "C": column += amount
            elif operation == "D": column = max(0, column - amount)
            elif operation == "J" and values[0] in (2, 3): cells.clear()
            elif operation == "K":
                cells = {position: value for position, value in cells.items()
                         if position[0] != row or (values[0] == 0 and position[1] < column)
                         or (values[0] == 1 and position[1] > column)}
            continue
        for character in part:
            if character == "\r": column = 0
            elif character == "\n": row += 1
            elif character >= " ":
                cells[row, column] = character
                column += 0 if unicodedata.combining(character) else (2 if unicodedata.east_asian_width(character) in "WF" else 1)
    return "\n".join("".join(cells.get((r, c), " ") for c in range(160)) for r in range(40))


def click(label):
    deadline = time.monotonic() + 2
    while time.monotonic() < deadline:
        for row, line in enumerate(screen_text().splitlines()):
            column = line.find(label)
            if column >= 0:
                os.write(terminal, f"\x1b[<0;{column + 1};{row + 1}M\x1b[<0;{column + 1};{row + 1}m".encode())
                return True
        if not receive():
            return False
    return False


def expect(text, seconds=5, since=0, compact=False):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        output = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", data[since:].decode("utf-8", "replace"))
        # Incremental redraws can skip unchanged blank cells by moving the
        # cursor. UI layout is tested separately; here compare emitted words.
        if compact:
            output = re.sub(r"\s", "", output)
        expected = re.sub(r"\s", "", text) if compact else text
        if expected in output or (compact and expected in re.sub(r"\s", "", screen_text())):
            return True
        if not receive():
            return False
    return False


try:
    if scenario == "project":
        observations = {"project": expect("Inspect runtime fixture goal"), "fleet": "Current work" in visible()}
    else:
        observations = {"fleet": expect("Current work")}
    if scenario in ("autosave", "paste") and observations["fleet"]:
        expect("Inspect runtime fixture goal", compact=True)
        os.write(terminal, b"f")
        if expect("Editing feedback", compact=True):
            # Let startup worker notifications settle before editing. Nothing
            # else is sent to the terminal until persistence is observed.
            deadline = time.monotonic() + 0.3
            while time.monotonic() < deadline:
                receive()
            expected = "Save while I am thinking"
            if scenario == "paste":
                expected = "H?qR\nsecond line"
                observations["paste_enabled"] = b"\x1b[?2004h" in data
                os.write(terminal, b"\x1b[200~" + expected.encode() + b"\x1b[201~")
            else:
                os.write(terminal, expected.encode())
            deadline = time.monotonic() + 2
            observations["autosaved"] = False
            while time.monotonic() < deadline:
                output = subprocess.check_output([
                    executable, "state", "--root", str(fixture / "state"),
                    "show", "codex", "root-goal",
                ])
                drafts = json.loads(output)["record"]["drafts"]
                if drafts and drafts[0]["text"] == expected:
                    observations["autosaved"] = True
                    break
                receive(0.05)
        os.write(terminal, b"\x03")
    if scenario in ("tracking-cli", "criteria-cli") and observations["fleet"]:
        observations["fake_on_path"] = subprocess.check_output(
            ["/bin/sh", "-c", "command -v zoe"], env=command_environment, text=True
        ).strip() == str(fake)
        expect("Inspect runtime fixture goal", compact=True)
        key, operation = (b"T", "context") if scenario == "tracking-cli" else (b"A", "show")
        os.write(terminal, key)
        deadline = time.monotonic() + 3
        command = None
        while time.monotonic() < deadline:
            saved = json.loads(subprocess.check_output([
                executable, "state", "--root", str(fixture / "state"),
                "show", "codex", "root-goal",
            ]))
            drafts = saved["record"]["drafts"]
            if drafts:
                command = next((line for line in drafts[0]["text"].splitlines()
                                if " state --root " in line and f" {operation} " in line), None)
                if command:
                    break
            receive(0.05)
        observations["current_executable"] = bool(command) and shlex.split(command)[0] == os.path.realpath(executable)
        result = subprocess.run(["/bin/sh", "-c", command or "exit 73"],
                                cwd=fixture / "project", env=command_environment,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            record = json.loads(result.stdout)
            goal = record["goal"] if operation == "context" else record["record"]["goal"]
            observations["reads_exact_store"] = result.returncode == 0 and goal["title"] == "Inspect runtime fixture goal"
        except (ValueError, KeyError, TypeError):
            observations["reads_exact_store"] = False
            observations["command_error"] = result.stderr
        os.write(terminal, b"\x03")
    if scenario in ("replay", "mouse-replay") and observations["fleet"]:
        expect("Inspect runtime fixture goal", compact=True)
        os.write(terminal, b"H")
        expect("All history")
        opened = False
        for _ in range(5):
            os.write(terminal, b"f")
            if expect("Editing feedback", seconds=0.5):
                opened = True
                break
        if opened:
            os.write(terminal, b"Keep H/q R draft\x1b")
            expect("Draft saved", compact=True)
            entered = True
            returned = True
            for _ in range(2):
                mark = len(data)
                if scenario == "mouse-replay":
                    clicked = click("[Replay R]")
                    observations["mouse"] = observations.get("mouse", True) and clicked
                    if not clicked:
                        entered = False
                        break
                else:
                    os.write(terminal, b"R")
                if not expect("Read-only replay", since=mark):
                    entered = False
                    break
                # These are controls only in the fleet; replay must ignore them.
                os.write(terminal, b"\x1bOQ\x1bOR\x1bOS")
                mark = len(data)
                os.write(terminal, b"q")
                if not expect("All history", since=mark):
                    returned = False
                    break
            observations.update(replay=entered, returned=returned)
            if entered and returned:
                mark = len(data)
                os.write(terminal, b"f")
                observations["draft"] = expect("Keep H/q R draft", since=mark, compact=True)
                mark = len(data)
                os.write(terminal, b"\x1b")
                expect("Draft saved", since=mark, compact=True)
        else:
            observations["replay"] = False
    try:
        if scenario not in ("autosave", "paste", "tracking-cli", "criteria-cli"):
            os.write(terminal, b"q")
    except OSError:
        pass  # An invalid CLI invocation can exit before accepting input.
    deadline = time.monotonic() + 4
    while time.monotonic() < deadline:
        child, state = os.waitpid(pid, os.WNOHANG)
        if child:
            status = state
            break
        receive()
    if status is None:
        os.kill(pid, signal.SIGTERM)
        _, status = os.waitpid(pid, 0)
    observations["exit"] = os.waitstatus_to_exitcode(status)
    if scenario == "paste":
        while receive(0):
            if not select.select([terminal], [], [], 0)[0]:
                break
        observations["paste_disabled"] = b"\x1b[?2004l" in data
    if peer is not None:
        try:
            _, errors = peer.communicate(timeout=3)
        except subprocess.TimeoutExpired:
            peer.terminate()
            _, errors = peer.communicate()
        observations["peer_exit"] = peer.returncode
        if peer.returncode:
            observations["peer_error"] = errors.decode()[-600:]
    if observations["exit"] != 0 or not any(value is True for value in observations.values()) or observations.get("replay") is False:
        observations["tail"] = visible()[-1200:]
    print(json.dumps(observations))
finally:
    if status is None:
        try:
            os.kill(pid, signal.SIGTERM)
            os.waitpid(pid, 0)
        except ProcessLookupError:
            pass
    os.close(terminal)
    if peer is not None and peer.poll() is None:
        peer.terminate()
        peer.wait()
