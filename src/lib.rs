#![no_std]

use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{plot, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH, plot_str, plot_num, clear_row};
    use core::option::Option::Some;
use core::{
    clone::Clone,
    cmp::{Eq, PartialEq},
    iter::Iterator,
    marker::Copy,
    prelude::rust_2024::derive,
};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::RngCore;


const UPDATE_FREQUENCY: usize = 1;
const GAME_HEIGHT: usize = BUFFER_HEIGHT - 2;
const HEADER_SPACE: usize = BUFFER_HEIGHT - GAME_HEIGHT;
const ARRAY_SIZE: usize = GAME_HEIGHT * BUFFER_WIDTH;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SnakeGame<const WIDTH: usize, const HEIGHT: usize> {
    cells: [[Cell; WIDTH]; HEIGHT],
    snake: Snake<WIDTH,HEIGHT>,
    status: Status,
    last_key: Option<Dir>,
    countdown: usize,
    total_ticks: usize
}

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
#[repr(u8)]
pub enum Dir {
    N, S, E, W
}

impl Dir {
    fn icon(&self) -> char {
        match self {
            Dir::N => '^',
            Dir::S => 'v',
            Dir::E => '>',
            Dir::W => '<'
        }
    }
    fn reverse(&self) -> Dir {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::W,
            Dir::W => Dir::E
        }
    }
}

impl From<char> for Dir {
    fn from(icon: char) -> Self {
        match icon {
            'v' => Dir::S,
            '^' => Dir::N,
            '<' => Dir::W,
            '>' => Dir::E,
            _ => panic!("Illegal icon: '{}'", icon)
        }
    }
}

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
#[repr(u8)]
pub enum Cell {
    Empty,
    Wall,
    Snake,
    Food,
    Body
}

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
pub struct Position<const WIDTH: usize, const HEIGHT: usize> {
    col: i16, row: i16
}

impl <const WIDTH: usize, const HEIGHT: usize> Position<WIDTH,HEIGHT> {
    pub fn is_legal(&self) -> bool {
        0 <= self.col && self.col < WIDTH as i16 && 0 <= self.row && self.row < HEIGHT as i16
    }

    pub fn row_col(&self) -> (usize, usize) {
        (self.row as usize, self.col as usize)
    }

    pub fn neighbor(&self, d: Dir) -> Position<WIDTH,HEIGHT> {
        match d {
            Dir::N => Position {row: self.row - 1, col: self.col},
            Dir::S => Position {row: self.row + 1, col: self.col},
            Dir::E => Position {row: self.row,     col: self.col + 1},
            Dir::W => Position {row: self.row,     col: self.col - 1}
        }
    }
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
struct Snake<const WIDTH: usize, const HEIGHT: usize> {
    pos: Position<WIDTH,HEIGHT>, dir: Dir, size: usize, 
    body: [Position<WIDTH,HEIGHT>; ARRAY_SIZE], insert_index: usize, 
    remove_index: usize
}

impl <const WIDTH: usize, const HEIGHT: usize> Snake<WIDTH,HEIGHT> {
    fn new(pos: Position<WIDTH,HEIGHT>, icon: char) -> Self {
        Snake {pos, dir: Dir::from(icon), size: 0, body: [Position { col: 0, row: 0}; ARRAY_SIZE], insert_index: 0, remove_index: 0}
    }

    fn icon(&self) -> char {
        self.dir.icon()
    }
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum Status {
    Normal,
    Over,
    Start
}

const START: &'static str =
    "################################################################################
     #                                                                              #
     #                                                                              #
     #     >                                                                        #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                        @     #
     #                                                                              #
     #                                                                              #
     ################################################################################";


pub type MainGame = SnakeGame<BUFFER_WIDTH,GAME_HEIGHT>;
 
impl <const WIDTH: usize, const HEIGHT: usize> SnakeGame<WIDTH, HEIGHT> {
    pub fn new() -> Self {
        let mut game = SnakeGame {
            cells: [[Cell::Empty; WIDTH]; HEIGHT],
            snake: Snake::new(Position { col: 0, row: 0}, '>'),
            last_key: None,
            status: Status::Normal,
            countdown: UPDATE_FREQUENCY,
            total_ticks: 0
        };
        game.reset();
        game.status = Status::Over;
        game
    }

    pub fn tick(&mut self) {
        self.total_ticks +=1;
        if self.total_ticks == usize::MAX {
            self.total_ticks = 0;
        }
        if self.countdown_complete() {
            self.update();
            self.draw();
        }
    }
    
    fn draw(&mut self) {
        self.draw_header();
        self.draw_board();
    }
    
    fn draw_header(&mut self) {
        match self.status() {
            Status::Normal => self.draw_normal_header(),
            Status::Over => self.draw_game_over_header(),
            Status::Start => self.draw_start_header()
        }
    }

    fn draw_start_header(&mut self) {
        let header_color = ColorCode::new(Color::White, Color::Green);
        let score_text = "Welcome to snake!";
        plot_str(score_text, 0, 0, header_color);
        self.draw_subheader("Press 1 to start.");
    }
    
    fn draw_normal_header(&mut self) {
        let header_color = ColorCode::new(Color::White, Color::Green);
        let score_text = "Score:";
        clear_row(0, Color::Green);
        clear_row(1, Color::Green);
        plot_str(score_text, 0, 0, header_color);
        plot_num(self.score() as isize, score_text.len() + 1, 0, header_color);
    }
    
    fn draw_subheader(&self, subheader: &str) {
        plot_str(subheader, 0, 1, ColorCode::new(Color::Yellow, Color::Green));
    }
    
    fn draw_game_over_header(&mut self) {
        self.draw_normal_header();
        self.draw_subheader("Game over. Press 1 to restart.");
    }
    
    fn draw_board(&mut self) {
        for p in self.cell_pos_iter() {
            let (row, col) = p.row_col();
            let (c, color) = self.get_icon_color(p, &self.cell(p));
            plot(c, col, row + HEADER_SPACE, color);
        }
    }
    
    fn get_icon_color(&mut self, p: Position<WIDTH,HEIGHT>, cell: &Cell) -> (char, ColorCode) {
        let (icon, foreground) =
            if p == self.snake_at() {
                (match self.status() {
                    Status::Over => 'X',
                    _ => self.snake_icon()
                }, Color::Blue)
            } else {
                match cell {
                Cell::Body => ('o', Color::Blue),
                Cell::Empty => (' ', Color::Black),
                Cell::Wall => ('#', Color::Brown),
                Cell::Food => ('@', Color::Red),
                Cell::Snake => (self.snake_icon(), Color::Blue)
                }
            };
        (icon, ColorCode::new(foreground, Color::Green))
    }

    fn reset(&mut self) {
        for (row, row_chars) in START.split('\n').enumerate() {
            for (col, icon) in row_chars.trim().chars().enumerate() {
                self.translate_icon(row, col, icon);
            }
        }
        self.status = Status::Normal;
        self.last_key = None;
    }

    pub fn score(&self) -> usize {
        self.snake.size
    }

    fn translate_icon(&mut self, row: usize, col: usize, icon: char) {
        match icon {
            '#' => self.cells[row][col] = Cell::Wall,
            ' ' => self.cells[row][col] = Cell::Empty,
            'o' => self.cells[row][col] = Cell::Body,
            '@' => self.cells[row][col] = Cell::Food,
            '>' |'<' | '^' | 'v' => {
                self.snake = Snake::new(Position {row: row as i16, col: col as i16}, icon);
            },
            _ =>  panic!("Unrecognized character: '{}'", icon)
        }
    }

    pub fn cell(&self, p: Position<WIDTH,HEIGHT>) -> Cell {
        self.cells[p.row as usize][p.col as usize]
    }

    pub fn cell_pos_iter(&self) -> RowColIter<WIDTH,HEIGHT> {
        RowColIter { row: 0, col: 0 }
    }

    pub fn snake_at(&self) -> Position<WIDTH,HEIGHT> {
        self.snake.pos
    }

    pub fn snake_icon(&self) -> char {
        self.snake.icon()
    }

    pub fn update(&mut self) {
        self.resolve_move();
        self.last_key = None;
    }

    pub fn key(&mut self, key: DecodedKey) {
        match self.status {
            Status::Over => {
                match key {
                    DecodedKey::RawKey(KeyCode::Key1) | DecodedKey::Unicode('1') => self.reset(),
                    _ => {}
                }
            }
            _ => {
                let key = key2dir(key);
                if key.is_some() {
                    self.last_key = key;
                }
            }
        }
    }

    pub fn countdown_complete(&mut self) -> bool {
        if self.countdown == 0 {
            self.countdown = UPDATE_FREQUENCY;
            true
        } else {
            self.countdown -= 1;
            false
        }
    }

    fn resolve_move(&mut self) {
        if let Some(dir) = self.last_key {
            if dir != self.snake.dir.reverse() {
                self.snake.dir = dir;
            }
        }
        let dir = self.snake.dir;
        let neighbor = self.snake.pos.neighbor(dir);
        if neighbor.is_legal() {
            let (row, col) = neighbor.row_col();
            if (self.cells[row][col] == Cell::Body) | (self.cells[row][col] == Cell::Wall) | (self.status == Status::Over) {
                self.status = Status::Over
            }
            else {
                self.move_to(neighbor, dir);
            }
        }
    }

    fn update_snake_body(&mut self, new_body: Position<WIDTH,HEIGHT>, grow:bool) {
        self.snake.body[self.snake.insert_index] = new_body;
        self.snake.insert_index += 1;
        if self.snake.insert_index == ARRAY_SIZE {
            self.snake.insert_index = 0;
        }
        if !grow {
            let cleared_pos = self.snake.body[self.snake.remove_index];
            self.snake.remove_index += 1;
            if self.snake.remove_index == ARRAY_SIZE {
                self.snake.remove_index = 0;
            }
            self.cells[cleared_pos.row as usize][cleared_pos.col as usize] = Cell::Empty;
        }
    }

    fn move_to(&mut self, neighbor: Position<WIDTH,HEIGHT>, dir: Dir) {
        let curr_pos = self.snake.pos;
        self.cells[curr_pos.row as usize][curr_pos.col as usize] = Cell::Body;
        self.snake.pos = neighbor;
        self.snake.dir = dir;
        let (row, col) = neighbor.row_col();
        match self.cells[row][col] {
            Cell::Food => {
                self.cells[row][col] = Cell::Empty;
                self.snake.size += 1;
                self.new_food();
                self.update_snake_body(curr_pos, true);
            }
            _ => {self.update_snake_body(curr_pos, false);}
        }
    }

    fn new_food(&mut self) {
        let mut small_rng = SmallRng::seed_from_u64(self.total_ticks as u64); // https://stackoverflow.com/questions/67627335/how-do-i-use-the-rand-crate-without-the-standard-library
        let mut row = ((small_rng.next_u32() as f64) / 4294967296.0 * ((HEIGHT-3)as f64) + (1 as f64)) as usize;
        let mut col = ((small_rng.next_u32() as f64) / 4294967296.0 * ((WIDTH-3)as f64) + (1 as f64)) as usize;
        while !(self.cells[row][col] == Cell::Empty) {
            row = ((small_rng.next_u32() as f64) / 4294967296.0 * ((HEIGHT-3)as f64) + (1 as f64)) as usize;
            col = ((small_rng.next_u32() as f64) / 4294967296.0 * ((WIDTH-3)as f64) + (1 as f64)) as usize;
        }
        self.cells[row][col] = Cell::Food;
    }

    pub fn status(&self) -> Status {
        self.status
    }

}

pub struct RowColIter<const WIDTH: usize, const HEIGHT: usize> {
    row: usize, col: usize
}

impl <const WIDTH: usize, const HEIGHT: usize> Iterator for RowColIter<WIDTH,HEIGHT> {
    type Item = Position<WIDTH,HEIGHT>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row < HEIGHT {
            let result = Some(Position {row: self.row as i16, col: self.col as i16});
            self.col += 1;
            if self.col == WIDTH {
                self.col = 0;
                self.row += 1;
            }
            result
        } else {
            None
        }
    }
}

fn key2dir(key: DecodedKey) -> Option<Dir> {
    match key {
        DecodedKey::RawKey(k) => match k {
            KeyCode::ArrowUp => Some(Dir::N),
            KeyCode::ArrowDown => Some(Dir::S),
            KeyCode::ArrowLeft => Some(Dir::W),
            KeyCode::ArrowRight => Some(Dir::E),
            _ => None
        }
        DecodedKey::Unicode(c) => match c {
            'w' => Some(Dir::N),
            'a' => Some(Dir::W),
            's' => Some(Dir::S),
            'd' => Some(Dir::E),
            _ => None
        }
    }
}