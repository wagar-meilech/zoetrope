#![cfg(all(feature = "native", unix))]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use zoetrope::documents::{DocumentLaunchKind, DocumentLauncher};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct EditorFixture {
    root: PathBuf,
}

impl EditorFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "zoetrope-document-launch-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let script = format!(
            r#"#!/usr/bin/python3
import json, pathlib, sys, urllib.parse
root = pathlib.Path({root})
args = sys.argv[1:]
record = {{"program": pathlib.Path(sys.argv[0]).name, "args": args}}
if args == ["--list-extensions"]:
    print((root / "extensions.txt").read_text())
elif record["program"] == "open":
    uri = urllib.parse.urlsplit(args[0])
    target = urllib.parse.urlsplit(urllib.parse.parse_qs(uri.query)["uri"][0])
    record["selected_file"] = urllib.parse.unquote(target.path)
else:
    record["selected_file"] = args[-1]
with (root / "record.jsonl").open("a") as out:
    out.write(json.dumps(record) + "\n")
exit_file = root / (record["program"] + "-exit")
if exit_file.exists():
    sys.exit(int(exit_file.read_text()))
"#,
            root = serde_json::to_string(root.to_str().unwrap()).unwrap()
        );
        fs::write(
            root.join("extensions.txt"),
            "AnandShah.mdreader\nwagar.zoetrope-preview",
        )
        .unwrap();
        for program in ["code", "open"] {
            let path = root.join(program);
            fs::write(&path, &script).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        Self { root }
    }

    fn launcher(&self) -> DocumentLauncher {
        DocumentLauncher::new(self.root.join("code"), self.root.join("open"))
    }

    fn records(&self) -> Vec<serde_json::Value> {
        fs::read_to_string(self.root.join("record.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

impl Drop for EditorFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn requests_preview_for_the_exact_selected_file_without_interpreting_its_name() {
    let editor = EditorFixture::new();
    let path = editor.root.join("报告 space $(touch NEVER); & #+%.md");
    fs::write(&path, "# Selected document").unwrap();

    let launch = editor.launcher().open(&path).unwrap();

    assert_eq!(launch.path, path);
    assert_eq!(launch.kind, DocumentLaunchKind::PreviewRequested);
    assert_eq!(
        launch.to_string(),
        format!("Requested VS Code preview: {}", path.display())
    );
    let records = editor.records();
    assert_eq!(records[0]["args"], serde_json::json!(["--list-extensions"]));
    assert_eq!(records[1]["program"], "open");
    assert_eq!(records[1]["args"].as_array().unwrap().len(), 1);
    assert!(
        records[1]["args"][0]
            .as_str()
            .unwrap()
            .starts_with("vscode://wagar.zoetrope-preview/open?uri=file%3A%2F%2F%2F")
    );
    assert_eq!(records[1]["selected_file"], path.to_str().unwrap());
}

#[test]
fn unavailable_documents_report_a_clear_error_without_launching_an_editor() {
    let editor = EditorFixture::new();
    let missing = editor.root.join("deleted.md");

    let error = editor.launcher().open(&missing).unwrap_err();

    assert_eq!(error.path, missing);
    assert!(error.to_string().contains("document is unavailable"));
    assert!(!editor.root.join("record.jsonl").exists());
    let error = editor.launcher().open(&editor.root).unwrap_err();
    assert!(error.to_string().contains("not a regular file"));
    assert!(!editor.root.join("record.jsonl").exists());
}

#[test]
fn an_unavailable_bridge_requests_the_selected_file_in_the_plain_editor() {
    let editor = EditorFixture::new();
    fs::write(editor.root.join("extensions.txt"), "AnandShah.mdreader").unwrap();
    let path = editor.root.join("plain 报告 & $().md");
    fs::write(&path, "# Fallback").unwrap();

    let launch = editor.launcher().open(&path).unwrap();

    assert_eq!(launch.path, path);
    assert!(matches!(
        launch.kind,
        DocumentLaunchKind::EditorRequested { .. }
    ));
    assert!(launch.to_string().contains("Requested VS Code file"));
    assert!(launch.to_string().contains("preview bridge is unavailable"));
    let records = editor.records();
    assert_eq!(records.len(), 2);
    assert_eq!(records[1]["program"], "code");
    assert_eq!(
        records[1]["args"],
        serde_json::json!(["--reuse-window", path])
    );
    assert_eq!(records[1]["selected_file"], path.to_str().unwrap());
}

#[test]
fn failed_uri_dispatch_falls_back_to_the_selected_file_instead_of_claiming_a_preview() {
    let editor = EditorFixture::new();
    fs::write(editor.root.join("open-exit"), "1").unwrap();
    let path = editor.root.join("fallback.md");
    fs::write(&path, "# Fallback").unwrap();

    let launch = editor.launcher().open(&path).unwrap();

    assert!(matches!(
        launch.kind,
        DocumentLaunchKind::EditorRequested { .. }
    ));
    assert!(launch.to_string().contains("preview dispatch failed"));
    let records = editor.records();
    assert_eq!(records.last().unwrap()["program"], "code");
    assert_eq!(
        records.last().unwrap()["args"],
        serde_json::json!(["--reuse-window", path])
    );
}

#[test]
fn non_markdown_files_cannot_be_dispatched_as_reader_documents() {
    let editor = EditorFixture::new();
    let path = editor.root.join("script.command");
    fs::write(&path, "echo this is not Markdown").unwrap();

    let error = editor.launcher().open(&path).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("select a Markdown file (.md or .markdown)")
    );
    assert!(!editor.root.join("record.jsonl").exists());
}
