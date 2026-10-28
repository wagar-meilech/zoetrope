"""Normalize only generated merge/WIP headers; consume/emit one commit message.

Used at final handoff with a backup ref, scoped to the agent-owned feature
branch. This script changes no files or refs itself.
"""

import sys

message = sys.stdin.read()
header, separator, body = message.partition("\n")

if header.startswith(("Merge branch ", "Merge remote-tracking branch ")):
    original = message.rstrip("\n")
    message = "chore(api): merge integration changes\n\n" + original + "\n"
elif header.startswith("wip(") or header.startswith("wip:"):
    message = "chore" + header[3:] + separator + body

sys.stdout.write(message)
