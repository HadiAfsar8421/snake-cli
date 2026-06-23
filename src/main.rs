use crossterm::{
    QueueableCommand, cursor, event, execute, style,
    terminal::{self, enable_raw_mode},
};
use std::io::{self, Write, stdout};

fn main() -> Result<(), io::Error> {
    let mut stdout = stdout();
    
    execute!(stdout,
        terminal::EnterAlternateScreen,
        terminal::SetSize(66, 66),
        terminal::Clear(terminal::ClearType::All),
    )?;
    enable_raw_mode()?;
    


    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Debug, Clone, Copy)]
struct SnakeUnit (u16, u16);

#[derive(Debug)]
struct Snake {
    head: SnakeUnit,
    body: Vec<SnakeUnit>,
    direction: Direction,
}

impl Snake {
    fn 
}
