use super::*;
use crate::workstate::WorkUpdate;
pub(super) enum StateCommand {
    Load(SessionKey),
    Update(SessionKey, WorkUpdate, String),
    Shutdown,
}
#[derive(Default)]
pub(super) struct StateOutput {
    pub record: Option<(SessionKey, WorkRecord)>,
    pub status: Option<String>,
}
pub(super) struct StateWorker {
    pub tx: mpsc::SyncSender<StateCommand>,
    pub output: Arc<Mutex<StateOutput>>,
    thread: Option<thread::JoinHandle<()>>,
}
impl StateWorker {
    pub fn start(root: PathBuf, notify: Arc<tokio::sync::Notify>) -> Self {
        let (tx, rx) = mpsc::sync_channel(32);
        let output = Arc::new(Mutex::new(StateOutput::default()));
        let shared = output.clone();
        let thread = thread::spawn(move || {
            let store = Store::new(root);
            let mut loaded = BTreeSet::new();
            while let Ok(command) = rx.recv() {
                let (key, result, message) = match command {
                    StateCommand::Shutdown => break,
                    StateCommand::Load(key) => {
                        let mut result = store.read(&key);
                        if loaded.insert(key.clone())
                            && result.as_ref().is_ok_and(|record| {
                                record.questions.iter().any(|q| {
                                    q.state == crate::workstate::QuestionState::Pending
                                        && q.request_id.is_some()
                                })
                            })
                        {
                            result = store
                                .apply(&key, WorkUpdate::ExpirePendingQuestions { target: None });
                        }
                        (key, result, None)
                    }
                    StateCommand::Update(key, update, message) => {
                        let result = store.apply(&key, update);
                        (key, result, Some(message))
                    }
                };
                let mut output = shared.lock().unwrap_or_else(|e| e.into_inner());
                match result {
                    Ok(record) => {
                        output.record = Some((key, record));
                        if let Some(message) = message {
                            output.status = Some(message);
                        }
                    }
                    Err(error) => {
                        output.status = Some(format!("Could not save/read local work: {error}"))
                    }
                }
                drop(output);
                notify.notify_one();
            }
        });
        Self {
            tx,
            output,
            thread: Some(thread),
        }
    }
    pub fn shutdown(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = self.tx.send(StateCommand::Shutdown);
            let _ = thread.join();
        }
    }
}
