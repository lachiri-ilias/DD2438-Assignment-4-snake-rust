use crate::Coord;
use crate::{Battlesnake, Board, Game};
use log::info;
use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fmt;
use std::time::Instant;

const TIMEOUT: u128 = 450;

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
pub fn get_move(game: &Game, turn: &i32, board: &Board, you: &Battlesnake) -> Value {
    let start_time = Instant::now();
    let mut best_move = Move::Up;
    let mut best_score = std::i32::MIN;

    let mut depth = 3;

    (best_move, best_score) = mini_max(board, you, depth, i32::MIN, i32::MAX, start_time, 0);
    info!("best move: {}, with score: {}", best_move, best_score);
    json!({ "move": best_move })
}

fn mini_max(
    board: &Board,
    you: &Battlesnake,
    depth: usize,
    mut alpha: i32,
    mut beta: i32,
    start_time: Instant,
    mut index: usize,
) -> (Move, i32) {
    if depth == 0 {
        return (Move::Up, evaluate_board(board, you));
    }

    let all_snakes = &board.snakes;
    index = index % all_snakes.len();
    let curr_snake = &all_snakes[index];
    info!("----> depth = {}, snake index = {}", depth, index);
    print_board(board, &board.snakes[index]);
    let maximizing = curr_snake.id == you.id;

    let possible_moves = get_possible_moves(board, curr_snake);

    let mut best_move = Move::Up;
    let mut best_score = if maximizing { i32::MIN } else { i32::MAX };

    for &dir in &possible_moves {
        let mut new_board = board.clone();

        simulate_move(&mut new_board, index, &dir);

        let (_, score) = mini_max(
            &new_board,
            you,
            depth - 1,
            alpha,
            beta,
            start_time,
            index + 1,
        );

        if maximizing && score > best_score {
            best_score = score;
            best_move = dir;
        } else if !maximizing && score < best_score {
            best_score = score;
            best_move = dir;
        }

        if maximizing {
            alpha = score.max(alpha);
        } else {
            beta = score.min(beta);
        }

        if alpha >= beta {
            break;
        }
    }

    (best_move, best_score)
}

fn simulate_move(mut board: &mut Board, snake_index: usize, action: &Move) {
    let (dx, dy) = match action {
        Move::Up => (0, 1),
        Move::Down => (0, -1),
        Move::Left => (-1, 0),
        Move::Right => (1, 0),
    };

    let mut new_head = board.snakes[snake_index].head;
    new_head.x += dx;
    new_head.y += dy;

    board.snakes[snake_index].body.insert(0, new_head);
    board.snakes[snake_index].head = new_head;

    struct CollisionOutcome {
        snake_dies: bool,
        other_snake_dies: Option<usize>,
        head_to_head: bool,
    }

    // Initialize the outcome with no collisions
    let mut outcome = CollisionOutcome {
        snake_dies: false,
        other_snake_dies: None,
        head_to_head: false,
    };

    // Collision detection phase
    // check for any collisions with snakes only (walls not needed because get_possible_moves already handles that)
    for (id, other_snake) in board.snakes.iter().enumerate() {
        if id == snake_index {
            continue;
        }

        for (i, &coord) in other_snake.body.iter().enumerate() {
            if coord == board.snakes[snake_index].head {
                if i == 0 {
                    outcome.head_to_head = true;
                    // Head to head collision
                    if board.snakes[snake_index].body.len() >= other_snake.body.len() {
                        outcome.other_snake_dies = Some(id);
                    } else {
                        outcome.snake_dies = true;
                    }
                } else {
                    // Head to body collision
                    outcome.snake_dies = true;
                }
                break;
            }
        }

        if outcome.snake_dies || outcome.other_snake_dies.is_some() {
            break;
        }
    }

    if outcome.snake_dies {
        // Handle the current snake's death
        // ...
        board.snakes[snake_index].health = 0;
        board.snakes[snake_index].body.clear();
        board.snakes[snake_index].length = 0;
        board.snakes[snake_index].head = Coord { x: -1, y: -1 };
        if outcome.head_to_head {
            let other_id = outcome.other_snake_dies.unwrap();
            board.snakes[other_id].length -=
                (board.snakes[other_id].body.len() - board.snakes[snake_index].body.len()) as i32;
            for _ in 0..(board.snakes[other_id].body.len() - board.snakes[snake_index].body.len()) {
                board.snakes[other_id].body.pop();
            }
        }
    }
    if let Some(other_id) = outcome.other_snake_dies {
        // Handle the other snake's death
        // ...
        board.snakes[snake_index].length -=
            (board.snakes[snake_index].body.len() - board.snakes[other_id].body.len()) as i32;
        for _ in 0..(board.snakes[snake_index].body.len() - board.snakes[other_id].body.len()) {
            board.snakes[snake_index].body.pop();
        }
        // remove the dead snake from the board
        board.snakes[other_id].health = 0;
        board.snakes[other_id].body.clear();
        board.snakes[other_id].length = 0;
        board.snakes[other_id].head = Coord { x: -1, y: -1 };
    }

    // check for food
    if board.food.contains(&board.snakes[snake_index].head) {
        // then we don't remove the tail and reset the health
        board.snakes[snake_index].health = 100;
        board.snakes[snake_index].length += 1;
        // remove the food from the board
        let food_index = board
            .food
            .iter()
            .position(|&coord| coord == board.snakes[snake_index].head);
        board.food.remove(food_index.unwrap());
    } else {
        board.snakes[snake_index].health -= 1;
        board.snakes[snake_index].body.pop();
    }
    // TODO: check for hazards
}

pub fn get_possible_moves(board: &Board, you: &Battlesnake) -> Vec<Move> {
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

    let board_width = board.width as i32;
    let board_height = board.height as i32;

    if my_head.x == 0 {
        possible_moves.retain(|m| m != &Move::Left)
    }
    if my_head.x == board_width - 1 {
        possible_moves.retain(|m| m != &Move::Right)
    }
    if my_head.y == 0 {
        possible_moves.retain(|m| m != &Move::Down)
    }
    if my_head.y == board_height - 1 {
        possible_moves.retain(|m| m != &Move::Up)
    }

    possible_moves
}

// "you" might not be needed here
pub fn score_node(board: &Board, you: &Battlesnake) -> i32 {
    let mut score = 0;

    // Health: Higher health increases the score
    score += you.health;

    // Length: Longer snakes have an advantage
    score += you.body.len() as i32 * 10;

    // Last snake standing: Huge bonus for being the only snake left
    if board.snakes.len() == 1 && board.snakes[0].id == you.id {
        score += 1000;
    }

    // Proximity to food: Closer to food is generally better
    if let Some(closest_food) = board
        .food
        .iter()
        .min_by_key(|food| (food.x - you.body[0].x).abs() + (food.y - you.body[0].y).abs())
    {
        let distance_to_food =
            (closest_food.x - you.body[0].x).abs() + (closest_food.y - you.body[0].y).abs();
        // Invert the distance to make closer food give a higher score
        score += 100 / (1 + distance_to_food);
    }

    // You can add more heuristics here to refine how the score is calculated

    score
}

fn evaluate_board(board: &Board, you: &Battlesnake) -> i32 {
    let head = &you.body[0];
    let mut best_food_score = std::i32::MIN;
    // let health_factor = if you.health < 50 { 20.0 } else { 10.0 }; // Adjust the factor for more responsiveness
    if you.health > 90 {
        best_food_score = i32::MAX;
    }
    for food in &board.food {
        let food_distance = (food.x - head.x).abs() + (food.y - head.y).abs();
        let food_score = -food_distance as f32;

        // Safety check for the path to the food
        best_food_score = best_food_score.max(food_score as i32);
    }

    // Additional scoring for free space availability
    // let free_spaces = calculate_free_space_around_head(board, you);
    // if free_spaces < 2 {
    //     // Dangerously low free space might indicate a potential trap
    //     best_food_score /= 2; // Penalize potentially risky food positions
    // }

    best_food_score
}

fn calculate_free_space_around_head(board: &Board, you: &Battlesnake) -> i32 {
    let head = you.body.first().unwrap();
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    let mut free_spaces = 0;

    for (dx, dy) in directions.iter() {
        let new_x = head.x + dx;
        let new_y = head.y + dy;
        if new_x >= 0 && new_x < board.width as i32 && new_y >= 0 && new_y < board.height as i32 {
            if !board
                .snakes
                .iter()
                .any(|s| s.body.iter().any(|pos| pos.x == new_x && pos.y == new_y))
            {
                free_spaces += 1;
            }
        }
    }

    free_spaces
}
pub fn print_board(board: &Board, you: &Battlesnake) {
    let snakes = &board.snakes;
    let snake_chars = ('A'..='Z').collect::<Vec<_>>();

    for (index, snake) in snakes.iter().enumerate() {
        let snake_char = snake_chars.get(index).unwrap_or(&'?');
        println!("Snake {}: {}", snake_char, snake.health);
    }

    let snake_coords: Vec<_> = snakes
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
