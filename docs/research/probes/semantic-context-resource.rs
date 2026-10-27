//! Manual public-runtime resource probe; compile after a release build.
//! See docs/SEMANTIC-CONTEXT-VALIDATION.md for the exact command and observations.
use std::path::PathBuf;
use zoetrope::console::DiscoveryRoots;
use zoetrope::fleet::{FleetConfig, runtime};
fn main() -> anyhow::Result<()> {
    let root = PathBuf::from(std::env::args_os().nth(1).expect("fixture root"));
    let config = FleetConfig {
        roots: DiscoveryRoots {
            claude_projects: None,
            codex_home: Some(root.join("codex")),
        },
        state_root: root.join("state"),
        socket: root.join("unavailable-control.sock"),
        ..FleetConfig::default()
    };
    tokio::runtime::Runtime::new()?.block_on(runtime::run(config))
}
