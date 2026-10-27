//! Bounded, explicitly bound local task sources. No runtime messages or remote IO.
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::console::{AgentKey, SessionKey};
use crate::provider::Provider;
use crate::workstate::{Store, WorkOutcomeLink, WorkRecord, WorkStatus};

const MAX_CACHE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_SOURCE_BYTES: u64 = 256 * 1024;

/// Current local-source observation, separate from supplemental reports/evidence.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SourceTask {
    pub id: String,
    pub source_id: String,
    pub task_id: String,
    pub label: String,
    pub status: Option<WorkStatus>,
    pub status_label: String,
    pub owner: Option<AgentKey>,
    pub owner_label: String,
    pub depends_on: Option<Vec<String>>,
    pub parent_id: Option<String>,
    pub purpose: Option<String>,
    pub change: Option<String>,
    pub next: Option<String>,
    pub blocker: Option<String>,
    pub outcome_links: Vec<WorkOutcomeLink>,
    pub source: String,
    pub updated_at: String,
    pub fingerprint: String,
    pub provenance: BTreeMap<String, String>,
    pub stale: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Source {
    selector: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    fingerprint: String,
    #[serde(default)]
    updated_at: String,
    #[serde(default)]
    tasks: Vec<SourceTask>,
    #[serde(default)]
    retired: Vec<SourceTask>,
    #[serde(default)]
    authority: BTreeSet<String>,
    #[serde(default)]
    error: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Binding {
    project: PathBuf,
    session: SessionKey,
    enabled: bool,
    sources: Vec<Source>,
}
#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
struct Registry {
    projects: Vec<PathBuf>,
    bindings: Vec<Binding>,
}

fn read(root: &Path) -> Result<Registry> {
    let file = match fs::File::open(root.join("tracking.json")) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Registry::default());
        }
        Err(error) => return Err(error.into()),
    };
    let mut bytes = Vec::new();
    file.take(MAX_CACHE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_CACHE_BYTES {
        bail!("tracking cache exceeds 8 MiB");
    }
    let registry: Registry =
        serde_json::from_slice(&bytes).context("invalid tracking cache; left unchanged")?;
    if registry.bindings.len() > 128 || registry.projects.len() > 64 {
        bail!("tracking registry exceeds bounds");
    }
    Ok(registry)
}
fn write(root: &Path, registry: &Registry) -> Result<()> {
    let bytes = serde_json::to_vec(registry)?;
    if bytes.len() as u64 > MAX_CACHE_BYTES {
        bail!("tracking cache exceeds 8 MiB; left unchanged");
    }
    let path = root.join(format!("tracking-{}.tmp", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| -> Result<()> {
        let mut file = options.open(&path)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&path, root.join("tracking.json"))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result
}
fn lock(root: &Path) -> Result<fs::File> {
    fs::create_dir_all(root)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(root.join("tracking.lock"))?;
    fs2::FileExt::lock_exclusive(&file)?;
    Ok(file)
}

const USAGE: &str = "zoe tracking enable|status|disable|sync --project DIR --provider codex|claude --session EXACT_ID [--source FILE#HEADING | --source FILE@ticket] [--root STATE_DIR]";

/// Public command: exact project/session scope, without modifying project files.
pub fn run_cli(args: impl Iterator<Item = String>) -> Result<()> {
    let mut args = args.peekable();
    let Some(operation) = args.next() else {
        println!("{USAGE}");
        return Ok(());
    };
    if matches!(operation.as_str(), "--help" | "-h" | "help") {
        println!("{USAGE}");
        return Ok(());
    }
    if !matches!(operation.as_str(), "enable" | "status" | "disable" | "sync") {
        bail!("{USAGE}");
    }
    let (mut project, mut provider, mut session, mut root) = (None, None, None, None);
    let mut sources = Vec::new();
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .with_context(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--project" => project = Some(PathBuf::from(value)),
            "--provider" => {
                provider = Some(Provider::parse(&value).context("expected codex or claude")?)
            }
            "--session" => session = Some(value),
            "--root" => root = Some(PathBuf::from(value)),
            "--source" => sources.push(Source {
                selector: value,
                id: String::new(),
                fingerprint: String::new(),
                updated_at: String::new(),
                tasks: Vec::new(),
                retired: Vec::new(),
                authority: BTreeSet::new(),
                error: None,
            }),
            _ => bail!("unknown tracking argument {flag:?}"),
        }
    }
    let project = fs::canonicalize(project.context("--project is required")?)
        .context("project root is unavailable")?;
    let session = SessionKey {
        provider: provider.context("--provider is required")?,
        session_id: session.context("--session is required")?,
    };
    if session.session_id.is_empty()
        || session.session_id.len() > 100
        || session.session_id.chars().any(char::is_control)
    {
        bail!("invalid exact session identity");
    }
    let root = match root {
        Some(root) => root,
        None => Store::default_root()?,
    };
    let _lock = lock(&root)?;
    let mut registry = read(&root)?;
    let previous = registry.clone();
    let index = registry
        .bindings
        .iter()
        .position(|binding| binding.project == project && binding.session == session);
    if operation == "enable" {
        if sources.is_empty() || sources.len() > 16 {
            bail!("enable requires 1 to 16 explicit sources");
        }
        if !registry.projects.contains(&project) {
            if registry.projects.len() >= 64 {
                bail!("at most 64 tracked projects");
            }
            registry.projects.push(project.clone());
        }
        let mut seen = BTreeSet::new();
        for source in &mut sources {
            source_path(&project, &source.selector)?;
            if !seen.insert(source.selector.clone()) {
                bail!("duplicate source selector");
            }
            source.id = digest(format!("{}:{}", project.display(), source.selector).as_bytes());
            if let Some(old) = index.and_then(|index| {
                registry.bindings[index]
                    .sources
                    .iter()
                    .find(|old| old.selector == source.selector)
            }) {
                *source = old.clone();
            }
            refresh_source(&project, &session, source);
        }
        let binding = Binding {
            project: project.clone(),
            session: session.clone(),
            enabled: true,
            sources,
        };
        if let Some(index) = index {
            registry.bindings[index] = binding;
        } else {
            if registry.bindings.len() >= 128 {
                bail!("at most 128 goal bindings");
            }
            registry.bindings.push(binding);
        }
    } else {
        if !sources.is_empty() {
            bail!("--source is only accepted by enable");
        }
        if operation == "disable"
            && let Some(index) = index
        {
            registry.bindings[index].enabled = false;
        }
    }
    if operation == "sync"
        && let Some(index) = index
        && registry.bindings[index].enabled
    {
        for source in &mut registry.bindings[index].sources {
            refresh_source(&project, &session, source);
        }
    }
    if registry != previous {
        write(&root, &registry)?;
    }
    let binding = registry
        .bindings
        .iter()
        .find(|binding| binding.project == project && binding.session == session);
    let sources: Vec<_> = binding.into_iter().flat_map(|binding| &binding.sources).map(|source| serde_json::json!({"selector":source.selector,"source_id":source.id,"fingerprint":source.fingerprint,"updated_at":source.updated_at,"tasks":source.tasks.len(),"retired":source.retired.len(),"stale":source.error.is_some(),"error":source.error})).collect();
    println!(
        "{}",
        serde_json::json!({"project":project,"project_opt_in":registry.projects.contains(&project),"session":session,"enabled":binding.is_some_and(|binding| binding.enabled),"sources":sources})
    );
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    // Stable identity/change detector; never used as a security decision.
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
fn parts(selector: &str) -> Result<(&str, Option<&str>)> {
    if let Some(path) = selector.strip_suffix("@ticket") {
        return Ok((path, None));
    }
    let (path, heading) = selector
        .split_once('#')
        .context("source requires FILE#HEADING or FILE@ticket")?;
    if heading.is_empty() || heading.len() > 128 {
        bail!("table heading requires 1 to 128 bytes");
    }
    Ok((path, Some(heading)))
}
fn source_path(project: &Path, selector: &str) -> Result<PathBuf> {
    let (path, _) = parts(selector)?;
    let path = Path::new(path);
    if path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
        || path.extension().is_none_or(|extension| extension != "md")
    {
        bail!("source must be an explicit relative Markdown file within the project");
    }
    let resolved = fs::canonicalize(project.join(path)).context("source file is unavailable")?;
    if !resolved.starts_with(project) {
        bail!("source resolves outside selected project");
    }
    Ok(resolved)
}
fn cells(line: &str) -> Result<Vec<String>> {
    let line = line.trim();
    if !line.starts_with('|') || !line.ends_with('|') {
        bail!("table rows require leading and trailing pipes");
    }
    let mut result = Vec::new();
    let mut cell = String::new();
    let mut escaped = false;
    for ch in line[1..line.len() - 1].chars() {
        if escaped {
            if ch != '|' && ch != '\\' {
                cell.push('\\');
            }
            cell.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '|' {
            result.push(cell.trim().to_owned());
            cell.clear();
        } else {
            cell.push(ch);
        }
    }
    if escaped {
        cell.push('\\');
    }
    result.push(cell.trim().to_owned());
    if result.len() > 16 || result.iter().any(|cell| cell.len() > 1024) {
        bail!("table exceeds 16 columns or 1024 bytes per cell");
    }
    Ok(result)
}
fn field(name: &str) -> Option<&'static str> {
    match name.trim().to_ascii_lowercase().as_str() {
        "id" | "ticket" => Some("id"),
        "task" | "title" => Some("label"),
        "status" => Some("status"),
        "owner" => Some("owner"),
        "blocked by" | "depends on" | "dependencies" => Some("depends_on"),
        "purpose" => Some("purpose"),
        "change" => Some("change"),
        "next" => Some("next"),
        "blocker" => Some("blocker"),
        _ => None,
    }
}
fn parse_table(
    text: &str,
    heading: &str,
) -> Result<(Vec<BTreeMap<String, String>>, BTreeSet<String>)> {
    let mut in_fence = false;
    let mut lines = Vec::new();
    let mut heading_level = None;
    let mut matches = 0;
    let mut active = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let level = trimmed.chars().take_while(|ch| *ch == '#').count();
        if level > 0 && trimmed.as_bytes().get(level) == Some(&b' ') {
            if trimmed[level + 1..].trim() == heading {
                matches += 1;
                active = true;
                heading_level = Some(level);
            } else if heading_level.is_some_and(|old| level <= old) {
                active = false;
            }
            continue;
        }
        if active {
            lines.push(trimmed);
        }
    }
    if matches != 1 {
        bail!("table heading must identify exactly one section");
    }
    let start = lines
        .iter()
        .position(|line| line.starts_with('|'))
        .context("selected section has no task table")?;
    let headers = cells(lines[start])?;
    let separator = cells(lines.get(start + 1).context("missing table separator")?)?;
    if separator.len() != headers.len()
        || separator.iter().any(|cell| {
            cell.trim_matches(':').len() < 3 || !cell.trim_matches(':').chars().all(|ch| ch == '-')
        })
    {
        bail!("invalid Markdown table separator");
    }
    let fields: Vec<_> = headers.iter().map(|header| field(header)).collect();
    let authority: BTreeSet<_> = fields
        .iter()
        .flatten()
        .map(|field| (*field).to_owned())
        .collect();
    if fields.iter().flatten().count() != authority.len()
        || !["id", "label", "status"]
            .iter()
            .all(|name| authority.contains(*name))
    {
        bail!("table requires unique ID, Task and Status columns");
    }
    let mut rows = Vec::new();
    for line in lines
        .iter()
        .skip(start + 2)
        .take_while(|line| line.starts_with('|'))
    {
        let values = cells(line)?;
        if values.len() != headers.len() {
            bail!("table row has wrong cell count");
        }
        rows.push(
            fields
                .iter()
                .zip(values)
                .filter_map(|(field, value)| field.map(|field| (field.to_owned(), value)))
                .collect(),
        );
        if rows.len() > 512 {
            bail!("source exceeds 512 tasks");
        }
    }
    Ok((rows, authority))
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}
fn task(
    source: &Source,
    session: &SessionKey,
    values: &BTreeMap<String, String>,
    fingerprint: &str,
    time: &str,
) -> Result<SourceTask> {
    let value = |key: &str| values.get(key).map(String::as_str).unwrap_or("");
    let task_id = value("id");
    if !identifier(task_id) || value("label").is_empty() || value("label").len() > 512 {
        bail!("task requires stable ID (1..64 ASCII letters/digits/._-) and title (1..512 bytes)");
    }
    let qualify = |id: &str| format!("source:{}:{id}", source.id);
    let status_label = value("status").to_owned();
    let status = match status_label.to_ascii_lowercase().as_str() {
        "planned" | "pending" | "ready-for-agent" => Some(WorkStatus::Planned),
        "active" | "in_progress" | "in-progress" => Some(WorkStatus::Active),
        "blocked" => Some(WorkStatus::Blocked),
        "done" | "completed" | "resolved" => Some(WorkStatus::Done),
        "cancelled" | "canceled" => Some(WorkStatus::Cancelled),
        _ => None,
    };
    let owner_label = value("owner").to_owned();
    let native = if owner_label == "main" || owner_label.starts_with("/root/") {
        Some(owner_label.clone())
    } else {
        owner_label
            .strip_prefix("native:")
            .filter(|id| !id.is_empty() && id.len() <= 100 && !id.chars().any(char::is_control))
            .map(str::to_owned)
    };
    let owner = native.map(|agent_id| AgentKey {
        session: session.clone(),
        agent_id,
    });
    let dependencies = value("depends_on");
    let depends_on = if dependencies.is_empty() {
        None
    } else if dependencies.eq_ignore_ascii_case("none") || dependencies == "[]" {
        Some(Vec::new())
    } else {
        let ids: Vec<_> = dependencies.split(',').map(str::trim).collect();
        if ids.len() > 64 || ids.iter().any(|id| !identifier(id)) {
            bail!("dependencies require at most 64 comma-separated IDs, None, [] or blank");
        }
        Some(ids.into_iter().map(qualify).collect())
    };
    let context = |name| values.get(name).filter(|value| !value.is_empty()).cloned();
    Ok(SourceTask {
        id: qualify(task_id),
        source_id: source.id.clone(),
        task_id: task_id.into(),
        label: value("label").into(),
        status,
        status_label,
        owner,
        owner_label,
        depends_on,
        parent_id: None,
        purpose: context("purpose"),
        change: context("change"),
        next: context("next"),
        blocker: context("blocker"),
        outcome_links: Vec::new(),
        source: source.selector.clone(),
        updated_at: time.into(),
        fingerprint: fingerprint.into(),
        provenance: source
            .authority
            .iter()
            .map(|field| (field.clone(), format!("{}@{fingerprint}", source.selector)))
            .collect(),
        stale: false,
    })
}
fn refresh_source(project: &Path, session: &SessionKey, source: &mut Source) {
    let result = (|| -> Result<()> {
        let path = source_path(project, &source.selector)?;
        let file = fs::File::open(path)?;
        if !file.metadata()?.is_file() || file.metadata()?.len() > MAX_SOURCE_BYTES {
            bail!("source must be a regular file of at most 256 KiB");
        }
        let mut bytes = Vec::new();
        file.take(MAX_SOURCE_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_SOURCE_BYTES {
            bail!("source exceeds 256 KiB");
        }
        let fingerprint = digest(&bytes);
        if fingerprint == source.fingerprint && source.error.is_none() {
            return Ok(());
        }
        let text = std::str::from_utf8(&bytes).context("source is not UTF-8")?;
        let (_, heading) = parts(&source.selector)?;
        let (rows, authority) = parse_table(
            text,
            heading.context("ticket metadata is not supported yet")?,
        )?;
        source.authority.extend(authority);
        let time = chrono::Utc::now().to_rfc3339();
        let tasks: Vec<_> = rows
            .iter()
            .map(|values| task(source, session, values, &fingerprint, &time))
            .collect::<Result<_>>()?;
        let ids: BTreeSet<_> = tasks.iter().map(|task| &task.id).collect();
        if ids.len() != tasks.len() {
            bail!("duplicate task IDs in source");
        }
        if tasks
            .iter()
            .map(|task| task.depends_on.as_ref().map_or(0, Vec::len))
            .sum::<usize>()
            > 8192
        {
            bail!("source exceeds 8192 edges");
        }
        source.retired.extend(
            source
                .tasks
                .iter()
                .filter(|task| !ids.contains(&task.id))
                .cloned(),
        );
        if source.retired.len() > 512 {
            source.retired.drain(..source.retired.len() - 512);
        }
        source.tasks = tasks;
        source.fingerprint = fingerprint;
        source.updated_at = time;
        Ok(())
    })();
    source.error = result.err().map(|error| error.to_string());
    for task in &mut source.tasks {
        task.stale = source.error.is_some();
    }
}

/// Populate cached source observations; this does not read project files.
pub(crate) fn project(root: &Path, record: &mut WorkRecord) -> Result<()> {
    record.source_tasks.clear();
    record.source_diagnostics.clear();
    for binding in read(root)?
        .bindings
        .iter()
        .filter(|binding| binding.session == record.session)
    {
        for source in &binding.sources {
            record.source_tasks.extend(source.tasks.clone());
            if !binding.enabled {
                record
                    .source_diagnostics
                    .push(format!("Tracking disabled: {}", source.selector));
            }
            if let Some(error) = &source.error {
                record
                    .source_diagnostics
                    .push(format!("Stale {}: {error}", source.selector));
            }
            for task in &source.tasks {
                if task.status.is_none() {
                    record.source_diagnostics.push(format!(
                        "Unknown status for {}: {}",
                        task.task_id, task.status_label
                    ));
                }
            }
        }
    }
    record.source_diagnostics.truncate(64);
    Ok(())
}
