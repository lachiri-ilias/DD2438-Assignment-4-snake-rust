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
use serde_json::{json, Value};
use std::collections::HashSet;

use std::collections::HashMap;
use rand::seq::SliceRandom;

use crate::{Battlesnake, Board, Coord, Game};

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
            if snake.body.len() > you.body.len() {
                println!("$$$$  Avoiding head-to-head collision with snake {}", snake.id);
                return false;
            }
        }
    }

    true
}

fn evaluate_board(board: &Board, you: &Battlesnake) -> i32 {
    let head = &you.body[0];
    let mut best_score = std::i32::MIN;
    let health_factor = if you.health < 60 { 50.0 } else { 10.0 };

    for food in &board.food {
        let food_distance = (food.x - head.x).abs() + (food.y - head.y).abs();
        let food_score = -(food_distance as f32) * health_factor;

        // Check if the food is directly reachable next move
        println!("Food distance: {}", food_distance);
        if food_distance <= 4 {
            return (food_score + 1000.0) as i32;  // Assign a very high score to prioritize eating
        }

        if food_score > best_score as f32 {
            best_score = food_score as i32;
        }
    }

    best_score
}


// fn evaluate_board(board: &Board, you: &Battlesnake) -> i32 {
//     let head = &you.body[0];
//     let mut best_food_score = std::i32::MIN;

//     // Dynamic health factor increases as health decreases
//     let health_factor = 100.0 - you.health as f32; // The less health, the higher the factor
//     let base_food_weight = 60.0; // Base weight for food proximity

//     for food in &board.food {
//         let food_distance = (food.x - head.x).abs() + (food.y - head.y).abs();
        
//         // Increase score significantly as snake approaches food
//         let food_score = -(food_distance as f32) * (base_food_weight + health_factor);
        
//         if food_score > best_food_score as f32 {
//             best_food_score = food_score as i32;
//         }
    
//     }

//     best_food_score
// }



// fn evaluate_board(board: &Board, you: &Battlesnake) -> i32 {
//     let head = &you.body[0];
//     let mut best_score = std::i32::MIN;
//     let health_importance = if you.health < 60 { 50.0 } else { 10.0 };

//     for food in &board.food {
//         let distance = (food.x - head.x).abs() + (food.y - head.y).abs();
//         let food_score = -distance as f32 * health_importance;
//         let area_control = evaluate_area_control(board, head);
//         let total_score = food_score + area_control as f32;
//         // let total_score = food_score;

//         if total_score > best_score {
//             best_score = total_score as i32;
//         }
//     }
//     // // FOR Royal board
//     // // Factor in hazards if applicable 
//     // if let Some(hazards) = board.hazards {
//     //     let hazard_penalty = if hazards.contains(head) { -100 } else { 0 };
//     //     best_score += hazard_penalty;
//     // }

//     best_score
// }


fn simulate_move(board: &mut Board, snake: &mut Battlesnake, move_dir: &str) {
    let (dx, dy) = match move_dir {
        "up" => (0, 1),
        "down" => (0, -1),
        "left" => (-1, 0),
        "right" => (1, 0),
        _ => (0, 0),
    };

    let mut new_head = snake.body[0].clone();
    new_head.x += dx;
    new_head.y += dy;

    // Check if the new head position is on a food
    if let Some(index) = board.food.iter().position(|f| f.x == new_head.x && f.y == new_head.y) {
        snake.body.insert(0, new_head); // Add new head to the body
        board.food.remove(index); // Remove the food from the board
    } else {
        snake.body.pop(); // Remove the last segment of the body if not eating
        snake.body.insert(0, new_head); // Add new head to the body
    }
}



// Implementing Area Control and Tactical Escapes
fn evaluate_area_control(board: &Board, head: &Coord) -> i32 {
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];  // Up, right, down, left
    let mut accessible_area = 0;
    
    let mut queue = vec![(head.x, head.y)];
    let mut visited = HashSet::new();
    visited.insert((head.x, head.y));

    while let Some((x, y)) = queue.pop() {
        for (dx, dy) in directions.iter() {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && nx < board.width as i32 && ny >= 0 && ny < board.height as i32 
                && !visited.contains(&(nx, ny)) 
                && !board.snakes.iter().any(|snake| snake.body.iter().any(|pos| pos.x == nx && pos.y == ny)) {
                queue.push((nx, ny));
                visited.insert((nx, ny));
                accessible_area += 1;
            }
        }
    }
    accessible_area
}




fn minimax(board: &Board, snakes: &Vec<Battlesnake>, depth: i32, alpha: i32, beta: i32, maximizing_player_index: usize) -> (i32, String) {
    if depth == 0 {
        return (evaluate_board(board, &snakes[0]), String::from("none"));  // Assuming index 0 is your snake
    }

    let mut alpha = alpha;
    let mut beta = beta;
    let mut current_best_move = String::from("none");
    let directions = ["up", "down", "left", "right"];
    let mut best_score = if maximizing_player_index == 0 { i32::MIN } else { i32::MAX };

    for &move_dir in &directions {
        let move_safe = is_move_safe(board, &snakes[maximizing_player_index], move_dir);
        // println!("Depth {} Move {} is safe: {}", depth, move_dir, move_safe);
        if move_safe {
            let mut new_board = board.clone(); 
            let mut new_snakes = snakes.clone();
            
            // Simulate move for the current player
            simulate_move(&mut new_board, &mut new_snakes[maximizing_player_index], move_dir);
            
            let next_player_index = (maximizing_player_index + 1) % snakes.len();
            let (score, _) = minimax(&new_board, &new_snakes, depth - 1, alpha, beta, next_player_index);

            if maximizing_player_index == 0 {  // Your snake is maximizing
                if score > best_score {
                    best_score = score;
                    current_best_move = move_dir.to_string();
                }
                alpha = std::cmp::max(alpha, score);
                if beta <= alpha {
                    break;
                }
            } else {  // Opponent snake is minimizing
                if score < best_score {
                    best_score = score;
                    current_best_move = move_dir.to_string();
                }
                beta = std::cmp::min(beta, score);
                if beta <= alpha {
                    break;
                }
            }
        }
    }

    (best_score, current_best_move)
}



pub fn get_move(_game: &Game, turn: &i32, board: &Board, you: &Battlesnake) -> Value {
    let depth = 8; // Adjust depth based on performance and time constraints
    let snakes = &board.snakes; // Add opponents here as well
   
    // find my snak Id in the snakes list only run this once
    let my_snake_index = snakes.iter().position(|s| s.id == you.id).unwrap();
    // println!("My snake index is: {}", my_snake_index);
    
    let (score, best_move) = minimax(board, &snakes, depth, i32::MIN, i32::MAX, my_snake_index); // 0 is your snake's index

    if best_move == "none" {
        println!("No best move found, choosing a random safe move...%%%%%%%%%%%%%%%%%%");
        let safe_moves = ["up", "down", "left", "right"].iter().filter(|&m| is_move_safe(board, you, m)).collect::<Vec<_>>();
        let random_move = safe_moves.choose(&mut rand::thread_rng()).unwrap();
        return json!({ "move": random_move });
    }

    println!("MOVE {}: Best move is '{}' with a score of {}", turn, best_move, score);
    json!({ "move": best_move })
}