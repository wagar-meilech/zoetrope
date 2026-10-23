//! Explicit native document opening. Successful dispatch is not render verification.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

const BRIDGE_ID: &str = "wagar.zoetrope-preview";

/// The strongest claim supported by a successful editor dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentLaunchKind {
    PreviewRequested,
    EditorRequested { reason: String },
}

/// The selected document and the request sent to the external editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentLaunch {
    pub path: PathBuf,
    pub kind: DocumentLaunchKind,
}

impl fmt::Display for DocumentLaunch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            DocumentLaunchKind::PreviewRequested => {
                write!(f, "Requested VS Code preview: {}", self.path.display())
            }
            DocumentLaunchKind::EditorRequested { reason } => {
                write!(
                    f,
                    "Requested VS Code file: {} ({reason})",
                    self.path.display()
                )
            }
        }
    }
}

/// A document that could not be dispatched, with a reason suitable for the UI.
#[derive(Debug)]
pub struct DocumentLaunchError {
    pub path: PathBuf,
    pub reason: String,
}

impl fmt::Display for DocumentLaunchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cannot open document {}: {}",
            self.path.display(),
            self.reason
        )
    }
}

impl std::error::Error for DocumentLaunchError {}

/// OS/VS Code executable adapter. Commands always receive separate arguments.
#[derive(Debug, Clone)]
pub struct DocumentLauncher {
    code_program: PathBuf,
    uri_opener_program: PathBuf,
}

impl Default for DocumentLauncher {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        let opener = "open";
        #[cfg(target_os = "windows")]
        let opener = "explorer.exe";
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let opener = "xdg-open";
        Self::new("code", opener)
    }
}

impl DocumentLauncher {
    /// Override external executable locations, for example for a different VS Code install.
    pub fn new(code_program: impl Into<PathBuf>, uri_opener_program: impl Into<PathBuf>) -> Self {
        Self {
            code_program: code_program.into(),
            uri_opener_program: uri_opener_program.into(),
        }
    }

    /// Request MD Reader for the exact selected Markdown file, without a shell or a model call.
    ///
    /// Blocking native IO: call this off the UI thread, only for an explicit open action.
    /// Checks metadata without reading document contents. Returns dispatch state, not render proof.
    pub fn open(&self, selected: impl AsRef<Path>) -> Result<DocumentLaunch, DocumentLaunchError> {
        let selected = selected.as_ref();
        let fail = |reason: String| DocumentLaunchError {
            path: selected.to_path_buf(),
            reason,
        };
        let path = if selected.is_absolute() {
            selected.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|error| fail(error.to_string()))?
                .join(selected)
        };
        let metadata = std::fs::metadata(&path)
            .map_err(|error| fail(format!("document is unavailable: {error}")))?;
        if !metadata.is_file() {
            return Err(fail("selected document is not a regular file".into()));
        }
        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown")
            })
        {
            return Err(fail("select a Markdown file (.md or .markdown)".into()));
        }
        let text = path
            .to_str()
            .ok_or_else(|| fail("the selected path is not valid Unicode".into()))?;
        #[cfg(target_os = "windows")]
        let text = format!("/{}", text.replace('\\', "/"));
        #[cfg(target_os = "windows")]
        let text = text.as_str();
        let file_uri = format!("file://{}", percent_encode(text, true));
        let uri = format!(
            "vscode://{BRIDGE_ID}/open?uri={}",
            percent_encode(&file_uri, false)
        );
        let extensions = Command::new(&self.code_program)
            .arg("--list-extensions")
            .output();
        let bridge_available = extensions.is_ok_and(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .any(|id| id.trim().eq_ignore_ascii_case(BRIDGE_ID))
        });
        let reason = if bridge_available {
            match Command::new(&self.uri_opener_program).arg(uri).output() {
                Ok(output) if output.status.success() => {
                    return Ok(DocumentLaunch {
                        path,
                        kind: DocumentLaunchKind::PreviewRequested,
                    });
                }
                Ok(output) => format!("preview dispatch failed ({})", output.status),
                Err(error) => format!("preview dispatch failed: {error}"),
            }
        } else {
            "preview bridge is unavailable".into()
        };
        let output = Command::new(&self.code_program)
            .arg("--reuse-window")
            .arg(&path)
            .output()
            .map_err(|error| fail(format!("{reason}; VS Code file dispatch failed: {error}")))?;
        if !output.status.success() {
            return Err(fail(format!(
                "{reason}; VS Code file dispatch failed ({})",
                output.status
            )));
        }
        Ok(DocumentLaunch {
            path,
            kind: DocumentLaunchKind::EditorRequested {
                reason: format!("{reason}; use MD Reader: Full Render in VS Code"),
            },
        })
    }
}

fn percent_encode(text: &str, preserve_slashes: bool) -> String {
    use std::fmt::Write;
    let mut encoded = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'.' | b'_' | b'~')
            || (preserve_slashes && byte == b'/')
        {
            encoded.push(char::from(byte));
        } else {
            write!(encoded, "%{byte:02X}").expect("writing to String cannot fail");
        }
    }
    encoded
}
