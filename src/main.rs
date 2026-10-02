use std::io;
use std::io::Write;
use crossterm::{cursor::MoveTo, event::{self, Event::{self, Key}, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind::Moved}, execute, queue, terminal::{self, ClearType::All, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}};

struct Pos {
}

struct Editor{
    content:Vec<String>,
    row: u32,
    column: u32,
    height: u32,
    wedth:u32,
    dirty:bool,
}    

impl Editor {
    fn new()->Self{
        Self { content: vec![String::new()], row: 0, column: 0, height: 0, wedth: 0, dirty: true }
    }
    fn is_dirty(&self)->bool{
        self.dirty
    }
    fn set_dirty(&mut self, dirty:bool){
        self.dirty=dirty;
    }
    fn update_pos(&mut self, row: u32, column: u32) {
        self.row = row;
        self.column = column;
        self.dirty = true;
    }
    fn push_str(&mut self,str_line:String){
        self.content.push(str_line);
    }
    fn insert(&mut self,c:char,row:u32){
        if let Some(line)=self.content.get_mut(row as usize){
            line.push(c);
            if self.row<self.height{
                self.row+=1;
            }
        }
    }
}
    

fn main()->io::Result<()>{
    let mut stdout=io::stdout();
    enable_raw_mode()?;
    execute!(stdout,EnterAlternateScreen)?;
    
    execute!(stdout,MoveTo(0,0))?;
    let mut ed=Editor::new();
    let (w,h) =terminal::size()?;

    writeln!(stdout,"info")?;
    loop{
        match event::read()? {
            Event::Key(k) if k.kind==KeyEventKind::Press=>{
                if k.modifiers.contains(KeyModifiers::CONTROL) && k.code==KeyCode::Char('q'){
                    break;
                }
                if let KeyCode::Char(c)=k.code{
                    if !k.modifiers.contains(KeyModifiers::CONTROL){

                    }
                }

            },
            _ =>{}
                    
                }        
}

    disable_raw_mode()?;
    execute!(stdout,LeaveAlternateScreen)?;
    Ok(())
}
