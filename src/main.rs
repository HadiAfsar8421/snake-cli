use crossterm::{
    QueueableCommand, cursor, event::{self, Event, KeyCode}, execute, queue, style, terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::{io::{self, Write, stdout}, time::{Duration, Instant}};
use rand;

use crate::CollisionType::{Death, None};

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

fn generate_food_position(width: u16, height: u16) -> Position {
    (rand::random_range(0..width), rand::random_range(0..height))
}

fn main() -> Result<(), io::Error> {
    let mut stdout = stdout();

    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::Clear(ClearType::All),
    )?;
    enable_raw_mode()?;

    let width: u16 = 64;
    let height: u16 = 64;

    let mut game = GameState::new(width, height);
    let mut high_score = game.score;
    let borders = generate_borders(width + 2, height + 2);

    let mut time_since_last_tick = Instant::now();
    let tick_interval = Duration::from_millis(200);

    loop {
        
        queue!(stdout, terminal::Clear(ClearType::All))?;

        let (cur_width, cur_height) = terminal::size()?;
        if cur_width < (width + 2) || (cur_height < height) {
            execute!(
                stdout, 
                cursor::MoveTo(0, 0),
                style::Print("Terminal too small, please increase terminal size to at least 66x66\n")
            )?;
            continue; 
        }
        let x_off = (cur_width - (width + 2)) / 2;
        let y_off = (cur_height - (height + 2)) / 2;

        for (x, y) in borders.clone() {
            queue!(
                stdout, 
                cursor::MoveTo(x + x_off, y + y_off),
                style::Print("#"),
            )?;
        }

        if event::poll(Duration::from_millis(0))?  {
            if let Event::Key(key) = event::read()? && key.is_press() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('r') | KeyCode::Char('R') => game.reset(width, height),
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => game.change_direction(Direction::UP),
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => game.change_direction(Direction::DOWN),
                    KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => game.change_direction(Direction::LEFT),
                    KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => game.change_direction(Direction::RIGHT),
                    _ => {}
                }
            }
        }

    }

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

struct GameState {
    snake: Snake,
    input_dir: Direction,
    food_pos: Position,
    score: u64,
}

impl GameState {
    fn new(width: u16, height: u16) -> Self {
        Self { snake: Snake::new((width / 2, height / 2), Direction::RIGHT), input_dir: Direction::RIGHT, food_pos: generate_food_position(width, height), score: 1 }
    }

    fn reset(&mut self, width: u16, height: u16) {
        self.snake = Snake::new((width / 2, height / 2), Direction::RIGHT);
        self.input_dir = Direction::RIGHT;
        self.food_pos = generate_food_position(width, height);
        self.score = 1
    } 

    fn handle_collision(&mut self, border: Vec<Position>) -> CollisionType {
        if self.snake.is_colliding(&border, true) {
            return Death(self.score)
        } else if self.snake.is_colliding(&[self.food_pos], false) {
            self.score += 1;
        }
        return None
    }

    fn advance(&mut self) {
        self.snake.advance();
    }

    fn change_direction(&mut self, direction: Direction) {
        self.snake.change_direction(direction);
    }

    fn snake_body(&self) -> &[Position] {
        &self.snake.body[1..]
    }

    fn snake_head(&self) -> &Position {
        &self.snake.body[0]
    }
}

enum CollisionType {
    Death(u64),
    None,
}

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
