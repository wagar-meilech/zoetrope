"""Create a synthetic local work fixture for manual console/resource inspection."""
import datetime
import json
import pathlib
import subprocess
import sys

executable, destination = sys.argv[1:]
program = str(pathlib.Path(executable).resolve())
root = pathlib.Path(destination).resolve()
root.mkdir(parents=True, exist_ok=True)
marker = root / "synthetic-fixture.json"
if any(root.iterdir()):
    raise SystemExit("fixture destination must be empty; choose a new directory")
marker.write_text(json.dumps({"synthetic": True, "session": "semantic-demo"}))
project = root / "project"
project.mkdir(exist_ok=True)
sessions = root / "codex/sessions/2026/10/27"
sessions.mkdir(parents=True, exist_ok=True)
timestamp = datetime.datetime.now(datetime.timezone.utc).isoformat()
def recorded_plan(call, entries):
    return {"timestamp": timestamp, "type": "response_item", "payload": {
        "type": "function_call", "name": "update_plan", "call_id": call,
        "arguments": json.dumps({"plan": entries})}}
records = [
    {"type": "session_meta", "payload": {"id": "semantic-demo", "session_id": "semantic-demo",
     "cwd": str(project), "thread_source": "user", "source": "cli"}},
    {"timestamp": timestamp, "type": "response_item", "payload": {"type": "message",
     "role": "user", "content": [{"type": "input_text", "text": "Semantic resource fixture"}]}},
    recorded_plan("root-plan", [{"step": "Original reported plan", "status": "in_progress"}]),
]
(sessions / "rollout-root.jsonl").write_text("".join(json.dumps(r)+"\n" for r in records))
child = [
    {"type": "session_meta", "payload": {"id": "semantic-helper", "session_id": "semantic-demo",
     "cwd": str(project), "thread_source": "subagent", "parent_thread_id": "semantic-demo",
     "agent_nickname": "Steering helper"}},
    recorded_plan("helper-plan", [{"step": "Check recipient visibility", "status": "completed"},
                                  {"step": "Review available actions", "status": "in_progress"}]),
]
(sessions / "rollout-helper.jsonl").write_text("".join(json.dumps(r)+"\n" for r in child))
def apply(update):
    result = subprocess.run([program, "state", "--root", str(root/"state"), "apply", "codex",
                             "semantic-demo", json.dumps(update)], check=True, capture_output=True)
    return json.loads(result.stdout)
session = {"provider": "codex", "session_id": "semantic-demo"}
owner = {"session": session, "agent_id": "main"}
apply({"op":"set_goal", "title":"Semantic resource fixture",
       "criteria":[{"id":"reporting","label":"Reporting survives restart"},
                   {"id":"steering","label":"Steering availability is visible"},
                   {"id":"work","label":"Current work is understandable"},
                   {"id":"checks","label":"Final checks are recorded"}],
       "reason":"Synthetic manual fixture", "provenance":"fixture"})
items = [
    {"id":"reporting", "label":"Record durable work", "status":"done", "depends_on":[],
     "purpose":"Keep meaningful work across restarts",
     "outcome_links":[{"criterion_id":"reporting","goal_revision":1}]},
    {"id":"steering", "label":"Expose steering controls", "status":"active",
     "depends_on":["reporting"], "purpose":"Show the actual recipient before sending",
     "change":"Added discoverable actions", "next":"Inspect narrow-terminal layout",
     "outcome_links":[{"criterion_id":"steering","goal_revision":1}]},
    {"id":"verification", "label":"Validate the combined behavior", "status":"blocked",
     "depends_on":["steering", "unreported-final-review"],
     "blocker":"Waiting for the explicitly referenced review",
     "outcome_links":[{"criterion_id":"checks","goal_revision":1}]},
]
for item in items:
    item.update({"owner":owner,"source":"synthetic fixture"})
    apply({"op":"upsert_work_item", "expected_revision":0, "item":item})
apply({"op":"verify", "id":"reporting", "revision":1, "evidence":[
       {"kind":"user_attestation","detail":"Synthetic fixture attestation; no real goal claim",
        "provenance":"fixture"}]})
print(json.dumps({"root":str(root), "synthetic":True, "work_items":len(items),
                  "native_plan_producers":2, "live_control":"unavailable by design"}))
