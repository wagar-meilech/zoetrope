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
