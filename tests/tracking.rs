#![cfg(feature = "native")]

use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Output};

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "zoe-tracking-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn run(&self, operation: &str, extras: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_zoe"))
            .args(["tracking", operation, "--project"])
            .arg(&self.0)
            .args(["--provider", "codex", "--session", "goal", "--root"])
            .arg(self.0.join("state"))
            .args(extras)
            .output()
            .unwrap()
    }
    fn ok(&self, operation: &str, extras: &[&str]) -> Value {
        let output = self.run(operation, extras);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
    fn write(&self, name: &str, contents: &str) {
        std::fs::write(self.0.join(name), contents).unwrap();
    }
    fn show(&self) -> Value {
        let output = Command::new(env!("CARGO_BIN_EXE_zoe"))
            .args(["state", "--root"])
            .arg(self.0.join("state"))
            .args(["show", "codex", "goal"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
}

#[test]
fn designated_table_imports_stable_tasks_with_honest_owners_and_never_verifies() {
    let project = Project::new();
    project.write("WORK.md", "## Unrelated\n| ID | Task | Status |\n| --- | --- | --- |\n| WRONG | Ignore | done |\n\n## Tasks\n| ID | Task | Status | Owner | Blocked by | Purpose |\n| --- | --- | --- | --- | --- | --- |\n| T1 | Escape \\| pipes | active | main | None | Fix it |\n| T2 | [Human review](https://example.invalid/issue/2) | done | Alice | T1 | |\n");
    project.ok("enable", &["--source", "WORK.md#Tasks"]);
    let shown = project.show();
    let tasks = shown["record"]["source_tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["label"], "Escape | pipes");
    assert_eq!(tasks[0]["status"], "active");
    assert_eq!(tasks[0]["owner"]["agent_id"], "main");
    assert_eq!(tasks[0]["depends_on"], serde_json::json!([]));
    assert_eq!(tasks[1]["owner"], Value::Null);
    assert_eq!(tasks[1]["owner_label"], "Alice");
    assert_eq!(tasks[1]["depends_on"][0], tasks[0]["id"]);
    assert_eq!(shown["progress"]["percent"], Value::Null);
    let stable_id = tasks[0]["id"].clone();
    project.write(
        "WORK.md",
        "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| T1 | Renamed | done |\n",
    );
    project.ok("sync", &[]);
    let changed = project.show();
    assert_eq!(changed["record"]["source_tasks"][0]["id"], stable_id);
    assert_eq!(changed["record"]["source_tasks"][0]["label"], "Renamed");
    assert_eq!(changed["record"]["source_tasks"][0]["status"], "done");
    assert_eq!(
        changed["record"]["source_tasks"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn ordinary_file_edits_refresh_on_the_existing_console_worker_path_without_sync() {
    use std::time::SystemTime;
    use zoetrope::console::{Console, DiscoveryRoots, SessionKey};
    use zoetrope::provider::Provider;
    let project = Project::new();
    let sessions = project.0.join("codex/sessions/2026/10/27");
    std::fs::create_dir_all(&sessions).unwrap();
    std::fs::write(sessions.join("rollout-goal.jsonl"), format!("{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"goal\",\"session_id\":\"goal\",\"thread_source\":\"user\",\"cwd\":{}}}}}\n",serde_json::to_string(&project.0).unwrap())).unwrap();
    project.write(
        "WORK.md",
        "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| T1 | First title | active |\n",
    );
    project.ok("enable", &["--source", "WORK.md#Tasks"]);
    let roots = DiscoveryRoots {
        claude_projects: None,
        codex_home: Some(project.0.join("codex")),
    };
    let key = SessionKey {
        provider: Provider::Codex,
        session_id: "goal".into(),
    };
    let mut console = Console::with_state(roots, project.0.join("state"));
    console.refresh(SystemTime::now());
    assert_eq!(
        console.work(&key).unwrap().source_tasks[0].label,
        "First title"
    );
    project.write("WORK.md", "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| T1 | Normal edit updates graph | done |\n");
    console.refresh(SystemTime::now());
    assert_eq!(
        console.work(&key).unwrap().source_tasks[0].label,
        "Normal edit updates graph"
    );
    project.ok("disable", &[]);
    project.write(
        "WORK.md",
        "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| T1 | Disabled edit | planned |\n",
    );
    console.refresh(SystemTime::now());
    assert_eq!(
        console.work(&key).unwrap().source_tasks[0].label,
        "Normal edit updates graph"
    );
}

#[test]
fn invalid_missing_and_duplicate_sources_keep_last_good_until_valid_empty_removes_work() {
    let project = Project::new();
    let valid =
        "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| T1 | Keep last good | done |\n";
    project.write("WORK.md", valid);
    project.ok("enable", &["--source", "WORK.md#Tasks"]);
    let original = project.show()["record"]["source_tasks"][0].clone();
    for invalid in [
        "## Tasks\nTemporarily editing table\n".to_owned(),
        "## Tasks\n|\n|---|\n".to_owned(),
        format!("{valid}| T1 | Duplicate | active |\n"),
        "X".repeat(256 * 1024 + 1),
    ] {
        project.write("WORK.md", &invalid);
        assert_eq!(project.ok("sync", &[])["sources"][0]["stale"], true);
        let shown = project.show();
        assert_eq!(
            shown["record"]["source_tasks"][0]["label"],
            "Keep last good"
        );
        assert_eq!(shown["record"]["source_tasks"][0]["stale"], true);
    }
    std::fs::remove_file(project.0.join("WORK.md")).unwrap();
    assert_eq!(project.ok("sync", &[])["sources"][0]["stale"], true);
    project.write("WORK.md", valid);
    assert_eq!(project.ok("sync", &[])["sources"][0]["stale"], false);
    assert_eq!(
        project.show()["record"]["source_tasks"][0]["id"],
        original["id"]
    );
    project.write(
        "WORK.md",
        "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n",
    );
    assert_eq!(project.ok("sync", &[])["sources"][0]["tasks"], 0);
    assert_eq!(project.ok("status", &[])["sources"][0]["retired"], 1);
    assert!(
        project.show()["record"]["source_tasks"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn explicit_ticket_metadata_stops_at_the_body_and_ignores_fenced_examples() {
    let project = Project::new();
    project.write("T1.md", "# Repair save behavior\n\n**ID:** T1\n**Status:** blocked\n**Owner:** Unassigned\n**Blocked by:** None\n**Next:** Ask operator\n\n## Acceptance\nStatus: done\n```md\nID: BAD\nStatus: done\n```\n");
    let status = project.ok("enable", &["--source", "T1.md@ticket"]);
    assert_eq!(status["sources"][0]["stale"], false);
    let shown = project.show();
    let task = &shown["record"]["source_tasks"][0];
    assert_eq!(task["task_id"], "T1");
    assert_eq!(task["label"], "Repair save behavior");
    assert_eq!(task["status"], "blocked");
    assert_eq!(task["next"], "Ask operator");
    assert_eq!(task["owner"], Value::Null);
}

#[test]
fn source_owned_fields_win_and_clears_do_not_revive_supplements() {
    let project = Project::new();
    project.write("WORK.md", "## Tasks\n| ID | Task | Status | Owner |\n| --- | --- | --- | --- |\n| T1 | Source title | planned | Alice |\n");
    project.ok("enable", &["--source", "WORK.md#Tasks"]);
    let id = project.show()["record"]["source_tasks"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let update = serde_json::json!({"op":"upsert_work_item","expected_revision":0,"item":{"id":id,"label":"Conflicting report title","status":"done","owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},"source":"supplement","purpose":"Context only in report","next":"Run checks"}});
    let output = Command::new(env!("CARGO_BIN_EXE_zoe"))
        .args(["state", "--root"])
        .arg(project.0.join("state"))
        .args(["apply", "codex", "goal", &update.to_string()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let shown = project.show();
    let task = &shown["record"]["source_tasks"][0];
    assert_eq!(task["label"], "Source title");
    assert_eq!(task["status"], "planned");
    assert_eq!(task["owner_label"], "Alice");
    assert_eq!(task["owner"], Value::Null);
    assert_eq!(task["purpose"], "Context only in report");
    assert!(
        task["provenance"]["purpose"]
            .as_str()
            .unwrap()
            .contains("supplement")
    );
    project.write("WORK.md", "## Tasks\n| ID | Task | Status | Next |\n| --- | --- | --- | --- |\n| T1 | Duplicate | planned | |\n| T1 | Duplicate | done | |\n");
    project.ok("sync", &[]);
    assert_eq!(
        project.show()["record"]["source_tasks"][0]["next"],
        "Run checks",
        "invalid edits must retain prior field authority as well as data"
    );
    project.write("WORK.md", "## Tasks\n| ID | Task | Status | Next |\n| --- | --- | --- | --- |\n| T1 | Source title | planned | |\n");
    project.ok("sync", &[]);
    let cleared = project.show();
    assert_eq!(cleared["record"]["source_tasks"][0]["owner"], Value::Null);
    assert_eq!(cleared["record"]["source_tasks"][0]["next"], Value::Null);
}

#[test]
fn automatic_identity_validates_exact_runtime_root_and_current_child_headers() {
    let project = Project::new();
    let sessions = project.0.join("codex/sessions/2026/10/27");
    std::fs::create_dir_all(&sessions).unwrap();
    std::fs::write(sessions.join("rollout-root.jsonl"), "{\"type\":\"session_meta\",\"payload\":{\"id\":\"root\",\"session_id\":\"root\",\"thread_source\":\"user\"}}\n").unwrap();
    std::fs::write(sessions.join("rollout-child.jsonl"), "{\"type\":\"session_meta\",\"payload\":{\"id\":\"child\",\"session_id\":\"root\",\"parent_thread_id\":\"root\",\"thread_source\":\"subagent\"}}\n").unwrap();
    project.write(
        "WORK.md",
        "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n",
    );
    let run = |root_id: &str| {
        Command::new(env!("CARGO_BIN_EXE_zoe"))
            .args(["tracking", "enable", "--project"])
            .arg(&project.0)
            .args([
                "--provider",
                "codex",
                "--session",
                "auto",
                "--source",
                "WORK.md#Tasks",
                "--root",
            ])
            .arg(project.0.join("state"))
            .env("CODEX_HOME", project.0.join("codex"))
            .env("CODEX_SESSION_ID", root_id)
            .env("CODEX_THREAD_ID", "child")
            .output()
            .unwrap()
    };
    let matched = run("root");
    assert!(
        matched.status.success(),
        "{}",
        String::from_utf8_lossy(&matched.stderr)
    );
    let status: Value = serde_json::from_slice(&matched.stdout).unwrap();
    assert_eq!(status["session"]["session_id"], "root");
    assert!(!run("other-root").status.success());
}

#[test]
fn multiple_goal_scopes_stay_isolated_and_disabling_project_stops_every_binding() {
    let project = Project::new();
    project.write("ONE.md", "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| T1 | First goal only | active |\n");
    project.write("TWO.md", "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| T1 | Second goal only | blocked |\n");
    project.ok("enable", &["--source", "ONE.md#Tasks"]);
    project.ok(
        "enable",
        &["--session", "second-goal", "--source", "TWO.md#Tasks"],
    );
    assert_eq!(
        project.show()["record"]["source_tasks"][0]["label"],
        "First goal only"
    );
    let disabled = project.ok("disable", &[]);
    assert_eq!(disabled["project_opt_in"], false);
    assert_eq!(
        project.ok("status", &["--session", "second-goal"])["enabled"],
        false
    );
    project.write("ONE.md", "not valid now");
    assert_eq!(project.ok("sync", &[])["sources"][0]["stale"], false);
}

#[test]
fn canonical_sources_replace_parallel_old_plans_but_invalid_initial_sources_keep_fallback() {
    let project = Project::new();
    let update = serde_json::json!({"op":"upsert_work_item","expected_revision":0,"item":{"id":"old-plan","label":"Historical explicit plan","status":"active","owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},"source":"explicit report"}});
    let output = Command::new(env!("CARGO_BIN_EXE_zoe"))
        .args(["state", "--root"])
        .arg(project.0.join("state"))
        .args(["apply", "codex", "goal", &update.to_string()])
        .output()
        .unwrap();
    assert!(output.status.success());
    project.write("WORK.md", "table not finished");
    project.ok("enable", &["--source", "WORK.md#Tasks"]);
    let context = || {
        let output = Command::new(env!("CARGO_BIN_EXE_zoe"))
            .args(["state", "--root"])
            .arg(project.0.join("state"))
            .args(["context", "codex", "goal"])
            .output()
            .unwrap();
        assert!(output.status.success());
        serde_json::from_slice::<Value>(&output.stdout).unwrap()
    };
    assert_eq!(
        context()["work_items"][0]["label"],
        "Historical explicit plan"
    );
    project.write("WORK.md", "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| T1 | Canonical work | planned |\n");
    project.ok("sync", &[]);
    let current = context();
    assert!(current["work_items"].as_array().unwrap().is_empty());
    assert_eq!(current["source_snapshot"], true);
    assert_eq!(current["source_tasks"][0]["label"], "Canonical work");
    assert_eq!(
        project.show()["record"]["work_items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    project.write("WORK.md", "malformed");
    project.ok("sync", &[]);
    assert_eq!(context()["source_tasks"][0]["label"], "Canonical work");
    project.write(
        "WORK.md",
        "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n",
    );
    project.ok("sync", &[]);
    assert!(context()["work_items"].as_array().unwrap().is_empty());
    assert!(context()["source_tasks"].as_array().unwrap().is_empty());
}

#[test]
fn bounded_refresh_rejects_excess_rows_and_unchanged_content_is_a_persistent_noop() {
    let project = Project::new();
    let mut table = "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n".to_owned();
    for index in 0..512 {
        table.push_str(&format!("| T{index} | Work {index} | planned |\n"));
    }
    project.write("WORK.md", &table);
    let enabled = project.ok("enable", &["--source", "WORK.md#Tasks"]);
    assert_eq!(enabled["sources"][0]["tasks"], 512);
    let cache = project.0.join("state/tracking.json");
    let unchanged = std::fs::metadata(&cache).unwrap().modified().unwrap();
    project.write("WORK.md", &table);
    assert_eq!(project.ok("sync", &[]), enabled);
    assert_eq!(
        std::fs::metadata(&cache).unwrap().modified().unwrap(),
        unchanged
    );
    table.push_str("| TOO-MANY | Rejected | planned |\n");
    project.write("WORK.md", &table);
    assert_eq!(project.ok("sync", &[])["sources"][0]["stale"], true);
    assert_eq!(
        project.show()["record"]["source_tasks"]
            .as_array()
            .unwrap()
            .len(),
        512
    );
    // Several sources in one goal share a bounded effective task population.
    project.write(
        "OTHER.md",
        "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| X | Other | active |\n",
    );
    let failed = project.run(
        "enable",
        &["--source", "WORK.md#Tasks", "--source", "OTHER.md#Tasks"],
    );
    assert!(!failed.status.success());
}

#[test]
fn reenable_of_a_missing_allowlisted_file_retains_last_good_instead_of_losing_binding() {
    let project = Project::new();
    project.write(
        "WORK.md",
        "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| T1 | Saved source | planned |\n",
    );
    project.ok("enable", &["--source", "WORK.md#Tasks"]);
    project.ok("disable", &[]);
    std::fs::remove_file(project.0.join("WORK.md")).unwrap();
    let enabled = project.ok("enable", &["--source", "WORK.md#Tasks"]);
    assert_eq!(enabled["enabled"], true);
    assert_eq!(enabled["sources"][0]["stale"], true);
    assert_eq!(
        project.show()["record"]["source_tasks"][0]["label"],
        "Saved source"
    );
}

#[cfg(unix)]
#[test]
fn allowlist_rejects_symlinks_outside_the_selected_project() {
    let project = Project::new();
    let unrelated = Project::new();
    unrelated.write(
        "PRIVATE.md",
        "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| SECRET | Never import | done |\n",
    );
    std::os::unix::fs::symlink(unrelated.0.join("PRIVATE.md"), project.0.join("LINK.md")).unwrap();
    let result = project.run("enable", &["--source", "LINK.md#Tasks"]);
    assert!(!result.status.success());
    assert!(
        project.show()["record"]["source_tasks"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn ticket_set_dependencies_resolve_only_unambiguous_explicit_ids_in_the_same_goal() {
    let project = Project::new();
    project.write("T1.md", "# First\nID: T1\nStatus: done\nBlocked by: None\n");
    project.write(
        "T2.md",
        "# Second\nID: T2\nStatus: blocked\nBlocked by: T1\n",
    );
    project.ok(
        "enable",
        &["--source", "T1.md@ticket", "--source", "T2.md@ticket"],
    );
    let shown = project.show();
    let tasks = shown["record"]["source_tasks"].as_array().unwrap();
    assert_eq!(tasks[1]["depends_on"][0], tasks[0]["id"]);
}

#[test]
fn fenced_task_examples_cannot_become_authoritative_tables() {
    let project = Project::new();
    project.write("WORK.md", "~~~~md\n```\n## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| WRONG | Code example | done |\n~~~~\n## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n| REAL | Actual work | active |\n");
    project.ok("enable", &["--source", "WORK.md#Tasks"]);
    assert_eq!(
        project.show()["record"]["source_tasks"][0]["task_id"],
        "REAL"
    );
}

#[test]
fn malformed_native_owner_references_remain_unavailable_without_becoming_coordinator() {
    let project = Project::new();
    project.write("WORK.md", "## Tasks\n| ID | Task | Status | Owner |\n| --- | --- | --- | --- |\n| A | Invalid native root token | planned | native:main |\n| B | Invalid native alias token | planned | native:/root/worker |\n");
    project.ok("enable", &["--source", "WORK.md#Tasks"]);
    for task in project.show()["record"]["source_tasks"].as_array().unwrap() {
        assert_eq!(task["owner"], Value::Null);
    }
}

#[test]
fn source_set_growth_beyond_goal_bounds_is_stale_and_preserves_last_good_tasks() {
    let project = Project::new();
    let table = |prefix: &str| {
        let mut table = "## Tasks\n| ID | Task | Status |\n| --- | --- | --- |\n".to_owned();
        for n in 0..256 {
            table.push_str(&format!("| {prefix}{n} | Work | planned |\n"));
        }
        table
    };
    project.write("ONE.md", &table("A"));
    project.write("TWO.md", &table("B"));
    project.ok(
        "enable",
        &["--source", "ONE.md#Tasks", "--source", "TWO.md#Tasks"],
    );
    project.write(
        "TWO.md",
        &format!("{}| EXTRA | Too many | done |\n", table("B")),
    );
    let status = project.ok("sync", &[]);
    assert_eq!(status["sources"][1]["stale"], true);
    assert_eq!(status["sources"][1]["tasks"], 256);
    assert_eq!(
        project.show()["record"]["source_tasks"]
            .as_array()
            .unwrap()
            .len(),
        512
    );
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn exact_binding_and_project_opt_in_persist_without_editing_project_instructions() {
    let project = Project::new();
    project.write("CLAUDE.md", "Keep existing instructions.\n");
    project.write(
        "WORK.md",
        "## Tasks\n\n| ID | Task | Status |\n| --- | --- | --- |\n| T1 | Ship fix | planned |\n",
    );
    let enabled = project.ok("enable", &["--source", "WORK.md#Tasks"]);
    assert_eq!(enabled["enabled"], true);
    assert_eq!(enabled["session"]["session_id"], "goal");
    assert_eq!(enabled["sources"].as_array().unwrap().len(), 1);
    assert_eq!(
        project.ok("enable", &["--source", "WORK.md#Tasks"]),
        enabled
    );
    assert_eq!(project.ok("status", &[]), enabled);
    assert_eq!(project.ok("disable", &[])["enabled"], false);
    assert_eq!(project.ok("status", &[])["enabled"], false);
    assert_eq!(
        std::fs::read_to_string(project.0.join("CLAUDE.md")).unwrap(),
        "Keep existing instructions.\n"
    );
}
