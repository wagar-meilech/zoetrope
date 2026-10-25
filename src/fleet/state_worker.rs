use super::*;
use crate::workstate::WorkUpdate;
pub(super) enum StateCommand {
    Load(SessionKey),
    Update(SessionKey, Box<WorkUpdate>, String, Option<u64>),
    Flush(
        SessionKey,
        Box<WorkUpdate>,
        mpsc::Sender<Result<(), String>>,
    ),
    Shutdown,
}
#[derive(Default)]
pub(super) struct StateOutput {
    pub record: Option<(SessionKey, WorkRecord)>,
    pub status: Option<String>,
    pub saved: Option<(u64, Result<(), String>)>,
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
                let (key, result, message, save_id) = match command {
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
                        (key, result, None, None)
                    }
                    StateCommand::Flush(key, update, reply) => {
                        let result = store
                            .apply(&key, *update)
                            .map(|_| ())
                            .map_err(|e| e.to_string());
                        let _ = reply.send(result);
                        continue;
                    }
                    StateCommand::Update(key, update, message, save_id) => {
                        let result = store.apply(&key, *update);
                        (key, result, Some(message), save_id)
                    }
                };
                let mut output = shared.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(id) = save_id {
                    // Exactly one editor save may be in flight. Loads and other
                    // writes cannot overwrite its durable acknowledgment.
                    output.saved =
                        Some((id, result.as_ref().map(|_| ()).map_err(|e| e.to_string())));
                }
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
