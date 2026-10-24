use crossterm::event::{KeyCode,KeyEvent,KeyModifiers};
/// Small UTF-8 editor; cursor offsets always stay at character boundaries.
pub(super) struct Editor{pub text:String,pub cursor:usize}
impl Editor{
 pub fn new(text:String)->Self{let cursor=text.len();Self{text,cursor}}
 pub fn key(&mut self,key:&KeyEvent)->bool{
  match key.code{
   KeyCode::Char('u')if key.modifiers.contains(KeyModifiers::CONTROL)=>{self.text.clear();self.cursor=0;return true;},
   KeyCode::Char('a')if key.modifiers.contains(KeyModifiers::CONTROL)=>self.cursor=0,
   KeyCode::Char('e')if key.modifiers.contains(KeyModifiers::CONTROL)=>self.cursor=self.text.len(),
   KeyCode::Left=>self.cursor=self.text[..self.cursor].char_indices().next_back().map_or(0,|(i,_)|i),
   KeyCode::Right=>{if let Some(ch)=self.text[self.cursor..].chars().next(){self.cursor+=ch.len_utf8();}},
   KeyCode::Home=>self.cursor=self.text[..self.cursor].rfind('\n').map_or(0,|i|i+1),
   KeyCode::End=>self.cursor+=self.text[self.cursor..].find('\n').unwrap_or(self.text.len()-self.cursor),
   KeyCode::Backspace if self.cursor>0=>{let previous=self.text[..self.cursor].char_indices().next_back().unwrap().0;self.text.drain(previous..self.cursor);self.cursor=previous;return true;},
   KeyCode::Delete if self.cursor<self.text.len()=>{let n=self.text[self.cursor..].chars().next().unwrap().len_utf8();self.text.drain(self.cursor..self.cursor+n);return true;},
   KeyCode::Enter if self.text.len()<65536=>{self.text.insert(self.cursor,'\n');self.cursor+=1;return true;},
   KeyCode::Char(ch)if !key.modifiers.intersects(KeyModifiers::CONTROL|KeyModifiers::ALT)&&self.text.len()+ch.len_utf8()<=65536=>{self.text.insert(self.cursor,ch);self.cursor+=ch.len_utf8();return true;},
   _=>{},
  }false
 }
}
