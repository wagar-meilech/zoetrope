use super::*;
pub(super) struct DocumentWorker{pub tx:mpsc::SyncSender<PathBuf>,pub output:Arc<Mutex<Option<String>>>}
impl DocumentWorker{
 pub fn start(launcher:DocumentLauncher,notify:Arc<tokio::sync::Notify>)->Self{
  let(tx,rx)=mpsc::sync_channel::<PathBuf>(1);let output=Arc::new(Mutex::new(None));let shared=output.clone();
  // The editor can wait on its own process. It never holds up steering or state
  // saves, and dropping the console does not wait for an external GUI process.
  thread::spawn(move||while let Ok(path)=rx.recv(){let result=match launcher.open(path){Ok(outcome)=>outcome.to_string(),Err(error)=>error.to_string()};*shared.lock().unwrap_or_else(|e|e.into_inner())=Some(result);notify.notify_one();});Self{tx,output}
 }
}
