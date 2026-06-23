use crossterm::{
    QueueableCommand, cursor, event, execute, style, terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write, stdout};

fn generate_borders(height: u16, width: u16) -> Vec<Position> {
    let mut borders = Vec::new();
    
    for x in 0..width {
        borders.push((x, 0));
        borders.push((x, height - 1));
    }

    for y in 1..(height - 1) {
        borders.push((0, y));
        borders.push((width - 1, y));
    }

    borders
}

fn main() -> Result<(), io::Error> {
    let mut stdout = stdout();

    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::SetSize(66, 66),
        terminal::Clear(terminal::ClearType::All),
    )?;
    enable_raw_mode()?;

    let mut snake = Snake::new((32, 32), Direction::RIGHT);

    disable_raw_mode()?;
    execute!(
        stdout,
        terminal::LeaveAlternateScreen
    )?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

type Position = (u16, u16);

#[derive(Debug)]
struct Snake {
    body: Vec<Position>,
    direction: Direction,
    new_segment_position: Position,
}

impl Snake {

    fn new(position: Position, direction: Direction) -> Self {
        return Self { 
            body: Vec::from([position]), 
            direction, 
            new_segment_position: match direction {
                Direction::UP    => (position.0, position.1 + 1),
                Direction::DOWN  => (position.0, position.1 - 1),
                Direction::LEFT  => (position.0 + 1, position.1),
                Direction::RIGHT => (position.0 - 1, position.1),
            }
        }
    }

    fn advance(&mut self) {
        let length = self.body.len();
        self.new_segment_position = self.body[length - 1];
        for i in (1..length).rev() {
            self.body[i] = self.body[i - 1]
        }
        match self.direction {
            Direction::RIGHT => self.body[0].0 = self.body[0].0.saturating_add(1),
            Direction::LEFT => self.body[0].0 = self.body[0].0.saturating_sub(1),
            Direction::UP => self.body[0].0 = self.body[0].1.saturating_sub(1),
            Direction::DOWN => self.body[0].0 = self.body[0].1.saturating_add(1),
        };
    }

    fn change_direction(&mut self, direction: Direction) {
        match (self.direction, direction) {
            (Direction::UP, Direction::DOWN) => {}
            (Direction::DOWN, Direction::UP) => {}
            (Direction::LEFT, Direction::RIGHT) => {}
            (Direction::RIGHT, Direction::LEFT) => {}
            _ => self.direction = direction,
        }
    }

    fn is_colliding(&mut self, collision_positions: &[Position], check_self: bool) -> bool {
        if collision_positions.contains(&self.body[0]) {
            return true
        } else if check_self && self.body[1..].contains(&self.body[0]) {
            return true
        }
        false
    }
}
