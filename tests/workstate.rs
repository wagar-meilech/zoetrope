#![cfg(feature = "native")]

use serde_json::{Value, json};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct Home(PathBuf);
impl Home {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "zoe-state-{}-{nonce}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )))
    }
    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_zoe"))
            .arg("state")
            .arg("--root")
            .arg(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
    fn apply(&self, update: Value) -> Value {
        let output = self.run(&["apply", "codex", "goal", &update.to_string()]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        self.show()
    }
    fn show(&self) -> Value {
        let output = self.run(&["show", "codex", "goal"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
}
impl Drop for Home {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn local_goal_tracking_survives_process_restart_and_missing_tracking_is_unknown() {
    let home = Home::new();
    let untracked = home.show();
    assert_eq!(untracked["progress"]["percent"], Value::Null);
    assert_eq!(untracked["progress"]["label"], "Progress not tracked");
    home.apply(
        json!({"op":"set_goal", "title":"Ship editor fix", "criteria":[
        {"id":"behavior","label":"Broken edit works"},
        {"id":"validation","label":"Regression checks pass"}
    ], "reason":"Initial user contract", "provenance":"user"}),
    );
    let restored = home.show();
    assert_eq!(restored["record"]["goal"]["title"], "Ship editor fix");
    assert_eq!(restored["record"]["goal"]["revision"], 1);
    assert_eq!(restored["progress"]["label"], "0 of 2 verified · 0%");
}

#[test]
fn verified_progress_requires_evidence_and_uses_outcome_weights_not_activity() {
    let home = Home::new();
    home.apply(json!({"op":"set_goal","title":"Ship fix","criteria":[
        {"id":"implementation","label":"Behavior works","weight":3},
        {"id":"checks","label":"Final checks pass","weight":1}
    ],"reason":"Initial scope","provenance":"user"}));
    let invalid = home.run(&[
        "apply",
        "codex",
        "goal",
        &json!({"op":"verify","id":"implementation","evidence":[]}).to_string(),
    ]);
    assert!(!invalid.status.success());
    assert_eq!(home.show()["progress"]["percent"], 0.0);
    home.apply(json!({"op":"verify","id":"implementation","evidence":[{"kind":"reference","detail":"tests/editor_regression.rs passed on abc123","provenance":"codex"}]}));
    let progress = home.show();
    assert_eq!(progress["progress"]["verified"], 1);
    assert_eq!(progress["progress"]["total"], 2);
    assert_eq!(progress["progress"]["percent"], 75.0);
    assert_eq!(progress["progress"]["label"], "1 of 2 verified · 75%");
    home.apply(json!({"op":"verify","id":"checks","evidence":[{"kind":"user_attestation","detail":"I checked the final result","provenance":"user"}]}));
    assert_eq!(home.show()["progress"]["percent"], 100.0);
}

#[test]
fn changing_scope_retains_prior_evidence_and_only_reuses_unchanged_outcomes() {
    let home = Home::new();
    let initial = json!({"op":"set_goal","title":"Improve editor","criteria":[
        {"id":"behavior","label":"Edit works"},{"id":"checks","label":"Checks pass"}
    ],"reason":"Initial scope","provenance":"user"});
    home.apply(initial);
    home.apply(json!({"op":"verify","id":"behavior","evidence":[{"kind":"reference","detail":"Regression passed","provenance":"test"}]}));
    let revised=home.apply(json!({"op":"set_goal","title":"Improve editor","criteria":[
        {"id":"behavior","label":"Edit works"},{"id":"checks","label":"Checks pass"},{"id":"docs","label":"Usage documented"}
    ],"reason":"Need usage docs too","provenance":"user"}));
    assert_eq!(revised["record"]["goal"]["revision"], 2);
    assert_eq!(revised["progress"]["label"], "1 of 3 verified · 33%");
    assert_eq!(
        revised["record"]["scope_history"][0]["goal"]["criteria"][0]["evidence"][0]["detail"],
        "Regression passed"
    );
    let changed = home.apply(json!({"op":"set_goal","title":"Improve editor","criteria":[
        {"id":"behavior","label":"Undo also works"}
    ],"reason":"Acceptance behavior changed","provenance":"user"}));
    assert_eq!(changed["progress"]["verified"], 0);
    assert_eq!(
        changed["record"]["scope_history"][1]["reason"],
        "Acceptance behavior changed"
    );
    let stale=home.run(&["apply","codex","goal",&json!({"op":"verify","id":"behavior","revision":1,"evidence":[{"kind":"reference","detail":"Old check","provenance":"test"}]}).to_string()]);
    assert!(!stale.status.success());
    assert_eq!(home.show()["progress"]["verified"], 0);
}

#[test]
fn feedback_drafts_remain_with_the_exact_agent_across_navigation_and_restart() {
    use zoetrope::console::{
        AgentKey, Console, ConsoleCommand, DiscoveryRoots, FleetNodeId, SessionKey,
    };
    use zoetrope::provider::Provider;
    use zoetrope::workstate::WorkUpdate;
    let home = Home::new();
    let native = home.0.join("codex/sessions/2026/10/23");
    std::fs::create_dir_all(&native).unwrap();
    for id in ["first", "second"] {
        std::fs::write(native.join(format!("rollout-{id}.jsonl")), format!("{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"{id}\",\"thread_source\":\"user\",\"cwd\":\"/projects/editor\"}}}}\n")).unwrap();
    }
    let roots = DiscoveryRoots {
        claude_projects: None,
        codex_home: Some(home.0.join("codex")),
    };
    let first = SessionKey {
        provider: Provider::Codex,
        session_id: "first".into(),
    };
    let second = SessionKey {
        provider: Provider::Codex,
        session_id: "second".into(),
    };
    let target = AgentKey {
        session: first.clone(),
        agent_id: "main".into(),
    };
    let mut console = Console::with_state(roots.clone(), home.0.join("state"));
    console.refresh(SystemTime::now());
    console.command(ConsoleCommand::Select(FleetNodeId::Session(first.clone())));
    console
        .update_work(
            &first,
            WorkUpdate::SaveDraft {
                target: target.clone(),
                text: "Keep the keyboard shortcuts simple".into(),
            },
        )
        .unwrap();
    console.command(ConsoleCommand::Select(FleetNodeId::Session(second.clone())));
    assert!(console.work(&second).unwrap().drafts.is_empty());
    drop(console);
    let mut restored = Console::with_state(roots, home.0.join("state"));
    restored.refresh(SystemTime::now());
    restored.command(ConsoleCommand::Select(FleetNodeId::Session(first.clone())));
    let output = restored.render().to_string();
    assert!(
        output.contains("Keep the keyboard shortcuts simple"),
        "{output}"
    );
    assert_eq!(restored.work(&first).unwrap().drafts[0].target, target);
}

#[test]
fn unanswered_questions_and_partial_answers_persist_when_native_requests_expire() {
    let home = Home::new();
    let target = json!({"session":{"provider":"codex","session_id":"goal"},"agent_id":"child"});
    home.apply(json!({"op":"capture_question","question":{
        "id":"daemon-a:request-17","target":target,"request_id":17,"turn_id":"turn-4",
        "prompts":[{"id":"style","header":"Style","question":"Which style should we use?","options":[{"label":"Compact","description":"Less detail"},{"label":"Detailed","description":"More detail"}]}],"state":"pending"
    }}));
    home.apply(json!({"op":"save_answer","question_id":"daemon-a:request-17","answers":{"style":["Compact, but keep the graph readable"]}}));
    let before = home.show();
    assert_eq!(before["record"]["questions"][0]["state"], "pending");
    assert_eq!(
        before["record"]["questions"][0]["answers"]["style"][0],
        "Compact, but keep the graph readable"
    );
    assert_eq!(before["record"]["outbox"].as_array().unwrap().len(), 0);
    home.apply(
        json!({"op":"resolve_question","question_id":"daemon-a:request-17","state":"expired"}),
    );
    let restored = home.show();
    assert_eq!(restored["record"]["questions"][0]["state"], "expired");
    assert_eq!(
        restored["record"]["questions"][0]["prompts"][0]["question"],
        "Which style should we use?"
    );
    assert_eq!(
        restored["record"]["questions"][0]["answers"]["style"][0],
        "Compact, but keep the graph readable"
    );
    assert_eq!(restored["record"]["outbox"].as_array().unwrap().len(), 0);
    home.apply(json!({"op":"answer_as_feedback","question_id":"daemon-a:request-17","feedback_id":"feedback-1","mode":"guidance"}));
    let explicit = home.show();
    assert_eq!(explicit["record"]["questions"][0]["state"], "expired");
    assert_eq!(
        explicit["record"]["outbox"][0]["target"]["agent_id"],
        "child"
    );
    assert_eq!(explicit["record"]["outbox"][0]["state"], "draft");
    assert!(
        explicit["record"]["outbox"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Compact, but keep the graph readable")
    );
}

#[test]
fn outbox_preserves_queueing_and_uncertain_delivery_without_claiming_work_completed() {
    let home = Home::new();
    let target = json!({"session":{"provider":"codex","session_id":"goal"},"agent_id":"child"});
    for (id, mode) in [("guidance-1", "guidance"), ("later-1", "queue")] {
        home.apply(json!({"op":"create_feedback","id":id,"target":target,"text":"Keep the graph readable","mode":mode,"expected_turn":"turn-4"}));
    }
    home.apply(json!({"op":"record_delivery","id":"later-1","state":"queued","message":"Queued by runtime; idle queues may start immediately","queue_id":"native-8"}));
    home.apply(json!({"op":"record_delivery","id":"guidance-1","state":"sending","message":"Sending to selected child"}));
    let restored = home.show();
    assert_eq!(restored["record"]["outbox"][0]["state"], "unknown");
    assert_eq!(restored["record"]["outbox"][0]["expected_turn"], "turn-4");
    assert_eq!(restored["record"]["outbox"][1]["state"], "queued");
    assert_eq!(restored["record"]["outbox"][1]["queue_id"], "native-8");
    assert_eq!(restored["progress"]["percent"], Value::Null);
    home.apply(json!({"op":"record_delivery","id":"guidance-1","state":"accepted","message":"Runtime accepted guidance"}));
    let accepted = home.show();
    assert_eq!(accepted["record"]["outbox"][0]["state"], "accepted");
    assert_eq!(accepted["progress"]["percent"], Value::Null);
    let wrong=home.run(&["apply","codex","goal",&json!({"op":"create_feedback","id":"wrong","target":{"session":{"provider":"codex","session_id":"other"},"agent_id":"main"},"text":"Misroute","mode":"guidance"}).to_string()]);
    assert!(!wrong.status.success());
    assert_eq!(home.show()["record"]["outbox"].as_array().unwrap().len(), 2);
}

#[test]
fn concurrent_processes_keep_each_agents_draft_without_lost_updates() {
    use std::process::Stdio;
    let home = Home::new();
    let mut children = Vec::new();
    for index in 0..16 {
        let text = format!("Feedback {index}: {}", "x".repeat(16384));
        children.push(Command::new(env!("CARGO_BIN_EXE_zoe")).arg("state").arg("--root").arg(&home.0)
            .args(["apply","codex","goal"])
            .arg(json!({"op":"save_draft","target":{"session":{"provider":"codex","session_id":"goal"},"agent_id":format!("child-{index}")},"text":text}).to_string())
            .stdout(Stdio::null()).stderr(Stdio::piped()).spawn().unwrap());
    }
    for child in children {
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let restored = home.show();
    let drafts = restored["record"]["drafts"].as_array().unwrap();
    assert_eq!(drafts.len(), 16);
    for index in 0..16 {
        assert!(
            drafts
                .iter()
                .any(|draft| draft["target"]["agent_id"] == format!("child-{index}"))
        );
    }
}

#[test]
fn searchable_work_includes_the_goal_and_exact_document_path_with_its_producer() {
    use zoetrope::console::{Console, ConsoleCommand, DiscoveryRoots, SessionKey};
    use zoetrope::provider::Provider;
    use zoetrope::workstate::WorkUpdate;
    let home = Home::new();
    let native = home.0.join("codex/archived_sessions");
    std::fs::create_dir_all(&native).unwrap();
    std::fs::write(native.join("rollout-goal.jsonl"),"{\"type\":\"session_meta\",\"payload\":{\"id\":\"goal\",\"cwd\":\"/projects/editor\",\"thread_source\":\"user\"}}\n").unwrap();
    home.apply(json!({"op":"set_goal","title":"Make steering comfortable","criteria":[{"id":"behavior","label":"Drafts remain"}],"reason":"User scope","provenance":"user"}));
    let document = home.0.join("Docs with spaces/Решения #1+%&.md");
    home.apply(json!({"op":"add_document","document":{"id":"patch-8","path":document,"producer":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"writer"},"source":"native file change","change":"changed"}}));
    let roots = DiscoveryRoots {
        claude_projects: None,
        codex_home: Some(home.0.join("codex")),
    };
    let mut console = Console::with_state(roots, home.0.clone());
    console.refresh(SystemTime::now());
    console.command(ConsoleCommand::ToggleHistory);
    console.command(ConsoleCommand::Search("COMFORTABLE".into()));
    assert!(
        console
            .render()
            .to_string()
            .contains("Make steering comfortable")
    );
    console.command(ConsoleCommand::Search("РЕШЕНИЯ #1+%&".into()));
    assert!(
        console
            .render()
            .to_string()
            .contains("Make steering comfortable")
    );
    let key = SessionKey {
        provider: Provider::Codex,
        session_id: "goal".into(),
    };
    let record = console.work(&key).unwrap();
    assert_eq!(record.documents[0].path, document);
    assert_eq!(record.documents[0].producer.agent_id, "writer");
    console.update_work(&key,serde_json::from_value::<WorkUpdate>(json!({"op":"add_document","document":{"id":"patch-8","path":document,"producer":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"writer"},"source":"native file change","change":"changed"}})).unwrap()).unwrap();
    assert_eq!(console.work(&key).unwrap().documents.len(), 1);
}

#[test]
fn recorded_plans_are_drafts_and_supported_changes_and_questions_keep_their_source() {
    use zoetrope::console::{Console, ConsoleCommand, DiscoveryRoots, SessionKey};
    use zoetrope::provider::Provider;
    let home = Home::new();
    let native = home.0.join("codex/sessions/2026/10/23");
    std::fs::create_dir_all(&native).unwrap();
    let records = [
        json!({"type":"session_meta","payload":{"id":"goal","session_id":"goal","cwd":"/projects/editor","thread_source":"user"}}),
        json!({"type":"response_item","payload":{"type":"function_call","name":"update_plan","call_id":"plan-1","arguments":json!({"plan":[{"step":"Make editing work","status":"completed"},{"step":"Run final checks","status":"completed"}]}).to_string()}}),
        json!({"type":"event_msg","payload":{"type":"item_completed","thread_id":"goal","item":{"type":"FileChange","id":"patch-1","status":"completed","changes":{"docs/Usage #1+%&.md":{"type":"add"},"src/editor.rs":{"type":"update"}}}}}),
        json!({"type":"response_item","payload":{"type":"function_call","name":"request_user_input","call_id":"question-1","arguments":json!({"questions":[{"id":"layout","header":"Layout","question":"Which layout?","options":[{"label":"Compact","description":"Fewer lines"}]}]}).to_string()}}),
        json!({"type":"event_msg","payload":{"type":"item_completed","thread_id":"goal","item":{"type":"FileChange","id":"failed-patch","status":"failed","changes":{"docs/Missing.md":{"type":"add"}}}}}),
    ];
    let transcript = records
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(native.join("rollout-goal.jsonl"), &transcript).unwrap();
    let roots = DiscoveryRoots {
        claude_projects: None,
        codex_home: Some(home.0.join("codex")),
    };
    let mut console = Console::with_state(roots.clone(), home.0.join("state"));
    console.refresh(SystemTime::now());
    console.command(ConsoleCommand::ToggleExpanded(SessionKey {
        provider: Provider::Codex,
        session_id: "goal".into(),
    }));
    let key = SessionKey {
        provider: Provider::Codex,
        session_id: "goal".into(),
    };
    let work = console.work(&key).unwrap();
    assert_eq!(work.progress().percent, None);
    assert_eq!(work.plan_drafts[0].steps[0].status, "completed");
    assert_eq!(work.documents.len(), 1);
    assert_eq!(
        work.documents[0].path,
        PathBuf::from("/projects/editor/docs/Usage #1+%&.md")
    );
    assert_eq!(work.documents[0].producer.agent_id, "main");
    assert_eq!(
        work.questions[0].state,
        zoetrope::workstate::QuestionState::Expired
    );
    assert_eq!(work.questions[0].prompts[0].question, "Which layout?");
    assert!(work.questions[0].answers.is_empty());
    drop(console);
    let mut restored = Console::with_state(roots, home.0.join("state"));
    restored.refresh(SystemTime::now());
    assert_eq!(restored.work(&key).unwrap().questions.len(), 1);
    assert_eq!(
        std::fs::read_to_string(native.join("rollout-goal.jsonl")).unwrap(),
        transcript
    );
}

#[test]
fn reopening_a_verified_outcome_removes_credit_and_keeps_its_previous_evidence() {
    let home = Home::new();
    home.apply(json!({"op":"set_goal","title":"Ship fix","criteria":[{"id":"behavior","label":"Edit works"}],"reason":"Initial","provenance":"user"}));
    home.apply(json!({"op":"verify","id":"behavior","evidence":[{"kind":"reference","detail":"Earlier check passed","provenance":"test"}]}));
    let reopened=home.apply(json!({"op":"set_criterion_status","id":"behavior","status":"in_progress","revision":1,"reason":"Found another failing case"}));
    assert_eq!(reopened["progress"]["percent"], 0.0);
    assert_eq!(
        reopened["record"]["goal"]["criteria"][0]["evidence"][0]["detail"],
        "Earlier check passed"
    );
    assert_eq!(
        reopened["record"]["criterion_history"][0]["reason"],
        "Found another failing case"
    );
    let bypass=home.run(&["apply","codex","goal",&json!({"op":"set_criterion_status","id":"behavior","status":"verified","reason":"Pretend it passed"}).to_string()]);
    assert!(!bypass.status.success());
    assert_eq!(home.show()["progress"]["verified"], 0);
}

#[test]
fn proposing_tracking_is_explicit_and_adopted_native_plan_steps_start_unverified() {
    let home = Home::new();
    let target = json!({"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"});
    assert_eq!(home.show()["record"]["checkpoints"], false);
    home.apply(json!({"op":"set_checkpoints","enabled":true}));
    home.apply(json!({"op":"request_criteria","target":target}));
    let proposal = home.show();
    assert_eq!(proposal["record"]["checkpoints"], true);
    assert!(
        proposal["record"]["drafts"][0]["text"]
            .as_str()
            .unwrap()
            .contains("final checks")
    );
    assert_eq!(proposal["record"]["outbox"].as_array().unwrap().len(), 0);
    assert_eq!(proposal["progress"]["percent"], Value::Null);
    home.apply(json!({"op":"import_plan","proposal":{"id":"plan-1","producer":target,"steps":[{"step":"Behavior works","status":"completed"},{"step":"Final checks pass","status":"completed"}],"source":"native plan"}}));
    let adopted=home.apply(json!({"op":"adopt_plan","proposal_id":"plan-1","title":"Ship fix","reason":"I accept these criteria","provenance":"user"}));
    assert_eq!(adopted["progress"]["label"], "0 of 2 verified · 0%");
    assert_eq!(
        adopted["record"]["goal"]["criteria"][0]["status"],
        "pending"
    );
    assert_eq!(
        adopted["record"]["goal"]["criteria"][0]["evidence"],
        json!([])
    );
}

#[test]
fn invalid_cli_arguments_do_not_modify_an_existing_goal_record() {
    let home = Home::new();
    let update=json!({"op":"set_goal","title":"Must not be saved","criteria":[{"id":"bad","label":"Bad invocation"}],"reason":"Bad syntax","provenance":"user"}).to_string();
    let result = home.run(&["apply", "codex", "goal", &update, "unexpected"]);
    assert!(!result.status.success());
    assert_eq!(home.show()["record"]["goal"], Value::Null);
}

#[test]
fn reconnect_requires_fresh_pending_evidence_and_resolution_does_not_erase_the_answer() {
    let home = Home::new();
    let question = json!({"id":"daemon-a:request-9","target":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},"request_id":9,"turn_id":"turn-4","prompts":[{"id":"layout","question":"Which layout?"}],"state":"pending"});
    home.apply(json!({"op":"capture_question","question":question}));
    home.apply(json!({"op":"save_answer","question_id":"daemon-a:request-9","answers":{"layout":["Compact"]}}));
    home.apply(json!({"op":"expire_pending_questions"}));
    assert_eq!(home.show()["record"]["questions"][0]["state"], "expired");
    home.apply(json!({"op":"capture_question","question":question}));
    assert_eq!(home.show()["record"]["questions"][0]["state"], "pending");
    home.apply(
        json!({"op":"resolve_question","question_id":"daemon-a:request-9","state":"resolved"}),
    );
    home.apply(json!({"op":"capture_question","question":question}));
    let restored = home.show();
    assert_eq!(restored["record"]["questions"][0]["state"], "resolved");
    assert_eq!(
        restored["record"]["questions"][0]["answers"]["layout"][0],
        "Compact"
    );
    assert_eq!(restored["record"]["outbox"].as_array().unwrap().len(), 0);
}

#[test]
fn applying_updates_reports_compact_results_without_echoing_large_drafts_to_the_agent() {
    let home = Home::new();
    home.apply(json!({"op":"save_draft","target":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},"text":"Saved private draft ".repeat(2048)}));
    let result = home.run(&[
        "apply",
        "codex",
        "goal",
        &json!({"op":"set_checkpoints","enabled":true}).to_string(),
    ]);
    assert!(result.status.success());
    assert!(
        result.stdout.len() < 1024,
        "small state edits should not echo full scope history and drafts"
    );
    let response: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(response["saved"], true);
    assert!(!String::from_utf8_lossy(&result.stdout).contains("Saved private draft"));
    assert_eq!(home.show()["record"]["checkpoints"], true);
}

#[test]
fn work_reports_survive_restart_without_verifying_goal_outcomes() {
    let home = Home::new();
    home.apply(json!({"op":"set_goal","title":"Make steering clear","criteria":[{"id":"target","label":"Input reaches its target"}],"reason":"Agreed outcome","provenance":"operator"}));
    home.apply(
        json!({"op":"upsert_work_item","expected_revision":0,"item":{
            "id":"inspect","label":"Inspect current navigation",
            "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
            "status":"done","source":"coordinator report"
        }}),
    );
    let restarted = home.show();
    let item = &restarted["record"]["work_items"][0];
    assert_eq!(item["id"], "inspect");
    assert_eq!(item["label"], "Inspect current navigation");
    assert_eq!(item["owner"]["agent_id"], "main");
    assert_eq!(item["status"], "done");
    assert_eq!(item["revision"], 1);
    assert!(chrono::DateTime::parse_from_rfc3339(item["updated_at"].as_str().unwrap()).is_ok());
    assert_eq!(restarted["progress"]["verified"], 0);
    assert_eq!(restarted["progress"]["label"], "0 of 1 verified · 0%");
}

#[test]
fn concurrent_work_revisions_reject_stale_reports_and_accept_identical_retries() {
    let home = Home::new();
    let item = json!({"id":"steering","label":"Expose steering",
        "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
        "status":"active","source":"coordinator"});
    let create = json!({"op":"upsert_work_item","expected_revision":0,"item":item});
    let initial = home.apply(create.clone());
    let retried = home.apply(create);
    assert_eq!(
        retried["record"]["work_items"],
        initial["record"]["work_items"]
    );
    let mut finished = item.clone();
    finished["status"] = json!("done");
    let accepted =
        home.apply(json!({"op":"upsert_work_item","expected_revision":1,"item":finished}));
    assert_eq!(accepted["record"]["work_items"][0]["revision"], 2);
    let stale = home.run(&[
        "apply",
        "codex",
        "goal",
        &json!({"op":"upsert_work_item","expected_revision":1,"item":item}).to_string(),
    ]);
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("revision"));
    assert_eq!(
        home.show()["record"]["work_items"],
        accepted["record"]["work_items"]
    );
}

#[test]
fn work_context_preserves_declared_dependencies_and_revision_qualified_outcomes() {
    let home = Home::new();
    let item = json!({"id":"steering","label":"Expose steering",
        "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"worker"},
        "status":"blocked","source":"worker report",
        "parent_id":"design","depends_on":["runtime-api"],
        "outcome_links":[{"criterion_id":"correct-target","goal_revision":3}],
        "purpose":"Explain input before sending","change":"Child input is restricted",
        "next":"Show coordinator selection","blocker":"Waiting for runtime capability"});
    let saved = home.apply(json!({"op":"upsert_work_item","expected_revision":0,"item":item}));
    let work = &saved["record"]["work_items"][0];
    assert_eq!(work["parent_id"], "design");
    assert_eq!(work["depends_on"], json!(["runtime-api"]));
    assert_eq!(
        work["outcome_links"],
        json!([{"criterion_id":"correct-target","goal_revision":3}])
    );
    assert_eq!(work["purpose"], "Explain input before sending");
    assert_eq!(work["change"], "Child input is restricted");
    assert_eq!(work["next"], "Show coordinator selection");
    assert_eq!(work["blocker"], "Waiting for runtime capability");
    let mut independent = item.clone();
    independent["id"] = json!("independent");
    independent["depends_on"] = json!([]);
    home.apply(json!({"op":"upsert_work_item","expected_revision":0,"item":independent}));
    let mut unknown = item;
    unknown["id"] = json!("unknown");
    unknown.as_object_mut().unwrap().remove("depends_on");
    let state = home.apply(json!({"op":"upsert_work_item","expected_revision":0,"item":unknown}));
    assert_eq!(state["record"]["work_items"][1]["depends_on"], json!([]));
    assert_eq!(state["record"]["work_items"][2]["depends_on"], Value::Null);
    assert_eq!(state["progress"]["percent"], Value::Null);
}

#[test]
fn transition_reports_update_only_supplied_fields_and_can_clear_context_explicitly() {
    let home = Home::new();
    home.apply(json!({"op":"upsert_work_item","expected_revision":0,"item":{
        "id":"steering","label":"Expose steering",
        "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"worker"},
        "status":"blocked","source":"worker report","depends_on":["runtime-api"],
        "purpose":"Explain available input","blocker":"Waiting for runtime", "next":"Inspect capability"
    }}));
    let updated = home.apply(
        json!({"op":"upsert_work_item","expected_revision":1,"item":{
            "id":"steering","status":"active","blocker":null,"next":"Render available actions"
        }}),
    );
    let work = &updated["record"]["work_items"][0];
    assert_eq!(work["revision"], 2);
    assert_eq!(work["owner"]["agent_id"], "worker");
    assert_eq!(work["label"], "Expose steering");
    assert_eq!(work["source"], "worker report");
    assert_eq!(work["purpose"], "Explain available input");
    assert_eq!(work["depends_on"], json!(["runtime-api"]));
    assert_eq!(work["blocker"], Value::Null);
    assert_eq!(work["next"], "Render available actions");
    let cleared = home.apply(
        json!({"op":"upsert_work_item","expected_revision":2,"item":{
            "id":"steering","depends_on":null,"purpose":null
        }}),
    );
    assert_eq!(cleared["record"]["work_items"][0]["revision"], 3);
    assert_eq!(
        cleared["record"]["work_items"][0]["depends_on"],
        Value::Null
    );
    assert_eq!(cleared["record"]["work_items"][0]["purpose"], Value::Null);
    let invalid = home.run(&[
        "apply",
        "codex",
        "goal",
        &json!({"op":"upsert_work_item","expected_revision":0,"item":{"id":"missing"}}).to_string(),
    ]);
    assert!(!invalid.status.success());
    assert_eq!(
        home.show()["record"]["work_items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn invalid_work_reports_cannot_change_an_existing_record() {
    let home = Home::new();
    let valid = json!({"id":"steering","label":"Expose steering",
        "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
        "status":"active","source":"coordinator"});
    let initial = home.apply(json!({"op":"upsert_work_item","expected_revision":0,"item":valid}));
    let invalid_patches = [
        json!({"id":"steering","owner":{"session":{"provider":"codex","session_id":"another-goal"},"agent_id":"main"}}),
        json!({"id":"steering","label":" ","status":"done"}),
        json!({"id":"steering","source":""}),
        json!({"id":"steering","purpose":"x".repeat(1025)}),
        json!({"id":"steering","depends_on":["steering"]}),
        json!({"id":"steering","depends_on":["same","same"]}),
        json!({"id":"steering","parent_id":"steering"}),
        json!({"id":"steering","outcome_links":[{"criterion_id":"target","goal_revision":0}]}),
        json!({"id":"steering","outcome_links":[{"criterion_id":"","goal_revision":1}]}),
        json!({"id":"steering","unexpected_field":"silently ignored?"}),
    ];
    for patch in invalid_patches {
        let result = home.run(&[
            "apply",
            "codex",
            "goal",
            &json!({"op":"upsert_work_item","expected_revision":1,"item":patch}).to_string(),
        ]);
        assert!(!result.status.success(), "accepted invalid report: {patch}");
        assert_eq!(
            home.show()["record"]["work_items"],
            initial["record"]["work_items"]
        );
    }
}

#[test]
fn compact_context_reads_current_work_and_plans_without_echoing_private_history() {
    let home = Home::new();
    home.apply(json!({"op":"set_goal","title":"Make steering clear","criteria":[{"id":"target","label":"Input reaches its target"}],"reason":"Agreed","provenance":"operator"}));
    let owner = json!({"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"});
    home.apply(json!({"op":"save_draft","target":owner,"text":"PRIVATE DRAFT ".repeat(2048)}));
    home.apply(json!({"op":"verify","id":"target","evidence":[{"kind":"reference","detail":"VERBOSE EVIDENCE ".repeat(2048),"provenance":"operator"}]}));
    home.apply(json!({"op":"import_plan","proposal":{"id":"native-plan","producer":owner,"steps":[{"step":"Inspect runtime","status":"in_progress"}],"source":"recorded native plan"}}));
    home.apply(
        json!({"op":"upsert_work_item","expected_revision":0,"item":{
            "id":"steering","label":"Expose steering","owner":owner,
            "status":"active","source":"coordinator","next":"Show input capability"
        }}),
    );
    let result = home.run(&["context", "codex", "goal"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stdout.len() < 8192);
    let text = String::from_utf8_lossy(&result.stdout);
    assert!(!text.contains("PRIVATE DRAFT"));
    assert!(!text.contains("VERBOSE EVIDENCE"));
    assert!(!text.contains("drafts"));
    assert!(!text.contains("outbox"));
    let context: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(context["goal"]["title"], "Make steering clear");
    assert_eq!(context["goal"]["revision"], 1);
    assert_eq!(context["goal"]["criteria"][0]["id"], "target");
    assert_eq!(context["goal"]["criteria"][0]["evidence_count"], 1);
    assert_eq!(context["progress"]["verified"], 1);
    assert_eq!(context["work_items"][0]["id"], "steering");
    assert_eq!(context["work_items"][0]["next"], "Show input capability");
    assert_eq!(context["plans"][0]["steps"][0]["status"], "in_progress");
}

#[test]
fn context_pages_work_and_bounds_large_goal_and_plan_text() {
    let home = Home::new();
    home.apply(json!({"op":"set_goal","title":"Large report","criteria":[{"id":"huge","label":"Goal detail ".repeat(10000)}],"reason":"Bounded context","provenance":"operator"}));
    let owner = json!({"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"});
    home.apply(json!({"op":"import_plan","proposal":{"id":"huge-plan","producer":owner,"steps":[{"step":"Plan detail ".repeat(10000),"status":"in_progress"}],"source":"native plan"}}));
    for index in 0..24 {
        home.apply(json!({"op":"upsert_work_item","expected_revision":0,"item":{
            "id":format!("unit-{index:02}"),"label":format!("Work {index}"),"owner":owner,"status":"planned","source":"coordinator",
            "purpose":"p".repeat(1024),"change":"c".repeat(1024),"next":"n".repeat(1024),"blocker":"b".repeat(1024)
        }}));
    }
    let first = home.run(&["context", "codex", "goal"]);
    assert!(first.status.success());
    assert!(
        first.stdout.len() <= 65536,
        "context emitted {} bytes",
        first.stdout.len()
    );
    let context: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(context["work_items_total"], 24);
    assert!(context["work_items_omitted"].as_u64().unwrap() > 0);
    assert_eq!(context["goal"]["criteria"][0]["label_truncated"], true);
    assert_eq!(context["plans"][0]["steps"][0]["text_truncated"], true);
    assert!(context["next_work_offset"].as_u64().unwrap() > 0);
    let page = home.run(&["context", "codex", "goal", "--offset", "3", "--limit", "2"]);
    assert!(
        page.status.success(),
        "{}",
        String::from_utf8_lossy(&page.stderr)
    );
    let page: Value = serde_json::from_slice(&page.stdout).unwrap();
    assert_eq!(page["work_items"].as_array().unwrap().len(), 2);
    assert_eq!(page["work_items"][0]["id"], "unit-03");
    assert_eq!(page["work_items"][1]["id"], "unit-04");
    assert_eq!(page["next_work_offset"], 5);
    assert_eq!(page["work_items_omitted"], 22);
}

#[test]
fn work_receipts_return_the_saved_revision_without_echoing_report_context() {
    let home = Home::new();
    let create = json!({"op":"upsert_work_item","expected_revision":0,"item":{
        "id":"steering","label":"Expose steering",
        "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
        "status":"active","source":"coordinator","purpose":"VERBOSE CONTEXT ".repeat(50)
    }});
    let result = home.run(&["apply", "codex", "goal", &create.to_string()]);
    assert!(result.status.success());
    assert!(result.stdout.len() < 1024);
    assert!(!String::from_utf8_lossy(&result.stdout).contains("VERBOSE CONTEXT"));
    let receipt: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(receipt["saved"], true);
    assert_eq!(receipt["work_item"]["id"], "steering");
    assert_eq!(receipt["work_item"]["revision"], 1);
    let update = home.run(&["apply","codex","goal",&json!({"op":"upsert_work_item","expected_revision":1,"item":{"id":"steering","status":"done"}}).to_string()]);
    assert!(update.status.success());
    let receipt: Value = serde_json::from_slice(&update.stdout).unwrap();
    assert_eq!(receipt["work_item"]["revision"], 2);
}

#[test]
fn an_identical_report_with_an_unrelated_revision_is_not_acknowledged_as_a_retry() {
    let home = Home::new();
    let item = json!({"id":"steering","label":"Expose steering",
        "owner":{"session":{"provider":"codex","session_id":"goal"},"agent_id":"main"},
        "status":"active","source":"coordinator"});
    let initial = home.apply(json!({"op":"upsert_work_item","expected_revision":0,"item":item}));
    let invalid = home.run(&[
        "apply",
        "codex",
        "goal",
        &json!({"op":"upsert_work_item","expected_revision":99,"item":item}).to_string(),
    ]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("revision"));
    assert_eq!(
        home.show()["record"]["work_items"],
        initial["record"]["work_items"]
    );
}
