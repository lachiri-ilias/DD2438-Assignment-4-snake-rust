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

use crate::{Battlesnake, Board, Game};

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "Ilias_Saad", // TODO: Your Battlesnake Username
        "color": "#de1a24", // TODO: Choose color
        "head": "do-sammy", // TODO: Choose head
        "tail": "mystic-moon", // TODO: Choose tail
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


fn is_move_safe(board: &Board, you: &Battlesnake, direction: &str) -> bool {
    let head = you.body.first().unwrap();
    let mut new_x = head.x;
    let mut new_y = head.y;

    // Determine the new head position based on the given direction
    match direction {
        "up" => new_y += 1,
        "down" => new_y -= 1,
        "left" => new_x -= 1,
        "right" => new_x += 1,
        _ => return false, // Return false on invalid direction
    }

    // Check for out-of-bounds movement
    if new_x < 0 || new_x >= board.width as i32 || new_y < 0 || new_y >= board.height as i32 {
        return false;
    }

    // Check for collisions with itself
    if you.body.iter().any(|segment| segment.x == new_x && segment.y == new_y) {
        return false;
    }

    // Check for collisions with other snakes
    for snake in &board.snakes {
        if snake.body.iter().any(|segment| segment.x == new_x && segment.y == new_y) {
            return false;
        }
    }

    // Avoid head-to-head collisions unless we are longer
    for snake in &board.snakes {
        if snake.id != you.id && snake.body.first().map_or(false, |h| h.x == new_x && h.y == new_y) {
            if snake.body.len() >= you.body.len() {
                return false;
            }
        }
    }

    true
}
fn evaluate_board(board: &Board, you: &Battlesnake) -> i32 {
    let head = &you.body[0];
    let mut best_food_score = std::i32::MIN;
    let health_factor = if you.health < 50 { 20.0 } else { 10.0 }; // Adjust the factor for more responsiveness

    for food in &board.food {
        let food_distance = (food.x - head.x).abs() + (food.y - head.y).abs();
        let food_score = -food_distance as f32 * health_factor;
        
        // Safety check for the path to the food
            best_food_score = best_food_score.max(food_score as i32);
    }

    // Additional scoring for free space availability
    let free_spaces = calculate_free_space_around_head(board, you);
    if free_spaces < 2 { // Dangerously low free space might indicate a potential trap
        best_food_score /= 2; // Penalize potentially risky food positions
    }

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
            if !board.snakes.iter().any(|s| s.body.iter().any(|pos| pos.x == new_x && pos.y == new_y)) {
                free_spaces += 1;
            }
        }
    }

    free_spaces
}



// fn simulate_move(board: Board, you: Battlesnake, move_dir: &str) {
fn simulate_move(board: &mut Board, you: &mut Battlesnake, move_dir: &str) {
    let (dx, dy) = match move_dir {
        "up" => (0, 1),
        "down" => (0, -1),
        "left" => (-1, 0),
        "right" => (1, 0),
        _ => (0, 0),
    };

    let mut new_head = you.body[0].clone();
    new_head.x += dx;
    new_head.y += dy;

    // Check if the new head position is on a food
    if let Some(index) = board.food.iter().position(|f| f.x == new_head.x && f.y == new_head.y) {
        you.body.insert(0, new_head); // Add new head to the body
        board.food.remove(index); // Remove the food from the board
    } else {
        you.body.pop(); // Remove the last segment of the body
        you.body.insert(0, new_head); // Add new head to the body
    }
}



fn minimax(board: &Board, you: &Battlesnake, depth: i32, alpha: i32, beta: i32, is_maximizing: bool) -> (i32, String) {
    if depth == 0 {
        return (evaluate_board(board, you), String::from("none"));
    }

    let mut alpha = alpha;
    let mut beta = beta;
    let mut best_move = String::from("none");
    let mut best_score = if is_maximizing { std::i32::MIN } else { std::i32::MAX };

    for &move_dir in ["up", "down", "left", "right"].iter() {
        if is_move_safe(board, you, move_dir) {
            let mut new_board = board.clone();
            let mut new_you = you.clone();
            simulate_move(&mut new_board, &mut new_you, move_dir);

            let (score, _) = minimax(&new_board, &new_you, depth - 1, alpha, beta, !is_maximizing);

            if is_maximizing && score > best_score {
                best_score = score;
                best_move = move_dir.to_string();
            } else if !is_maximizing && score < best_score {
                best_score = score;
                best_move = move_dir.to_string();
            }

            if is_maximizing {
                alpha = std::cmp::max(alpha, score);
                if beta <= alpha {
                    break;
                }
            } else {
                beta = std::cmp::min(beta, score);
                if beta <= alpha {
                    break;
                }
            }
        }
    }

    (best_score, best_move)
}


pub fn get_move(_game: &Game, turn: &i32, board: &Board, you: &Battlesnake) -> Value {
    let depth = 4; // Depth limit for minimax recursion
    let (best_score, best_move) = minimax(board, you, depth,i32::MIN,i32::MAX, true);

    println!("MOVE {}: Best move is '{}' with a score of {}", turn, best_move, best_score);
    info!("MOVE {}: {}, score: {}", turn, best_move, best_score);
    json!({ "move": best_move })
}

