use crate::Coord;
use crate::{Battlesnake, Board, Game};
use chrono::{DateTime, Duration, Local};
use log::info;
use rand::seq::SliceRandom;
use rocket::{
    time::Time,
    tokio::time::{self, Timeout},
};
use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};
use std::fmt;
const TIMEOUT: Duration = Duration::milliseconds(400);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::Left => write!(f, "Left"),
            Move::Right => write!(f, "Right"),
            Move::Up => write!(f, "Up"),
            Move::Down => write!(f, "Down"),
        }
    }
}

impl Serialize for Move {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let variant_str = match self {
            Move::Left => "left",
            Move::Down => "down",
            Move::Up => "up",
            Move::Right => "right",
        };
        serializer.serialize_str(variant_str)
    }
}

struct BoardChanges {
    removed_snakes: VecDeque<Battlesnake>,
    removed_food: HashSet<Coord>,
    snake_health_changes: Vec<(usize, i32)>,
}

impl BoardChanges {
    fn new() -> Self {
        BoardChanges {
            removed_snakes: VecDeque::new(),
            removed_food: HashSet::new(),
            snake_health_changes: Vec::new(),
        }
    }

    fn add_removed_snake(&mut self, snake: Battlesnake) {
        self.removed_snakes.push_front(snake);
    }

    fn add_removed_food(&mut self, food: Coord) {
        self.removed_food.insert(food);
    }

    fn add_health_change(&mut self, snake_idx: usize, health_change: i32) {
        self.snake_health_changes.push((snake_idx, health_change));
    }

    fn revert(&mut self, board: &mut Board, snakes: &mut Vec<Battlesnake>) {
        // Restore removed snakes
        while let Some(snake) = self.removed_snakes.pop_back() {
            snakes.push(snake);
        }

        // Restore removed food
        for food in &self.removed_food {
            board.food.push(*food);
        }
        self.removed_food.clear();

        // Restore snake health changes
        for (idx, health_change) in &self.snake_health_changes {
            snakes[*idx].health -= health_change;
        }
        self.snake_health_changes.clear();
    }
}

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "si", // TODO: Your Battlesnake Username
        "color": "#888888", // TODO: Choose color
        "head": "default", // TODO: Choose head
        "tail": "default", // TODO: Choose tail
    });
}

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
pub fn get_move(_game: &Game, turn: &i32, _board: &Board, you: &Battlesnake) -> Value {
    let possible_moves = get_possible_moves(you);

    // Choose a random move from the safe ones
    let chosen = possible_moves.choose(&mut rand::thread_rng()).unwrap();

    let start = Local::now();
    let mut depth = 0;
    let mut chosen_move = Move::Up;
    let mut alpha = std::f64::MIN;
    let mut beta = std::f64::MAX;
    let mut timeout = false;

    // while !timeout {}
    info!("MOVE {}: {}", turn, chosen);
    print_board(_board, you);
    return json!({ "move": chosen });
}

pub fn get_possible_moves(you: &Battlesnake) -> Vec<Move> {
    let mut possible_moves = vec![Move::Up, Move::Down, Move::Left, Move::Right];

    let my_head = &you.body[0]; // Coordinates of your head
    let my_neck = &you.body[1];

    if my_neck.x < my_head.x {
        // Neck is left of head, don't move left
        possible_moves.retain(|m| m != &Move::Left)
    } else if my_neck.x > my_head.x {
        // Neck is right of head, don't move right
        possible_moves.retain(|m| m != &Move::Right)
    } else if my_neck.y < my_head.y {
        // Neck is below head, don't move down
        possible_moves.retain(|m| m != &Move::Down)
    } else if my_neck.y > my_head.y {
        // Neck is above head, don't move up
        possible_moves.retain(|m| m != &Move::Up)
    }

    possible_moves
}

// "you" might not be needed here
pub fn score_node(game: &Game, board: &Board, you: &Battlesnake) -> usize {
    0
}

pub fn mini_max(
    game: &Game,
    board: &Board,
    you: &Battlesnake,
    depth: usize,
    alpha: f64,
    beta: f64,
    start_time: DateTime<Local>,
) -> Option<(Move, usize)> {
    let curr_time = Local::now();

    if (curr_time - start_time >= TIMEOUT) {}
    Some((Move::Up, 0))
}

pub fn simulate_move(
    board: &mut Board,
    snakes: &mut Vec<Battlesnake>,
    snake_idx: usize,
    move_dir: &Move,
    changes: &mut BoardChanges,
) {
}

pub fn undo_move(board: &mut Board, snakes: &mut Vec<Battlesnake>, changes: &mut BoardChanges) {
    changes.revert(board, snakes);
}

pub fn print_board(board: &Board, you: &Battlesnake) {
    let snakes = &board.snakes;
    let mut snake_chars = ('A'..='Z').collect::<Vec<_>>();
    let mut snake_coords: Vec<_> = snakes
        .iter()
        .map(|snake| {
            let coords = snake.body.iter().cloned().collect::<HashSet<_>>();
            (snake.id.clone(), coords)
        })
        .collect();

    let player_coords = you.body.iter().cloned().collect::<HashSet<_>>();

    for y in (0..board.height).rev() {
        for x in 0..board.width {
            let coord = crate::Coord { x: x, y: y as i32 };
            let cell = if board.food.contains(&coord) {
                "F".to_string()
            } else if board.hazards.contains(&coord) {
                "-".to_string()
            } else if player_coords.contains(&coord) {
                "S".to_string()
            } else if let Some(snake_char) = snake_coords
                .iter()
                .find(|(_, coords)| coords.contains(&coord))
                .map(|(id, _)| {
                    let idx = snakes.iter().position(|s| s.id == *id).unwrap();
                    let char = snake_chars[idx];
                    (char.to_string()).repeat(1)
                })
            {
                snake_char
            } else {
                "#".to_string()
            };
            print!("{} ", cell);
        }
        println!();
    }
}
