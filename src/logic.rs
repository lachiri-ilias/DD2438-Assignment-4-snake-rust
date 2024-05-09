// Welcome to
// __________         __    __  .__                               __
// \______   \_____ _/  |__/  |_|  |   ____   ______ ____ _____  |  | __ ____
//  |    |  _/\__  \\   __\   __\  | _/ __ \ /  ___//    \\__  \ |  |/ // __ \
//  |    |   \ / __ \|  |  |  | |  |_\  ___/ \___ \|   |  \/ __ \|    <\  ___/
//  |________/(______/__|  |__| |____/\_____>______>___|__(______/__|__\\_____>
//
// This file can be a nice home for your Battlesnake logic and helper functions.
//
// To get you started we've included code to prevent your Battlesnake from moving backwards.
// For more info see docs.battlesnake.com

use log::info;
use rand::seq::SliceRandom;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{Battlesnake, Board, Coord, Game, GameState};

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "Ilias_Saad", // TODO: Your Battlesnake Username
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
pub fn get_move(_game: &Game, turn: &i32, board: &Board, you: &Battlesnake) -> Value {
    let mut is_move_safe: HashMap<_, _> = vec![
        ("up", true),
        ("down", true),
        ("left", true),
        ("right", true),
    ]
    .into_iter()
    .collect();

    // We've included code to prevent your Battlesnake from moving backwards
    let my_head = &you.body[0]; // Coordinates of your head
    let my_neck = &you.body[1]; // Coordinates of your "neck"  // let my_neck = you.body.get(1);

    if my_neck.x < my_head.x {
        // Neck is left of head, don't move left
        is_move_safe.insert("left", false);
    } else if my_neck.x > my_head.x {
        // Neck is right of head, don't move right
        is_move_safe.insert("right", false);
    } else if my_neck.y < my_head.y {
        // Neck is below head, don't move down
        is_move_safe.insert("down", false);
    } else if my_neck.y > my_head.y {
        // Neck is above head, don't move up
        is_move_safe.insert("up", false);
    }

    // TODO: Step 1 - Prevent your Battlesnake from moving out of bounds
    let board_width = board.width as i32;
    let board_height = board.height as i32;

    if my_head.x == 0 {
        is_move_safe.insert("left", false);
    }
    if my_head.x == board_width - 1 {
        is_move_safe.insert("right", false);
    }
    if my_head.y == 0 {
        is_move_safe.insert("down", false);
    }
    if my_head.y == board_height - 1 {
        is_move_safe.insert("up", false);
    }

    // TODO: Step 2 - Prevent your Battlesnake from colliding with itself
    let my_body = &you.body;

    // Check each possible move to see if it would collide with your own body
    for &(dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let new_x = my_head.x + dx;
        let new_y = my_head.y + dy;

        if my_body
            .iter()
            .any(|segment| segment.x == new_x && segment.y == new_y)
        {
            if dx == -1 {
                is_move_safe.insert("left", false);
            } else if dx == 1 {
                is_move_safe.insert("right", false);
            } else if dy == -1 {
                is_move_safe.insert("down", false);
            } else if dy == 1 {
                is_move_safe.insert("up", false);
            }
        }
    }

    // TODO: Step 3 - Prevent your Battlesnake from colliding with other Battlesnakes
    let opponents = &board.snakes;

    for opponent in opponents {
        for opponent_segment in &opponent.body {
            for &(dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let new_x = my_head.x + dx;
                let new_y = my_head.y + dy;

                if opponent_segment.x == new_x && opponent_segment.y == new_y {
                    if dx == -1 {
                        is_move_safe.insert("left", false);
                    } else if dx == 1 {
                        is_move_safe.insert("right", false);
                    } else if dy == -1 {
                        is_move_safe.insert("down", false);
                    } else if dy == 1 {
                        is_move_safe.insert("up", false);
                    }
                }
            }
        }
    }

    // TODO: Step 4 - Move towards food instead of random, to regain health and survive longer

    let food = &board.food; // Get a reference to the food positions on the board

    let closest_food = food
        .iter()
        .min_by_key(|item| (item.x - my_head.x).abs() + (item.y - my_head.y).abs());
    if let Some(food) = closest_food {
        let mut min_distance = std::i32::MAX;
        let mut chosen = "up"; // Default direction

        for &(dx, dy, dir) in &[
            (-1, 0, "left"),
            (1, 0, "right"),
            (0, -1, "down"),
            (0, 1, "up"),
        ] {
            let new_x = my_head.x + dx;
            let new_y = my_head.y + dy;
            let distance = (food.x - new_x).abs() + (food.y - new_y).abs();

            if distance < min_distance && is_move_safe[dir] == true {
                min_distance = distance;
                chosen = dir;
            }
        }

        info!("MOVE {}: {}", turn, chosen);
        return json!({ "move": chosen });
    }

    // Are there any safe moves left?
    let safe_moves = is_move_safe
        .into_iter()
        .filter(|&(_, v)| v)
        .map(|(k, _)| k)
        .collect::<Vec<_>>();

    // Choose a random move from the safe ones
    // let chosen = safe_moves.choose(&mut rand::thread_rng()).unwrap();
    let chosen = safe_moves.choose(&mut rand::thread_rng()).unwrap_or(&"up"); // default to "up" if no safe move is available

    info!("MOVE {}: {}", turn, chosen);
    return json!({ "move": chosen });
}

// TODO: add alpha-beta pruning, add 500ms limit
fn minimax(game_state: &GameState, depth: usize, is_maximizing_player: bool) -> (String, i32) {
    // Base case: Check if the game is over or depth is 0
    if depth == 0 || is_terminal(game_state) {
        return (String::new(), evaluate(game_state));
    }

    // Recursive case: Iterate over possible moves, apply them, and call minimax recursively
    let possible_moves = get_possible_moves(game_state);
    let mut best_move = String::new();
    let mut best_score = if is_maximizing_player {
        i32::MIN
    } else {
        i32::MAX
    };

    for m in possible_moves {
        let new_state = apply_move(game_state, &m);
        let (_, score) = minimax(&new_state, depth - 1, !is_maximizing_player);

        if is_maximizing_player && score > best_score {
            best_score = score;
            best_move = m;
        } else if !is_maximizing_player && score < best_score {
            best_score = score;
            best_move = m;
        }
    }

    (best_move, best_score)
}

// TODO: more complex evaluate function
fn evaluate(game_state: &GameState) -> i32 {
    let head = &game_state.you.head;
    let health = game_state.you.health;
    let mut score = health / 10; // Basic health contribution to score

    // Positive influence for being close to food
    for food in &game_state.board.food {
        let dist = (head.x - food.x).abs() + (head.y - food.y).abs();
        if dist != 0 {
            // Avoid division by zero
            score += 100 / dist;
        }
    }

    score
}

fn is_terminal(game_state: &GameState) -> bool {
    let head = &game_state.you.head;
    // Check if snake has hit the walls
    if head.x < 0
        || head.x >= game_state.board.width
        || head.y < 0
        || head.y >= game_state.board.height as i32
    {
        return true;
    }
    // Check if snake has collided with itself
    for part in &game_state.you.body[1..] {
        // Skip the head
        if head.x == part.x && head.y == part.y {
            return true;
        }
    }

    false
}

fn get_possible_moves(game_state: &GameState) -> Vec<String> {
    let head = &game_state.you.head;
    let mut moves = vec!["up", "down", "left", "right"]
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>();

    // Prevent moves that would collide with the snake's own body
    if game_state.you.body.contains(&Coord {
        x: head.x,
        y: head.y - 1,
    }) {
        moves.retain(|m| m != "up");
    }
    if game_state.you.body.contains(&Coord {
        x: head.x,
        y: head.y + 1,
    }) {
        moves.retain(|m| m != "down");
    }
    if game_state.you.body.contains(&Coord {
        x: head.x - 1,
        y: head.y,
    }) {
        moves.retain(|m| m != "left");
    }
    if game_state.you.body.contains(&Coord {
        x: head.x + 1,
        y: head.y,
    }) {
        moves.retain(|m| m != "right");
    }

    moves
}

fn apply_move(game_state: &GameState, move_: &str) -> GameState {
    let mut new_state = game_state.clone();
    let head = &mut new_state.you.head;
    match move_ {
        "up" => head.y += 1,
        "down" => head.y -= 1,
        "left" => head.x -= 1,
        "right" => head.x += 1,
        _ => {}
    }

    new_state
}
