use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue, style,
    terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::{
    io::{self, Write, stdout},
    thread,
    time::{Duration, Instant},
};

fn generate_borders(width: i32, height: i32) -> Vec<Position> {
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

fn generate_food_position(width: i32, height: i32, body: &[Position]) -> Option<Position> {
    let mut slots = Vec::new();
    for x in 0..width {
        for y in 0..height {
            let pos = (x, y);
            if !body.contains(&pos) {
                slots.push(pos);
            }
        }
    }

    if slots.is_empty() {
        return None;
    }

    Some(slots[rand::random_range(0..slots.len())])
}

fn main() -> io::Result<()> {
    let mut stdout = stdout();

    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(ClearType::All),
    )?;
    enable_raw_mode()?;

    let width: i32 = 64;
    let height: i32 = 32;

    let mut game = GameState::new(width, height);
    let borders = generate_borders(width + 2, height + 3);

    let tps: u8 = 5;
    let mut last_tick = Instant::now();
    let tick_interval = Duration::from_secs_f32(1.0 / (tps as f32));
    let mut render = true;
    let mut game_over = false;
    let mut game_over_rendered = false;
    let (w, h) = terminal::size()?;
    let mut current_size: (i32, i32) = (w as i32, h as i32);

    loop {
        let (w, h) = terminal::size()?;
        let new_size = (w as i32, h as i32);

        if new_size != current_size {
            render = true;
            current_size = new_size;
        }

        if (current_size.0 < (width + 2)) || (current_size.1 < (height + 3)) {
            execute!(
                stdout,
                cursor::MoveTo(0, 0),
                style::Print("Terminal too small, please increase terminal size\n")
            )?;
            continue;
        }
        let x_off = (current_size.0 - width) / 2;
        let y_off = (current_size.1 - height) / 2;

        if event::poll(Duration::from_millis(0))?
            && let Event::Key(key) = event::read()?
            && key.is_press()
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => break,
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    game.reset();
                    game_over = false;
                    game_over_rendered = false;
                    render = true;
                }
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                    game.change_input_direction(Direction::Up)
                }
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                    game.change_input_direction(Direction::Down)
                }
                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                    game.change_input_direction(Direction::Left)
                }
                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                    game.change_input_direction(Direction::Right)
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_interval && !game_over {
            game.change_direction();
            game.advance();
            game_over = game.handle_collision();

            render = true;
            last_tick = Instant::now()
        }

        if render || (!game_over_rendered && game_over) {
            let text = if game_over {
                if game.score == (width * height) as u64 {
                    String::from("You won!")
                } else {
                    format!("You died! Score: {}", game.score)
                }
            } else {
                format!("Score: {}", game.score)
            };
            queue!(
                stdout,
                terminal::Clear(ClearType::All),
                cursor::MoveTo(x_off.try_into().unwrap(), y_off.try_into().unwrap()),
                style::Print(text)
            )?;

            for &(x, y) in &borders {
                queue!(
                    stdout,
                    cursor::MoveTo(
                        (x + x_off).try_into().unwrap(),
                        (y + y_off + 1).try_into().unwrap()
                    ),
                    style::Print("#")
                )?;
            }

            let (x, y) = game.food_pos;
            queue!(
                stdout,
                cursor::MoveTo(
                    (x + x_off + 1).try_into().unwrap(),
                    (y + y_off + 2).try_into().unwrap()
                ),
                style::Print("$")
            )?;

            let &(x, y) = game.snake_head();
            queue!(
                stdout,
                cursor::MoveTo(
                    (x + x_off + 1).try_into().unwrap(),
                    (y + y_off + 2).try_into().unwrap()
                ),
                style::Print("@")
            )?;

            for &(x, y) in game.snake_body() {
                queue!(
                    stdout,
                    cursor::MoveTo(
                        (x + x_off + 1).try_into().unwrap(),
                        (y + y_off + 2).try_into().unwrap()
                    ),
                    style::Print("o")
                )?;
            }

            if game_over {
                queue!(
                    stdout,
                    cursor::MoveTo(
                        (x + x_off + 1).try_into().unwrap(),
                        (y + y_off + 2).try_into().unwrap()
                    ),
                    style::Print("X")
                )?;
                game_over_rendered = true;
            }

            stdout.flush()?;
            render = false;
        }

        thread::sleep(Duration::from_millis(5));
    }

    disable_raw_mode()?;
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

type Position = (i32, i32);

struct GameState {
    snake: Snake,
    input_dir: Direction,
    food_pos: Position,
    score: u64,
    width: i32,
    height: i32,
}

impl GameState {
    fn new(width: i32, height: i32) -> Self {
        let snake = Snake::new((width / 2, height / 2), Direction::Right);
        let food_pos = generate_food_position(width, height, &snake.body).unwrap();

        Self {
            snake,
            input_dir: Direction::Right,
            food_pos,
            score: 1,
            width,
            height,
        }
    }

    fn reset(&mut self) {
        self.snake = Snake::new((self.width / 2, self.height / 2), Direction::Right);
        self.input_dir = Direction::Right;
        self.food_pos = generate_food_position(self.width, self.height, &self.snake.body).unwrap();
        self.score = 1;
    }

    fn handle_collision(&mut self) -> bool {
        if (0..self.width).contains(&self.snake_head().0)
            && (0..self.height).contains(&self.snake_head().1)
            && !(self.snake_body().contains(self.snake_head()))
        {
            if &self.food_pos == self.snake_head() {
                self.score += 1;
                self.snake.body.push(self.snake.new_segment_position);
                if let Some(pos) = generate_food_position(self.width, self.height, &self.snake.body)
                {
                    self.food_pos = pos;
                } else {
                    return true;
                }
            }
            return false;
        }
        true
    }

    fn advance(&mut self) {
        self.snake.advance();
    }

    fn change_direction(&mut self) {
        self.snake.change_direction(self.input_dir);
    }

    fn change_input_direction(&mut self, direction: Direction) {
        self.input_dir = direction;
    }

    fn snake_body(&self) -> &[Position] {
        &self.snake.body[1..]
    }

    fn snake_head(&self) -> &Position {
        &self.snake.body[0]
    }
}

#[derive(Debug, Clone)]
struct Snake {
    body: Vec<Position>,
    direction: Direction,
    new_segment_position: Position,
}

impl Snake {
    fn new(position: Position, direction: Direction) -> Self {
        Self {
            body: Vec::from([position]),
            direction,
            new_segment_position: match direction {
                Direction::Up => (position.0, position.1 + 1),
                Direction::Down => (position.0, position.1 - 1),
                Direction::Left => (position.0 + 1, position.1),
                Direction::Right => (position.0 - 1, position.1),
            },
        }
    }

    fn advance(&mut self) {
        let length = self.body.len();
        self.new_segment_position = self.body[length - 1];
        for i in (1..length).rev() {
            self.body[i] = self.body[i - 1]
        }
        match self.direction {
            Direction::Right => self.body[0].0 += 1,
            Direction::Left => self.body[0].0 -= 1,
            Direction::Up => self.body[0].1 -= 1,
            Direction::Down => self.body[0].1 += 1,
        };
    }

    fn change_direction(&mut self, direction: Direction) {
        match (self.direction, direction) {
            (Direction::Up, Direction::Down) => {}
            (Direction::Down, Direction::Up) => {}
            (Direction::Left, Direction::Right) => {}
            (Direction::Right, Direction::Left) => {}
            _ => self.direction = direction,
        }
    }
}
