//! Native fleet frontend. Drawing and input consume cached data; local IO lives
//! on workers. The public key/render interface is shared by the terminal and tests.
use std::collections::{BTreeMap,BTreeSet};
use std::path::PathBuf;
use std::sync::{Arc,Mutex,mpsc};
use std::thread;
use std::time::{Duration,Instant,SystemTime};

use crossterm::event::{Event,KeyCode,KeyEventKind,KeyModifiers};
use rataflow::{Flow,Node,Edge,NodeContent,NodeRenderContext,StepEdge,Handle,HandlePosition,Reconnectable};
use ratatui::{Frame,buffer::Buffer,layout::{Rect,Layout,Constraint},style::{Style,Color,Modifier},widgets::{Block,Borders,Paragraph,Widget,Wrap}};
use crate::console::{Console,ConsoleCommand,DiscoveryRoots,FleetGraph,FleetNodeId,SessionKey};
use crate::documents::DocumentLauncher;
use crate::workstate::{Store,WorkRecord,WorkSummary};

/// Native locations and external adapters. Construction does no local IO.
#[derive(Clone)]
pub struct FleetConfig {
 pub roots:DiscoveryRoots,
 pub state_root:PathBuf,
 pub socket:PathBuf,
 pub documents:DocumentLauncher,
}
impl Default for FleetConfig {
 fn default()->Self{Self{roots:DiscoveryRoots::local(),state_root:Store::default_root().unwrap_or_else(|_|std::env::temp_dir().join("zoetrope-state")),socket:crate::control::default_socket(),documents:DocumentLauncher::default()}}
}
#[derive(Default)]
struct InventoryView {
 graph:FleetGraph,
 summaries:BTreeMap<SessionKey,WorkSummary>,
 selected:Option<WorkRecord>,
 path:Option<PathBuf>,
 selection:Option<FleetNodeId>,
 errors:Vec<String>,
}
enum InventoryCommand { Navigate(ConsoleCommand),Shutdown }
struct Inventory {
 tx:mpsc::SyncSender<InventoryCommand>,
 latest:Arc<Mutex<Option<InventoryView>>>,
 thread:Option<thread::JoinHandle<()>>,
}
impl Inventory {
 fn start(config:&FleetConfig,notify:Arc<tokio::sync::Notify>)->Self{
  let (tx,rx)=mpsc::sync_channel(32);let latest=Arc::new(Mutex::new(None));let output=latest.clone();let roots=config.roots.clone();let root=config.state_root.clone();
  let worker=thread::spawn(move||{
   let mut console=Console::with_state(roots,root);let mut refresh=Instant::now();
   loop{
    if Instant::now()>=refresh {console.refresh(SystemTime::now());refresh=Instant::now()+Duration::from_secs(5);}
    if console.selection().is_none(){if let Some(node)=console.graph().nodes.iter().find(|n|matches!(n.id,FleetNodeId::Session(_))){let id=node.id.clone();console.command(ConsoleCommand::Select(id));}}
    let selected_key=console.selection().and_then(session_key).cloned();
    let selected=selected_key.as_ref().and_then(|key|console.load_work(key).ok().cloned());
    let mut summaries=BTreeMap::new();for node in &console.graph().nodes{if let Some(key)=session_key(&node.id){if let Some(summary)=console.work_summary(key){summaries.insert(key.clone(),summary.clone());}}}
    let view=InventoryView{graph:console.graph().clone(),summaries,selected,path:console.selected_session().map(|s|s.root.path.clone()),selection:console.selection().cloned(),errors:console.work_errors().to_vec()};
    *output.lock().unwrap_or_else(|e|e.into_inner())=Some(view);notify.notify_one();
    match rx.recv_timeout(refresh.saturating_duration_since(Instant::now())){
     Ok(InventoryCommand::Navigate(command))=>console.command(command),
     Ok(InventoryCommand::Shutdown)|Err(mpsc::RecvTimeoutError::Disconnected)=>break,
     Err(mpsc::RecvTimeoutError::Timeout)=>{},
    }
   }
  });Self{tx,latest,thread:Some(worker)}
 }
}
fn session_key(id:&FleetNodeId)->Option<&SessionKey>{match id{FleetNodeId::Session(key)=>Some(key),FleetNodeId::Agent(agent)=>Some(&agent.session),_=>None}}
fn flow_id(id:&FleetNodeId)->String{serde_json::to_string(id).expect("fleet identity serializes")}
#[derive(Debug,Clone,PartialEq)]
struct Card{title:String,detail:String,progress:String}
impl NodeContent for Card{
 fn render(&self,ctx:&NodeRenderContext,buf:&mut Buffer){
  let color=if ctx.selected{Color::Yellow}else{Color::DarkGray};
  let mut lines=crate::ui::wrap(&self.title,ctx.area.width.saturating_sub(2)as usize,2);if !self.detail.is_empty(){lines.push(self.detail.clone());}if !self.progress.is_empty(){lines.push(self.progress.clone());}
  Paragraph::new(lines.join("\n")).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(color))).wrap(Wrap{trim:true}).render(ctx.area,buf);
 }
}
/// Cached terminal state. All inputs are public crossterm events; `poll` applies
/// worker results, `draw` does no filesystem/socket/editor work.
pub struct FleetConsole{
 inventory:Inventory,
 notify:Arc<tokio::sync::Notify>,
 view:InventoryView,
 flow:Flow<Card,StepEdge>,
 positions:BTreeMap<FleetNodeId,(f64,f64)>,
 history:bool,
 pending_center:bool,
 pub(crate) quit:bool,
 status:String,
}
impl FleetConsole{
 pub fn new(config:FleetConfig)->Self{
  let notify=Arc::new(tokio::sync::Notify::new());let inventory=Inventory::start(&config,notify.clone());
  Self{inventory,notify,view:InventoryView::default(),flow:Flow::new().with_deselect_on_pane_click(false).with_min_zoom(0.1),positions:BTreeMap::new(),history:false,pending_center:true,quit:false,status:"Reading local work…".into()}
 }
 /// Apply cached worker output. Returns whether the display changed.
 pub fn poll(&mut self)->bool{
  let view=self.inventory.latest.lock().unwrap_or_else(|e|e.into_inner()).take();
  let Some(view)=view else{return false;};
  // Equality is over the compact visual projection, not native transcript bytes.
  let changed=serde_json::to_string(&self.view.graph).ok()!=serde_json::to_string(&view.graph).ok()||self.view.selection!=view.selection||self.view.selected.as_ref().map(|r|r.progress().label)!=view.selected.as_ref().map(|r|r.progress().label)||self.view.errors!=view.errors;
  self.view=view;if changed{self.status=self.view.errors.first().cloned().unwrap_or_else(||"Local monitoring · no model calls".into());self.sync_graph();}changed
 }
 fn sync_graph(&mut self){
  let ids:BTreeSet<_>=self.view.graph.nodes.iter().map(|n|flow_id(&n.id)).collect();let remove:Vec<_>=self.flow.nodes().filter(|n|!ids.contains(&n.id)).map(|n|n.id.clone()).collect();for id in remove{self.flow.remove_node(&id);}
  let mut depths=BTreeMap::new();
  for (at,node)in self.view.graph.nodes.iter().enumerate(){
   let depth=node.parent.as_ref().and_then(|p|depths.get(p)).copied().unwrap_or(0usize)+usize::from(node.parent.is_some());depths.insert(node.id.clone(),depth.min(8));
   let pos=*self.positions.entry(node.id.clone()).or_insert((depth.min(8)as f64*36.0,at as f64*8.0));
   let progress=session_key(&node.id).and_then(|k|self.view.summaries.get(k)).map(|s|s.progress.label.clone()).unwrap_or_default();
   let title=match &node.id{FleetNodeId::Project(path)=>PathBuf::from(path).file_name().map(|n|n.to_string_lossy().into_owned()).unwrap_or_else(||path.clone()),FleetNodeId::Agent(agent)if agent.agent_id=="main"=>"Coordinator".into(),FleetNodeId::Session(key)if node.label==key.session_id=>node.description.clone().unwrap_or_else(||"Untitled goal".into()),FleetNodeId::Agent(agent)if node.label==agent.agent_id=>"Agent".into(),_=>node.label.clone()};
   let card=Card{title,detail:if node.archived{"Archived".into()}else{String::new()},progress:if matches!(node.id,FleetNodeId::Session(_)){progress}else{String::new()}};let id=flow_id(&node.id);
   if let Some(existing)=self.flow.node_content_mut(&id){if existing!=&card{*existing=card;}}else{let n=Node::new(id.clone(),pos,(34.0,7.0),card).with_deletable(false).with_connectable(false).with_handles(vec![Handle::source(HandlePosition::Right).with_hidden(true),Handle::target(HandlePosition::Left).with_hidden(true)]);let _=self.flow.add_node(n);}
   self.flow.set_node_position(&id,pos);
   if let Some(parent)=&node.parent{let _=self.flow.add_edge(Edge::new(format!("edge-{id}"),flow_id(parent),id).with_selectable(false).with_deletable(false).with_reconnectable(Reconnectable::None));}
  }
  if let Some(id)=&self.view.selection{self.flow.select_node(&flow_id(id));}
 }
 fn navigate(&mut self,command:ConsoleCommand){if self.inventory.tx.try_send(InventoryCommand::Navigate(command)).is_err(){self.status="Inventory busy; try again shortly".into();}}
 /// Handle one key/mouse event. Returns true only for an explicit quit.
 pub fn handle_event(&mut self,event:&Event)->bool{
  if let Event::Key(key)=event{if key.kind==KeyEventKind::Release{return false;}match key.code{
   KeyCode::Char('q')=>self.quit=true,
   KeyCode::Char('c')if key.modifiers.contains(KeyModifiers::CONTROL)=>self.quit=true,
   KeyCode::Char('H'|'h')=>{self.history=!self.history;self.positions.clear();self.navigate(ConsoleCommand::ToggleHistory);},
   _=>{},
  }}self.quit
 }
 /// Render cached state into a terminal frame.
 pub fn draw(&mut self,frame:&mut Frame){
  let [header,body,footer]=Layout::vertical([Constraint::Length(2),Constraint::Fill(1),Constraint::Length(2)]).areas(frame.area());
  frame.render_widget(Paragraph::new(format!("Zoetrope · {}\nGoal progress, agents and guidance",if self.history{"All history"}else{"Current work"})).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),header);
  let [canvas,panel]=Layout::horizontal([Constraint::Percentage(60),Constraint::Percentage(40)]).areas(body);
  frame.render_widget(&mut self.flow,canvas);if self.pending_center{self.flow.center_on_selected();self.pending_center=false;frame.render_widget(&mut self.flow,canvas);}
  let details=if let Some(record)=&self.view.selected{format!("{}\n{}",record.goal.as_ref().map(|g|g.title.as_str()).unwrap_or("Selected goal"),record.progress().label)}else{"Select a goal to inspect it".into()};
  frame.render_widget(Paragraph::new(details).block(Block::default().borders(Borders::ALL).title(" Selected work ")).wrap(Wrap{trim:true}),panel);
  frame.render_widget(Paragraph::new(format!("H history · q quit\n{}",self.status)),footer);
 }
 /// Flush and stop workers after an orderly exit. No native work is stopped.
 pub fn shutdown(&mut self){let _=self.inventory.tx.send(InventoryCommand::Shutdown);if let Some(worker)=self.inventory.thread.take(){let _=worker.join();}}
}
impl Drop for FleetConsole{fn drop(&mut self){self.shutdown();}}
