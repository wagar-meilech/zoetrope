#![cfg(all(feature = "native", unix))]

use std::io::{BufRead, BufReader, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture {
    root: PathBuf,
    checkout: PathBuf,
    home: PathBuf,
    tools: PathBuf,
    elsewhere: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zoe-install-{}-{nonce}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let checkout = root.join("source checkout's fork");
        let home = root.join("user home");
        let tools = root.join("build tools");
        let elsewhere = root.join("another project");
        for path in [&checkout.join("scripts"), &home, &tools, &elsewhere] {
            std::fs::create_dir_all(path).unwrap();
        }
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/install.sh");
        if source.exists() {
            std::fs::copy(source, checkout.join("scripts/install.sh")).unwrap();
        }
        std::fs::write(
            checkout.join("Cargo.toml"),
            "[package]\nname = 'zoetrope'\n",
        )
        .unwrap();
        std::fs::write(checkout.join("Cargo.lock"), "version = 4\n").unwrap();
        let skill = checkout.join("skills/zoetrope-tracking/references");
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(skill.parent().unwrap().join("SKILL.md"), "packaged-v1\n").unwrap();
        std::fs::write(skill.join("sources.md"), "source-contract-v1\n").unwrap();
        executable(
            &checkout.join("candidate"),
            "#!/bin/sh\nif [ \"${1:-}\" = --hold ]; then printf '%s\\n' ready; read ignored; fi\nprintf '%s\\n' installed-v1\n",
        );
        executable(
            &tools.join("cargo"),
            r#"#!/bin/sh
set -eu
release=no
locked=no
target=
binary=
while [ "$#" -gt 0 ]; do
    case "$1" in
        build) ;;
        --release) release=yes ;;
        --locked) locked=yes ;;
        --bin) shift; binary=$1 ;;
        --target-dir) shift; target=$1 ;;
        *) printf 'Unexpected build argument: %s\n' "$1" >&2; exit 28 ;;
    esac
    shift
done
[ "$release" = yes ] && [ "$locked" = yes ] && [ "$binary" = zoe ] && [ -n "$target" ]
[ -f Cargo.toml ] && [ -f Cargo.lock ]
[ ! -f build-fails ] || exit 27
mkdir -p "$target/release"
cp candidate "$target/release/zoe"
chmod 755 "$target/release/zoe"
"#,
        );
        Self {
            root,
            checkout,
            home,
            tools,
            elsewhere,
        }
    }
    fn command(&self, executable: &Path) -> Command {
        let mut command = Command::new(executable);
        command
            .current_dir(&self.elsewhere)
            .env("HOME", &self.home)
            .env("CODEX_HOME", self.root.join("codex home"))
            .env(
                "PATH",
                std::env::join_paths(std::iter::once(self.tools.clone()).chain(
                    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
                ))
                .unwrap(),
            )
            .env("CARGO_TARGET_DIR", self.root.join("build output"));
        command
    }
    fn install(&self, destination: Option<&Path>) -> Output {
        let mut command = self.command(Path::new("/bin/sh"));
        command.arg(self.checkout.join("scripts/install.sh"));
        if let Some(destination) = destination {
            command.arg("--bin-dir").arg(destination);
        }
        command.output().unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn executable(path: &Path, text: &str) {
    std::fs::write(path, text).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}
fn succeeds(output: &Output) {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn installing_to_the_user_bin_runs_the_release_without_cargo_at_launch() {
    let fixture = Fixture::new();
    succeeds(&fixture.install(None));
    let output = Command::new(fixture.home.join(".local/bin/zoe"))
        .env("PATH", "")
        .output()
        .unwrap();
    succeeds(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "installed-v1\n");
}

#[test]
fn updating_twice_from_another_directory_preserves_the_running_command() {
    let fixture = Fixture::new();
    let destination = fixture.root.join("installed commands' bin");
    succeeds(&fixture.install(Some(&destination)));
    let updater = destination.join("zoe-update");
    assert!(updater.is_file(), "installation must include zoe-update");
    let binary = destination.join("zoe");
    let mut original_file = std::fs::File::open(&binary).unwrap();
    let mut running = fixture
        .command(&binary)
        .arg("--hold")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut output = BufReader::new(running.stdout.take().unwrap());
    let mut ready = String::new();
    output.read_line(&mut ready).unwrap();
    assert_eq!(ready, "ready\n");
    executable(
        &fixture.checkout.join("candidate"),
        "#!/bin/sh\nprintf '%s\\n' installed-v2\n",
    );
    for _ in 0..2 {
        succeeds(&fixture.command(&updater).output().unwrap());
        let updated = fixture.command(&binary).output().unwrap();
        succeeds(&updated);
        assert_eq!(String::from_utf8(updated.stdout).unwrap(), "installed-v2\n");
    }
    let mut original_contents = String::new();
    original_file
        .read_to_string(&mut original_contents)
        .unwrap();
    assert!(original_contents.contains("installed-v1"));
    assert!(!original_contents.contains("installed-v2"));
    drop(running.stdin.take());
    let mut previous = String::new();
    output.read_to_string(&mut previous).unwrap();
    assert!(running.wait().unwrap().success());
    assert_eq!(previous, "installed-v1\n");
}

#[test]
fn a_moved_checkout_explains_reinstallation_without_breaking_the_installed_command() {
    let fixture = Fixture::new();
    let destination = fixture.root.join("installed bin");
    succeeds(&fixture.install(Some(&destination)));
    let moved = fixture.root.join("moved checkout");
    std::fs::rename(&fixture.checkout, &moved).unwrap();
    let updater = destination.join("zoe-update");
    let missing = fixture.command(&updater).output().unwrap();
    assert!(!missing.status.success());
    let message = String::from_utf8_lossy(&missing.stderr);
    assert!(
        message.contains("checkout") && message.contains("reinstall"),
        "{message}"
    );
    let installed = fixture.command(&destination.join("zoe")).output().unwrap();
    succeeds(&installed);
    assert_eq!(
        String::from_utf8(installed.stdout).unwrap(),
        "installed-v1\n"
    );
    succeeds(
        &fixture
            .command(Path::new("/bin/sh"))
            .arg(moved.join("scripts/install.sh"))
            .arg("--bin-dir")
            .arg(&destination)
            .output()
            .unwrap(),
    );
    succeeds(&fixture.command(&updater).output().unwrap());
}

#[test]
fn a_failed_update_leaves_both_installed_commands_usable_and_unchanged() {
    let fixture = Fixture::new();
    let destination = fixture.root.join("installed bin");
    succeeds(&fixture.install(Some(&destination)));
    let binary = destination.join("zoe");
    let updater = destination.join("zoe-update");
    let binary_before = std::fs::read(&binary).unwrap();
    let updater_before = std::fs::read(&updater).unwrap();
    executable(
        &fixture.checkout.join("candidate"),
        "#!/bin/sh\nprintf '%s\\n' installed-v2\n",
    );
    std::fs::write(fixture.checkout.join("build-fails"), "build failure").unwrap();
    let failed = fixture.command(&updater).output().unwrap();
    assert_eq!(failed.status.code(), Some(27));
    assert_eq!(std::fs::read(&binary).unwrap(), binary_before);
    assert_eq!(std::fs::read(&updater).unwrap(), updater_before);
    let installed = fixture.command(&binary).output().unwrap();
    succeeds(&installed);
    assert_eq!(
        String::from_utf8(installed.stdout).unwrap(),
        "installed-v1\n"
    );
    std::fs::remove_file(fixture.checkout.join("build-fails")).unwrap();
    succeeds(&fixture.command(&updater).output().unwrap());
    let installed = fixture.command(&binary).output().unwrap();
    succeeds(&installed);
    assert_eq!(
        String::from_utf8(installed.stdout).unwrap(),
        "installed-v2\n"
    );
}

#[test]
fn an_existing_command_directory_is_rejected_before_installed_files_change() {
    let fixture = Fixture::new();
    let destination = fixture.root.join("invalid bin");
    std::fs::create_dir_all(destination.join("zoe")).unwrap();
    std::fs::write(destination.join("zoe-update"), "existing updater").unwrap();
    let failed = fixture.install(Some(&destination));
    assert!(!failed.status.success());
    assert_eq!(
        std::fs::read_to_string(destination.join("zoe-update")).unwrap(),
        "existing updater"
    );
    assert!(!destination.join("zoe/zoe").exists());
}

#[test]
fn installation_and_updates_ship_the_skill_without_replacing_unrelated_content() {
    let fixture = Fixture::new();
    let skill = fixture.root.join("codex home/skills/zoetrope-tracking");
    let unrelated = fixture
        .root
        .join("codex home/skills/another-skill/SKILL.md");
    std::fs::create_dir_all(unrelated.parent().unwrap()).unwrap();
    std::fs::write(&unrelated, "user-owned skill").unwrap();
    let destination = fixture.root.join("installed bin");
    succeeds(&fixture.install(Some(&destination)));
    assert_eq!(
        std::fs::read_to_string(skill.join("SKILL.md")).unwrap(),
        "packaged-v1\n"
    );
    assert_eq!(
        std::fs::read_to_string(skill.join("references/sources.md")).unwrap(),
        "source-contract-v1\n"
    );
    std::fs::write(skill.join("personal-note"), "user-owned note").unwrap();
    std::fs::write(
        fixture.checkout.join("skills/zoetrope-tracking/SKILL.md"),
        "packaged-v2\n",
    )
    .unwrap();
    let updater = destination.join("zoe-update");
    succeeds(&fixture.command(&updater).output().unwrap());
    assert_eq!(
        std::fs::read_to_string(skill.join("SKILL.md")).unwrap(),
        "packaged-v2\n"
    );
    assert_eq!(
        std::fs::read_to_string(skill.join("personal-note")).unwrap(),
        "user-owned note"
    );
    assert_eq!(
        std::fs::read_to_string(&unrelated).unwrap(),
        "user-owned skill"
    );
    std::fs::write(
        fixture.checkout.join("skills/zoetrope-tracking/SKILL.md"),
        "unbuilt-v3\n",
    )
    .unwrap();
    std::fs::write(fixture.checkout.join("build-fails"), "build failure").unwrap();
    assert!(!fixture.command(&updater).output().unwrap().status.success());
    assert_eq!(
        std::fs::read_to_string(skill.join("SKILL.md")).unwrap(),
        "packaged-v2\n"
    );
}

#[test]
fn updating_a_packaged_file_replaces_a_user_symlink_without_overwriting_its_target() {
    let fixture = Fixture::new();
    let destination = fixture.root.join("installed bin");
    succeeds(&fixture.install(Some(&destination)));
    let installed = fixture
        .root
        .join("codex home/skills/zoetrope-tracking/SKILL.md");
    let personal = fixture.root.join("personal skill text");
    std::fs::write(&personal, "user-owned content").unwrap();
    std::fs::remove_file(&installed).unwrap();
    std::os::unix::fs::symlink(&personal, &installed).unwrap();
    succeeds(
        &fixture
            .command(&destination.join("zoe-update"))
            .output()
            .unwrap(),
    );
    assert_eq!(
        std::fs::read_to_string(&personal).unwrap(),
        "user-owned content"
    );
    assert_eq!(
        std::fs::read_to_string(&installed).unwrap(),
        "packaged-v1\n"
    );
    assert!(
        !installed
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[test]
fn repeated_updates_preserve_nested_user_extras_without_following_old_directory_links() {
    let fixture = Fixture::new();
    let destination = fixture.root.join("installed bin");
    succeeds(&fixture.install(Some(&destination)));
    let skill = fixture.root.join("codex home/skills/zoetrope-tracking");
    let packaged = fixture.checkout.join("skills/zoetrope-tracking");
    std::fs::create_dir_all(skill.join("references/personal/deep")).unwrap();
    std::fs::write(
        skill.join("references/personal/deep/.note"),
        "nested user note",
    )
    .unwrap();
    std::fs::write(skill.join(".local-note"), "hidden user note").unwrap();
    std::fs::write(
        skill.join("references/sources.md"),
        "old local package copy",
    )
    .unwrap();
    let external = fixture.root.join("private linked directory");
    std::fs::create_dir_all(&external).unwrap();
    std::fs::write(external.join("private.txt"), "leave private content alone").unwrap();
    std::os::unix::fs::symlink(&external, skill.join("references/managed")).unwrap();
    std::os::unix::fs::symlink(&external, skill.join("personal-directory-link")).unwrap();
    std::fs::create_dir_all(packaged.join("references/managed")).unwrap();
    std::fs::write(
        packaged.join("references/managed/guide.md"),
        "new package guide",
    )
    .unwrap();
    std::fs::create_dir_all(skill.join("references/replaced-directory")).unwrap();
    std::fs::write(
        skill.join("references/replaced-directory/old-note"),
        "conflicting extra",
    )
    .unwrap();
    std::fs::write(
        packaged.join("references/replaced-directory"),
        "new packaged file",
    )
    .unwrap();

    let updater = destination.join("zoe-update");
    for _ in 0..2 {
        let output = fixture.command(&updater).output().unwrap();
        succeeds(&output);
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            std::fs::read_to_string(skill.join("references/personal/deep/.note")).unwrap(),
            "nested user note"
        );
        assert_eq!(
            std::fs::read_to_string(skill.join(".local-note")).unwrap(),
            "hidden user note"
        );
        assert_eq!(
            std::fs::read_to_string(skill.join("references/sources.md")).unwrap(),
            "source-contract-v1\n"
        );
        assert_eq!(
            std::fs::read_to_string(skill.join("references/managed/guide.md")).unwrap(),
            "new package guide"
        );
        assert!(!skill.join("references/managed/private.txt").exists());
        assert_eq!(
            std::fs::read_to_string(skill.join("references/replaced-directory")).unwrap(),
            "new packaged file"
        );
        assert_eq!(
            std::fs::read_link(skill.join("personal-directory-link")).unwrap(),
            external
        );
        assert_eq!(
            std::fs::read_to_string(external.join("private.txt")).unwrap(),
            "leave private content alone"
        );
        assert!(!external.join("guide.md").exists());
    }
}
