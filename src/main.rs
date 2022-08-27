use crossterm::cursor::MoveToColumn;
use crossterm::queue;
use crossterm::{
    cursor::{self, MoveTo, MoveToNextLine},
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, SetBackgroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
    },
    QueueableCommand, Result,
};
use std::fmt::Write as fmtWrite;
use std::io::{BufWriter, Write};
use std::num::Wrapping;
use std::{fmt::Display, io::stdout};

fn main() -> Result<()> {
    let mut stdout = BufWriter::new(stdout());
    start(&mut stdout)?;
    let mut dbg = DebugPrinter {
        pos: (20, 20),
        width: 50,
        buf: String::new(),
    };
    let mut next_color_code: Wrapping<u8> = Wrapping(0);
    let mut pos = 0;
    let (col, row) = size()?;
    loop {
        match read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => pos += col,
            Event::Key(KeyEvent {
                code: KeyCode::Char(char),
                ..
            }) => {
                stdout
                    .queue(MoveTo(pos % col, pos / col))?
                    .queue(SetBackgroundColor(Color::AnsiValue(next_color_code.0)))?;
                write!(stdout, "{char}")?;
                pos += 1;
                next_color_code += 1;
                dbg.print(format_args!("size: {},{}", row, col));
                for _ in (pos % col)..col {
                    write!(stdout, " ")?;
                }
                for _ in (pos / col + 1)..row {
                    for _ in 0..col {
                        write!(stdout, " ")?;
                    }
                }
            }
            Event::Mouse(_) => (),
            Event::Paste(_) => (),
            Event::Resize(_, _) => (),
            _ => (),
        }
        // Debug Info
        queue!(stdout, MoveTo(30, 20), SetBackgroundColor(Color::Black))?;
        dbg.print(format_args!("pos: {pos}"));
        dbg.print(format_args!("next_color_code: {next_color_code}"));
        dbg.print(format_args!("size: {:?}", size()?));
        dbg.flush(&mut stdout)?;
        stdout.flush()?;
    }
    cleanup(&mut stdout)?;
    Ok(())
}

struct DebugPrinter {
    pos: (u16, u16),
    width: usize,
    buf: String,
}

impl DebugPrinter {
    fn flush(&mut self, mut writer: impl Write) -> Result<()> {
        write!(
            writer,
            "{}{}{}",
            MoveTo(self.pos.0, self.pos.1),
            SetBackgroundColor(Color::Black),
            self.buf
        )?;
        self.buf.clear();
        Ok(())
    }

    fn print<D: Display>(&mut self, d: D) {
        self.buf
            .write_fmt(format_args!("{d:-<0$}", self.width))
            .unwrap();

        self.buf
            .write_fmt(format_args!(
                "{}{}",
                MoveToNextLine(1),
                MoveToColumn(self.pos.1)
            ))
            .unwrap();
    }
}

fn start<W: Write>(mut writer: W) -> Result<()> {
    enable_raw_mode()?;
    execute!(
        writer,
        cursor::SavePosition,
        cursor::Hide,
        EnableMouseCapture,
        EnterAlternateScreen,
        MoveTo(0, 0)
    )?;
    Ok(())
}

fn cleanup<W: Write>(mut writer: W) -> Result<()> {
    execute!(
        writer,
        LeaveAlternateScreen,
        DisableMouseCapture,
        cursor::RestorePosition,
        cursor::Show
    )?;
    disable_raw_mode()?;
    Ok(())
}
