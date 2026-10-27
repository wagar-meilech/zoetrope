"""Measure an owned public FleetConsole runtime with an isolated local fixture."""
import fcntl
import json
import os
import pathlib
import pty
import select
import signal
import struct
import sys
import termios
import time

program, root = sys.argv[1:]
pid, terminal = pty.fork()
if pid == 0:
    os.environ["TERM"] = "xterm-256color"
    os.execv(program, [program, root])
fcntl.ioctl(terminal, termios.TIOCSWINSZ, struct.pack("HHHH", 42, 150, 0, 0))
captured = bytearray()

def drain(seconds):
    until = time.monotonic() + seconds
    while time.monotonic() < until:
        if select.select([terminal], [], [], min(0.05, max(0, until-time.monotonic())))[0]:
            try:
                chunk = os.read(terminal, 65536)
            except OSError:
                break
            if not chunk:
                break
            captured.extend(chunk)
            if len(captured) > 2 * 1024 * 1024:
                raise RuntimeError("unexpected terminal output flood")

def sample():
    raw = pathlib.Path(f"/proc/{pid}/stat").read_text()
    fields = raw[raw.rfind(")") + 2:].split()
    ticks = int(fields[11]) + int(fields[12])
    memory = {}
    for line in pathlib.Path(f"/proc/{pid}/status").read_text().splitlines():
        if line.startswith(("VmRSS:", "VmHWM:")):
            key, value, unit = line.split()
            memory[key.rstrip(":")] = int(value)
    return time.monotonic(), ticks, memory

try:
    drain(3)
    before = sample()
    idle_start_bytes = len(captured)
    drain(16)
    after = sample()
    hz = os.sysconf("SC_CLK_TCK")
    elapsed = after[0] - before[0]
    result = {
        "measurement": "isolated release public terminal runtime, current-work view",
        "elapsed_seconds": round(elapsed, 3),
        "process_cpu_percent": round((after[1]-before[1])/hz/elapsed*100, 3),
        "rss_kib": after[2].get("VmRSS"),
        "peak_rss_kib": after[2].get("VmHWM"),
        "idle_terminal_bytes": len(captured)-idle_start_bytes,
        "terminal_initialized": b"Zoetrope" in captured,
        "observed_work": b"Semantic resource fixture" in captured,
    }
    os.write(terminal, b"q")
    drain(1)
    until = time.monotonic() + 5
    while True:
        finished, status = os.waitpid(pid, os.WNOHANG)
        if finished:
            result["exit_code"] = os.waitstatus_to_exitcode(status)
            break
        if time.monotonic() > until:
            raise RuntimeError("runtime did not quit")
        drain(0.05)
    print(json.dumps(result))
finally:
    try:
        os.kill(pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    os.close(terminal)
