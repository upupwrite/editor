use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    event::{
        self,
        Event::{self, Key},
        KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
        MouseEventKind::Moved,
    },
    execute, queue,
    style::Print,
    terminal::{
        self, Clear, ClearType::All, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use std::io::Write;
use std::io::{self, stdout};

enum Move {
    left,
    right,
    up,
    down,
}

struct Editor {
    content: Vec<String>,
    row: u16,
    column: u16,
    height: u16,
    wedth: u16,
    row_offset: u16,
    column_offset: u16,
    dirty: bool,
}

impl Editor {
    fn new() -> Self {
        Self {
            content: vec![String::new()],
            row: 0,
            column: 0,
            height: 0,
            wedth: 0,
            row_offset: 0,
            column_offset: 0,
            dirty: true,
        }
    }
    fn is_dirty(&self) -> bool {
        self.dirty
    }
    fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
    fn update_pos(&mut self, row: u16, column: u16) {
        self.row = row;
        self.column = column;
        self.dirty = true;
    }
    fn push_str(&mut self, str_line: String) {
        self.content.push(str_line);
    }
    fn insert(&mut self, c: char, row: u16) {
        if let Some(line) = self.content.get_mut(row as usize) {
            line.push(c);
            self.column += 1;
            self.dirty = true;
        }
    }
    fn roll(&mut self) {}

    fn enter(&mut self, stdout: &mut impl Write) -> io::Result<()> {
        self.content.push(String::new());
        self.row += 1;
        if self.row >= self.height {
            self.row_offset += 1;
        }
        queue!(stdout, MoveTo(self.row, 0));
        self.dirty = true;
        Ok(())
    }
    fn move_cursor(&mut self, stdout: &mut impl Write, kind: Move, step: u16) -> io::Result<()> {
        match kind {
            Move::left => self.column = self.column.saturating_sub(step),
            Move::right => self.column = self.column.saturating_add(step),
            Move::up => self.row = self.row.saturating_sub(step),
            Move::down => self.row = self.row.saturating_add(step),
        }
        queue!(stdout, MoveTo(self.column, self.row))?;
        Ok(())
    }

    fn render(&mut self, stdout: &mut impl Write) -> io::Result<()> {
        queue!(stdout, Clear(terminal::ClearType::All))?;
        for (row, text) in self.content.iter().enumerate() {
            if row >= self.row_offset.into() {
                let screen_row = (row - self.row_offset as usize) as u16;
                queue!(stdout, MoveTo(0, screen_row))?;
                queue!(stdout, Print(text))?;
            }
        }
        self.dirty = false;
        stdout.flush()?;
        Ok(())
    }
}

//                            y(row)
//                            ↑
//                            |
//                            |
//                    --------+--------→ x(column)
//                            |
//                            |

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    execute!(stdout, MoveTo(0, 0))?;
    let mut ed = Editor::new();
    let (w, h) = terminal::size()?;
    ed.wedth = w;
    ed.height = h;

    writeln!(stdout, "info")?;
    loop {
        match event::read()? {
            Event::Key(k) if k.kind == KeyEventKind::Press => {
                if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('q') {
                    break;
                }
                if let KeyCode::Char(c) = k.code {
                    if !k.modifiers.contains(KeyModifiers::CONTROL) {
                        ed.insert(c, ed.row);
                    }
                }
                if let KeyCode::Enter = k.code {
                    ed.enter(&mut stdout)?;
                }
                match k.code {
                    KeyCode::Down => {
                        ed.move_cursor(&mut stdout, Move::down, 1)?;
                        stdout.flush()?;
                    }
                    KeyCode::Up => {
                        ed.move_cursor(&mut stdout, Move::up, 1)?;
                        stdout.flush()?;
                    }
                    KeyCode::Left => {
                        ed.move_cursor(&mut stdout, Move::left, 1)?;
                        stdout.flush()?;
                    }
                    KeyCode::Right => {
                        ed.move_cursor(&mut stdout, Move::right, 1)?;
                        stdout.flush()?;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        if ed.dirty {
            ed.render(&mut stdout)?;
        }
    }

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    for text in ed.content {
        println!("{text}");
    }
    Ok(())
}
