use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::{buffer::Buffer, layout::Rect, widgets::Paragraph, widgets::Widget};
use zoetrope::console::{
    AgentKey, Console, ConsoleCommand, DiscoveryRoots, FleetNodeId, SessionKey,
};
use zoetrope::provider::Provider;

struct Home(PathBuf);

impl Home {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let unique = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "zoetrope-console-{}-{nonce}-{unique}",
            std::process::id()
        )))
    }

    fn write(&self, path: &str, text: &str) -> PathBuf {
        let path = self.0.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
        path
    }

    fn console(&self) -> Console {
        Console::new(DiscoveryRoots {
            claude_projects: Some(self.0.join("claude/projects")),
            codex_home: Some(self.0.join("codex")),
        })
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn rendered(console: &Console) -> String {
    let area = Rect::new(0, 0, 100, 30);
    let mut buffer = Buffer::empty(area);
    Paragraph::new(console.render()).render(area, &mut buffer);
    buffer.content().iter().map(|c| c.symbol()).collect()
}

fn codex_meta(id: &str, project: &str) -> String {
    format!(
        "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"{id}\",\"session_id\":\"{id}\",\"cwd\":\"{project}\",\"thread_source\":\"user\",\"source\":\"cli\"}}}}\n"
    )
}

#[test]
fn opening_the_console_discovers_projects_from_both_providers() {
    let home = Home::new();
    home.write(
        "codex/sessions/2026/10/23/rollout-2026-10-23T10-00-00-codex-root.jsonl",
        &codex_meta("codex-root", "/projects/compiler"),
    );
    home.write(
        "claude/projects/-projects-editor/aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee.jsonl",
        "{\"type\":\"ai-title\",\"aiTitle\":\"Improve editor\"}\n",
    );
    let mut console = home.console();
    console.refresh(SystemTime::now());
    let output = rendered(&console);
    assert!(output.contains("/projects/compiler"), "{output}");
    assert!(output.contains("-projects-editor"), "{output}");
    assert!(output.contains("Improve editor"), "{output}");
    assert!(output.contains("codex-root"), "{output}");
}

#[test]
fn history_includes_archived_work_without_making_it_current() {
    let home = Home::new();
    home.write(
        "codex/archived_sessions/rollout-2026-09-12T10-00-00-archived-goal.jsonl",
        &codex_meta("archived-goal", "/projects/old-work"),
    );
    home.write(
        "codex/sessions/2026/10/23/rollout-broken.jsonl",
        "not json\n",
    );
    let mut console = home.console();
    console.refresh(SystemTime::now());
    assert!(!rendered(&console).contains("archived-goal"));
    console.command(ConsoleCommand::ToggleHistory);
    let output = rendered(&console);
    assert!(output.contains("archived-goal"), "{output}");
    assert!(output.contains("archived"), "{output}");
    console.command(ConsoleCommand::ToggleHistory);
    assert!(!rendered(&console).contains("archived-goal"));
}

#[test]
fn expanding_a_goal_keeps_nested_agents_and_internal_reviews_under_their_owner() {
    let home = Home::new();
    let day = "codex/sessions/2026/10/23";
    home.write(
        &format!("{day}/rollout-root.jsonl"),
        &codex_meta("root", "/projects/compiler"),
    );
    home.write(&format!("{day}/rollout-child.jsonl"),
        "{\"type\":\"session_meta\",\"payload\":{\"id\":\"child\",\"session_id\":\"root\",\"thread_source\":\"subagent\",\"parent_thread_id\":\"root\",\"agent_nickname\":\"Parser\"}}\n");
    home.write(&format!("{day}/rollout-grandchild.jsonl"),
        "{\"type\":\"session_meta\",\"payload\":{\"id\":\"grandchild\",\"session_id\":\"root\",\"thread_source\":\"subagent\",\"parent_thread_id\":\"child\",\"agent_nickname\":\"Tests\"}}\n");
    home.write(&format!("{day}/rollout-review.jsonl"),
        "{\"type\":\"session_meta\",\"payload\":{\"id\":\"review\",\"session_id\":\"root\",\"thread_source\":\"guardian_review\",\"parent_thread_id\":\"child\",\"source\":{\"subagent\":{\"other\":\"guardian\"}}}}\n");
    let session = SessionKey {
        provider: Provider::Codex,
        session_id: "root".into(),
    };
    let child = AgentKey {
        session: session.clone(),
        agent_id: "child".into(),
    };
    let mut console = home.console();
    console.refresh(SystemTime::now());
    assert_eq!(
        console
            .graph()
            .nodes
            .iter()
            .filter(|n| matches!(n.id, FleetNodeId::Session(_)))
            .count(),
        1
    );
    console.command(ConsoleCommand::ToggleExpanded(session.clone()));
    let graph = console.graph();
    let grandchild = graph
        .nodes
        .iter()
        .find(|n| n.label == "Tests")
        .expect("nested agent visible");
    assert_eq!(grandchild.parent, Some(FleetNodeId::Agent(child.clone())));
    assert!(!rendered(&console).contains("approval review"));
    console.command(ConsoleCommand::ToggleInternal);
    let graph = console.graph();
    let review = graph
        .nodes
        .iter()
        .find(|n| n.label == "approval review")
        .expect("helper can be revealed");
    assert_eq!(review.parent, Some(FleetNodeId::Agent(child)));
    assert!(review.internal);
}

#[test]
fn refresh_keeps_the_selected_agent_in_its_own_session() {
    let home = Home::new();
    for root in ["first", "second"] {
        home.write(
            &format!("codex/sessions/2026/10/23/rollout-{root}.jsonl"),
            &codex_meta(root, "/projects/compiler"),
        );
        home.write(&format!("codex/sessions/2026/10/23/rollout-{root}-child.jsonl"),
            &format!("{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"same-agent-id\",\"session_id\":\"{root}\",\"thread_source\":\"subagent\",\"parent_thread_id\":\"{root}\"}}}}\n"));
    }
    let mut console = home.console();
    console.refresh(SystemTime::now());
    let second = SessionKey {
        provider: Provider::Codex,
        session_id: "second".into(),
    };
    console.command(ConsoleCommand::ToggleExpanded(second.clone()));
    let selected = FleetNodeId::Agent(AgentKey {
        session: second,
        agent_id: "same-agent-id".into(),
    });
    console.command(ConsoleCommand::Select(selected.clone()));
    home.write(
        "codex/sessions/2026/10/23/rollout-newest.jsonl",
        &codex_meta("newest", "/projects/other"),
    );
    console.refresh(SystemTime::now());
    assert_eq!(console.selection(), Some(&selected));
    assert_eq!(
        console.selected_session().map(|s| s.id.as_str()),
        Some("second")
    );
    assert!(console.graph().nodes.iter().any(|n| n.id == selected));
}

#[test]
fn search_finds_a_collapsed_agents_name_and_preserves_the_selected_goal() {
    let home = Home::new();
    home.write(
        "codex/sessions/2026/10/23/rollout-root.jsonl",
        &codex_meta("root", "/projects/compiler"),
    );
    home.write(
        "codex/sessions/2026/10/23/rollout-unrelated.jsonl",
        &codex_meta("unrelated", "/projects/editor"),
    );
    home.write("codex/sessions/2026/10/23/rollout-child.jsonl",
        "{\"type\":\"session_meta\",\"payload\":{\"id\":\"child\",\"session_id\":\"root\",\"thread_source\":\"subagent\",\"parent_thread_id\":\"root\",\"agent_nickname\":\"Parser Specialist\"}}\n");
    let mut console = home.console();
    console.refresh(SystemTime::now());
    let selected = FleetNodeId::Session(SessionKey {
        provider: Provider::Codex,
        session_id: "root".into(),
    });
    console.command(ConsoleCommand::Select(selected.clone()));
    console.command(ConsoleCommand::Search("pArSeR".into()));
    let output = rendered(&console);
    assert!(output.contains("Parser Specialist"), "{output}");
    assert!(!output.contains("unrelated"), "{output}");
    assert_eq!(console.selection(), Some(&selected));
    console.command(ConsoleCommand::Search("nothing matches".into()));
    assert_eq!(console.selection(), Some(&selected));
    console.command(ConsoleCommand::Search(String::new()));
    assert!(rendered(&console).contains("unrelated"));
    assert_eq!(console.selection(), Some(&selected));
}

#[test]
fn large_history_keeps_its_latest_title_and_indexes_new_metadata() {
    let home = Home::new();
    let path = home.write(
        "claude/projects/-projects-editor/aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee.jsonl",
        &format!("{{\"type\":\"ai-title\",\"aiTitle\":\"Initial title\"}}\n{{\"type\":\"attachment\",\"padding\":\"{}\"}}\n{{\"type\":\"ai-title\",\"aiTitle\":\"Finished editor\"}}\n", "x".repeat(1024 * 1024)),
    );
    let mut console = home.console();
    console.refresh(SystemTime::now());
    assert!(rendered(&console).contains("Finished editor"));
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    writeln!(
        file,
        "{{\"type\":\"ai-title\",\"aiTitle\":\"Editor resumed\"}}"
    )
    .unwrap();
    console.refresh(SystemTime::now());
    assert!(rendered(&console).contains("Editor resumed"));
    console.refresh(SystemTime::now());
    assert!(rendered(&console).contains("Editor resumed"));
}

#[test]
fn expanding_a_claude_goal_discovers_its_agents_without_opening_replay() {
    let home = Home::new();
    let id = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
    home.write(
        &format!("claude/projects/-projects-editor/{id}.jsonl"),
        "{\"type\":\"ai-title\",\"aiTitle\":\"Editor\"}\n",
    );
    home.write(
        &format!("claude/projects/-projects-editor/{id}/subagents/agent-syntax.jsonl"),
        "{\"type\":\"assistant\",\"agentId\":\"syntax\",\"message\":{\"content\":[]}}\n",
    );
    home.write(
        &format!("claude/projects/-projects-editor/{id}/subagents/agent-syntax.meta.json"),
        "{\"agentType\":\"Explore\",\"description\":\"Syntax worker\"}",
    );
    let mut console = home.console();
    console.refresh(SystemTime::now());
    assert!(!rendered(&console).contains("Syntax worker"));
    console.command(ConsoleCommand::ToggleExpanded(SessionKey {
        provider: Provider::Claude,
        session_id: id.into(),
    }));
    assert!(rendered(&console).contains("Syntax worker"));
    assert_eq!(
        console
            .graph()
            .nodes
            .iter()
            .filter(|n| matches!(&n.id, FleetNodeId::Agent(k) if k.agent_id == "syntax"))
            .count(),
        1
    );
}

#[test]
fn codex_history_uses_searchable_native_thread_names_and_observes_renames() {
    let home = Home::new();
    home.write(
        "codex/archived_sessions/rollout-history.jsonl",
        &codex_meta("history", "/projects/compiler"),
    );
    let index = home.write("codex/session_index.jsonl", "{\"id\":\"history\",\"thread_name\":\"Repair Unicode parser\",\"updated_at\":\"2026-10-22T10:00:00Z\"}\n");
    let mut console = home.console();
    console.refresh(SystemTime::now());
    console.command(ConsoleCommand::ToggleHistory);
    console.command(ConsoleCommand::Search("unicode".into()));
    assert!(rendered(&console).contains("Repair Unicode parser"));
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(index)
        .unwrap();
    writeln!(file, "{{\"id\":\"history\",\"thread_name\":\"Parser follow-up\",\"updated_at\":\"2026-10-23T10:00:00Z\"}}").unwrap();
    console.refresh(SystemTime::now());
    console.command(ConsoleCommand::Search("follow-up".into()));
    assert!(rendered(&console).contains("Parser follow-up"));
}

#[test]
fn graph_keeps_each_projects_goals_together_when_recent_writes_interleave() {
    let home = Home::new();
    let now = SystemTime::now();
    for (id, project, age) in [
        ("compiler-new", "/projects/compiler", 1),
        ("editor", "/projects/editor", 2),
        ("compiler-old", "/projects/compiler", 3),
    ] {
        let path = home.write(
            &format!("codex/sessions/2026/10/23/rollout-{id}.jsonl"),
            &codex_meta(id, project),
        );
        std::fs::File::open(path)
            .unwrap()
            .set_modified(now - std::time::Duration::from_secs(age))
            .unwrap();
    }
    let mut console = home.console();
    console.refresh(now);
    let labels: Vec<_> = console
        .graph()
        .nodes
        .iter()
        .map(|node| node.label.as_str())
        .collect();
    assert_eq!(
        labels,
        [
            "/projects/compiler",
            "compiler-new",
            "compiler-old",
            "/projects/editor",
            "editor"
        ]
    );
}

#[test]
fn an_indexed_claude_agent_stays_searchable_after_collapsing_and_refreshing() {
    let home = Home::new();
    let id = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
    home.write(
        &format!("claude/projects/-projects-editor/{id}.jsonl"),
        "{\"type\":\"ai-title\",\"aiTitle\":\"Editor\"}\n",
    );
    home.write(
        &format!("claude/projects/-projects-editor/{id}/subagents/agent-syntax.jsonl"),
        "{}\n",
    );
    home.write(
        &format!("claude/projects/-projects-editor/{id}/subagents/agent-syntax.meta.json"),
        "{\"agentType\":\"Explore\",\"description\":\"Syntax worker\"}",
    );
    let session = SessionKey {
        provider: Provider::Claude,
        session_id: id.into(),
    };
    let mut console = home.console();
    console.refresh(SystemTime::now());
    console.command(ConsoleCommand::ToggleExpanded(session.clone()));
    console.command(ConsoleCommand::ToggleExpanded(session));
    console.refresh(SystemTime::now());
    console.command(ConsoleCommand::Search("Syntax worker".into()));
    assert!(rendered(&console).contains("Syntax worker"));
}
