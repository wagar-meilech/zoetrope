"""Drive the real zoe executable through an owned terminal (stdlib only)."""
import fcntl
import json
import os
import pathlib
import pty
import re
import select
import signal
import struct
import sys
import termios
import time

executable, fixture_root, scenario = sys.argv[1:]
fixture = pathlib.Path(fixture_root)
start_read, start_write = os.pipe()
pid, terminal = pty.fork()
if pid == 0:
    os.close(start_write)
    os.read(start_read, 1)
    os.close(start_read)
    os.chdir(fixture / "project")
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


def expect(text, seconds=5):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if text in visible():
            return True
        if not receive():
            return False
    return False


try:
    if scenario == "project":
        observations = {"project": expect("Inspect runtime fixture goal"), "fleet": "Current work" in visible()}
    else:
        observations = {"fleet": expect("Current work")}
    try:
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
    if not any(value is True for value in observations.values()):
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
