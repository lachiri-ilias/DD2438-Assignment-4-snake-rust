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
pub fn get_move(_game: &Game, turn: &i32, _board: &Board, you: &Battlesnake) -> Value {
    let start_time = Instant::now();
    let mut best_move = Move::Up;
    let mut best_score = std::f64::MIN;

    for depth in 1..=4 {
        let (m, score) = mini_max(
            _game,
            _board,
            you,
            depth,
            std::f64::MIN,
            std::f64::MAX,
            start_time,
            0, // TODO: make sure that the first snake in the array is our snake, so that we start with our snake
        );

        if score > best_score {
            best_move = m;
            best_score = score;
        }

        if start_time.elapsed().as_millis() >= TIMEOUT {
            break; // Exit loop if time is up
        }
    }

    json!({ "move": best_move })
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

fn mini_max(
    game: &Game,
    board: &Board,
    you: &Battlesnake,
    depth: usize,
    mut alpha: f64,
    mut beta: f64,
    start_time: Instant,
    index: usize,
) -> (Move, f64) {
    if depth == 0 || is_terminal_state(board, you) {
        return (Move::Up, score_node(board, you) as f64);
    }

    let snakes = &board.snakes; // All snakes in the game
    let current_snake = &snakes[index % snakes.len()]; // Get the snake at the current index

    let mut best_move = Move::Up;
    let mut best_score = if current_snake.id == you.id {
        std::f64::MIN
    } else {
        std::f64::MAX
    };

    let possible_moves = get_possible_moves(you);

    for &mov in &possible_moves {
        let new_board_state = simulate_move(board.clone(), index, mov);

        let elapsed_ms = start_time.elapsed().as_millis();
        if elapsed_ms >= TIMEOUT {
            break; // Exit loop if time is up
        }

        let (_, score) = mini_max(
            &game,
            &new_board_state,
            &you,
            depth - 1,
            alpha,
            beta,
            start_time,
            index + 1,
        );

        if current_snake.id == you.id {
            // Maximize if it's our snake's turn
            if score > best_score {
                best_score = score;
                best_move = mov;
                alpha = score.max(alpha);
            }
        } else {
            // Minimize for enemy snakes' turns
            if score < best_score {
                best_score = score;
                best_move = mov;
                beta = score.min(beta);
            }
        }

        if alpha >= beta {
            break; // Beta cutoff
        }
    }

    (best_move, best_score) // Return best_score
}

fn simulate_move(board: Board, snake_index: usize, action: Move) -> Board {
    let mut new_board = board.clone(); // Create a copy of the current board state

    // Update the position of the snake corresponding to the given index based on the chosen action
    let mut new_snakes = new_board.snakes.clone();
    let head = new_snakes[snake_index].body[0]; // Current head position

    let new_head = match action {
        Move::Up => Coord {
            x: head.x,
            y: head.y + 1,
        },
        Move::Down => Coord {
            x: head.x,
            y: head.y - 1,
        },
        Move::Left => Coord {
            x: head.x - 1,
            y: head.y,
        },
        Move::Right => Coord {
            x: head.x + 1,
            y: head.y,
        },
    };

    // Check for collisions with walls
    let mut collision = false;
    if new_head.x < 0
        || new_head.y < 0
        || new_head.x >= board.width
        || new_head.y >= board.height as i32
    {
        collision = true;
    } else {
        // Check for collisions with other snakes
        for other_snake in &new_snakes {
            for (i, &coord) in other_snake.body.iter().enumerate() {
                if coord == new_head {
                    if i == 0 {
                        // Head-to-head collision
                        collision = new_snakes[snake_index].length <= other_snake.length;
                    } else {
                        // Body collision
                        collision = true;
                    }
                    break;
                }
            }
            if collision {
                break;
            }
        }
    }

    // Apply collision result
    if collision {
        new_snakes[snake_index].health = 0;
        new_snakes[snake_index].body.clear();
    } else {
        // Move the snake's head to the new position
        new_snakes[snake_index].body.insert(0, new_head);

        // Check if the snake consumes food
        if board.food.contains(&new_head) {
            new_snakes[snake_index].health =
                std::cmp::min(new_snakes[snake_index].health + 10, 100);
            let food_index = new_board.food.iter().position(|&coord| coord == new_head);
            if let Some(index) = food_index {
                new_board.food.remove(index); // Remove consumed food from the board
            }
        }

        // Check if the snake hits a hazard
        if board.hazards.contains(&new_head) {
            new_snakes[snake_index].health = std::cmp::max(new_snakes[snake_index].health - 10, 0);
        }

        // Remove the tail segment if the snake didn't consume food
        if new_snakes[snake_index].health > 0 && !board.food.contains(&new_head) {
            new_snakes[snake_index].body.pop();
        }
    }

    new_board.snakes = new_snakes;
    new_board
}

fn is_terminal_state(board: &Board, you: &Battlesnake) -> bool {
    // Check if our snake is dead
    if you.health == 0 {
        return true;
    }

    // Check if there is only one snake left (ours)
    if board.snakes.len() == 1 && board.snakes[0].id == you.id {
        return true;
    }

    // Check if the game has ended due to all snakes being dead
    if board.snakes.iter().all(|snake| snake.health == 0) {
        return true;
    }

    // Additional terminal conditions can be added here

    false // If none of the conditions are met, the game is not in a terminal state
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
