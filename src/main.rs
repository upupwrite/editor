use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    style::Print,
    terminal::{
        self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode,
    },
};
use std::io::{self, Write};

enum Move {
    Left,
    Right,
    Up,
    Down,
}

struct Editor {
    content: Vec<String>,
    row: u16,
    column: u16,       
    height: u16,
    width: u16,
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
            width: 0,
            row_offset: 0,
            column_offset: 0,
            dirty: true,
        }
    }

    fn current_line_len(&self) -> u16 {
        self.content
            .get(self.row as usize)
            .map(|s| s.chars().count() as u16)
            .unwrap_or(0)
    }

    fn insert_char(&mut self, c: char) {
        let row = self.row as usize;
        if row >= self.content.len() {
            return;
        }
        let line = &mut self.content[row];
        let col = (self.column as usize).min(line.chars().count());
        let byte_idx = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert(byte_idx, c);
        self.column += 1;
        self.dirty = true;
    }

    fn enter(&mut self) {
        let row = self.row as usize;
        if row >= self.content.len() {
            return;
        }
        let line = &mut self.content[row];
        let col = (self.column as usize).min(line.chars().count());
        let byte_idx = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        let rest = line.split_off(byte_idx);
        self.content.insert(row + 1, rest);
        self.row += 1;
        self.column = 0;
        self.dirty = true;
    }

    fn move_cursor(&mut self, kind: Move, step: u16) {
        match kind {
            Move::Left => {
                self.column = self.column.saturating_sub(step);
            }
            Move::Right => {
                let max = self.current_line_len();
                self.column = self.column.saturating_add(step).min(max);
            }
            Move::Up => {
                self.row = self.row.saturating_sub(step);
            }
            Move::Down => {
                let max = self.content.len().saturating_sub(1) as u16;
                self.row = self.row.saturating_add(step).min(max);
            }
        }
        let line_len = self.current_line_len();
        self.column = self.column.min(line_len);

        self.dirty = true;
    }

    fn ensure_visible(&mut self) {
        let height = self.height.max(1) as u32;
        let width = self.width.max(1) as u32;

        let row = self.row as u32;
        let max_offset = (self.content.len() as u32).saturating_sub(height);
        let mut offset = self.row_offset as u32;

        if row < offset {
            offset = row;
        } else if row >= offset + height {
            offset = row - height + 1;
        }
        offset = offset.min(max_offset);
        self.row_offset = offset as u16;

        let col = self.column as u32;
        let mut col_off = self.column_offset as u32;

        if col < col_off {
            col_off = col;
        } else if col >= col_off + width {
            col_off = col - width + 1;
        }
        let line_len = self.current_line_len() as u32;
        let max_col_off = line_len.saturating_sub(1);
        col_off = col_off.min(max_col_off);
        self.column_offset = col_off as u16;
    }

    fn render(&mut self, stdout: &mut impl Write) -> io::Result<()> {
        self.ensure_visible();

        queue!(stdout, MoveTo(0, 0))?;
        queue!(stdout, Clear(ClearType::All))?;

        let offset = self.row_offset as usize;
        let height = self.height as usize;
        let col_off = self.column_offset as usize;
        let width = self.width as usize;

        for (i, text) in self
            .content
            .iter()
            .enumerate()
            .skip(offset)
            .take(height)
        {
            let screen_row = (i - offset) as u16;
            let visible: String = text.chars().skip(col_off).take(width).collect();
            queue!(stdout, MoveTo(0, screen_row))?;
            queue!(stdout, Print(visible))?;
        }

        let screen_row = self.row.saturating_sub(self.row_offset);
        let screen_col = self.column.saturating_sub(self.column_offset);
        queue!(stdout, MoveTo(screen_col, screen_row))?;

        self.dirty = false;
        stdout.flush()?;
        Ok(())
    }

    fn resize(&mut self, w: u16, h: u16) {
        self.width = w;
        self.height = h;

        let max_row = self.content.len().saturating_sub(1) as u16;
        self.row = self.row.min(max_row);
        let line_len = self.current_line_len();
        self.column = self.column.min(line_len);

        self.ensure_visible();
        self.dirty = true;
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    let mut ed = Editor::new();
    let (w, h) = terminal::size()?;
    ed.width = w;
    ed.height = h;

    execute!(stdout,MoveTo(0,0))?;
    loop {
        match event::read()? {
            Event::Key(k) if k.kind == KeyEventKind::Press => {
                match k.code {
                    KeyCode::Char('q')
                        if k.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        break;
                    }
                    KeyCode::Char(c)
                        if !k.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        ed.insert_char(c);
                    }
                    KeyCode::Enter => ed.enter(),
                    KeyCode::Up => ed.move_cursor(Move::Up, 1),
                    KeyCode::Down => ed.move_cursor(Move::Down, 1),
                    KeyCode::Left => ed.move_cursor(Move::Left, 1),
                    KeyCode::Right => ed.move_cursor(Move::Right, 1),
                    _ => {}
                }
            }
            Event::Resize(w, h) => {
                ed.resize(w, h);
            }
            _ => {}
        }

        if ed.dirty {
            ed.render(&mut stdout)?;
        }
    }

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;

    for text in &ed.content {
        println!("{text}");
    }
    Ok(())
}
