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
use std::{clone, cmp::max, collections::HashSet, env::remove_var, thread::current};

use rand::seq::SliceRandom;

use crate::{Battlesnake, Board, Coord, Game, GameState};
const PRINT: bool = false;
static mut GAME_STARTED: bool = false;
// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "Ilias_Saad", // TODO: Your Battlesnake Username
        "color": "#006233", // TODO: Choose color
        "head": "do-sammy", // TODO: Choose head
        "tail": "mystic-moon", // TODO: Choose tail
    });
}

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    if PRINT {
        println!(
            "--------------GAME START-----------, my snake id: {}",
            _you.id
        );
    }
    unsafe { GAME_STARTED = true };
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
    if you
        .body
        .iter()
        .any(|segment| segment.x == new_x && segment.y == new_y)
    {
        return false;
    }

    // Avoid head-to-head collisions unless we are longer
    let my_head = (you.body[0].x, you.body[0].y);
    let binding = [
        (new_x + 1, new_y),
        (new_x - 1, new_y),
        (new_x, new_y + 1),
        (new_x, new_y - 1),
    ];
    let surrounding_positions: Vec<_> = IntoIterator::into_iter(binding)
        .filter(|&pos| {
            pos.0 >= 0 && pos.0 < board.width && pos.1 >= 0 && pos.1 < board.height as i32
        })
        .filter(|&pos| (pos != my_head) || (you.body.len() == 1))
        .collect();

    for snake in &board.snakes {
        if snake.id != you.id {
            if surrounding_positions.iter().any(|(x, y)| {snake.body[0].x == *x && snake.body[0].y == *y}) {
                if snake.body.len() >= you.body.len() {
                    return false;
                }
                else if snake.name != "L7anch"{
                    return true;
                }
            }
        }
    }

    // Check for collisions with other snakes
    for snake in &board.snakes {
        if snake
            .body
            .iter()
            .any(|segment| segment.x == new_x && segment.y == new_y)
        {
            return false;
        }
    }
    true
}

fn simulate_move(board: &mut Board, snake_id: usize, move_dir: &str) -> Option<Coord> {
    let (dx, dy) = match move_dir {
        "up" => (0, 1),
        "down" => (0, -1),
        "left" => (-1, 0),
        "right" => (1, 0),
        _ => (0, 0),
    };

    let mut new_head = board.snakes[snake_id].body[0].clone();
    new_head.x += dx;
    new_head.y += dy;

    // Check if the new head position is on a food
    if let Some(index) = board
        .food
        .iter()
        .position(|f| f.x == new_head.x && f.y == new_head.y)
    {
        board.snakes[snake_id].health = 100;
        board.snakes[snake_id].length += 1;
        board.snakes[snake_id].body.insert(0, new_head); // Add new head to the body
        let removed_food = board.food.remove(index);
        return Some(removed_food);
    } else {
        board.snakes[snake_id].health -= 1;
        board.snakes[snake_id].body.pop(); // Remove the last segment of the body if not eating
        board.snakes[snake_id].body.insert(0, new_head); // Add new head to the body
        return None;
    }

    // here add head-to-head collision detection
}



fn predict_snake_move_towards_food(snake: &Battlesnake, board: &Board) -> Coord {
    if let Some(food) = board
        .food
        .iter()
        .min_by_key(|f| (f.x - snake.body[0].x).abs() + (f.y - snake.body[0].y).abs())
    {
        let head = &snake.body[0];
        if food.x > head.x {
            return Coord {
                x: head.x + 1,
                y: head.y,
            };
        } else if food.x < head.x {
            return Coord {
                x: head.x - 1,
                y: head.y,
            };
        } else if food.y > head.y {
            return Coord {
                x: head.x,
                y: head.y + 1,
            };
        } else if food.y < head.y {
            return Coord {
                x: head.x,
                y: head.y - 1,
            };
        }
    }
    snake.body[0].clone() // Return current head position if no food or can't move closer
}

fn evaluate_board(board: &Board, you_id: usize) -> i32 {
    let you = &board.snakes[you_id];
    let head = &you.body[0];
    let mut score = 0;

    // Check if the snake just ate food (health is max)
    let just_ate_food = you.health > 90;
    //let dead = you.health == 0;

    // Calculate distance to the nearest food
    let mut min_food_distance = std::i32::MAX;
    for food in &board.food {
        let food_distance = (food.x - head.x).abs() + (food.y - head.y).abs();
        if food_distance < min_food_distance {
            min_food_distance = food_distance;
        }
    }

    // Calculate distance to the nearest opponent
    let mut min_enemy_distance = std::i32::MAX;
    for snake in &board.snakes {
        if snake.id != you.id {
            let predicted_position = predict_snake_move_towards_food(snake, board);
            let distance_to_predicted =
                (predicted_position.x - head.x).abs() + (predicted_position.y - head.y).abs();
            if distance_to_predicted < min_enemy_distance {
                min_enemy_distance = distance_to_predicted;
            }
        }
    }

    let mut nb_of_snakes_dead = 0;
    for s in &board.snakes {
        if s.id != you.id && s.body.len() == 0 {
            nb_of_snakes_dead += 1;
        }
    }

    // Factor food distance into the score
    if just_ate_food {
        score += 100; // High score for eating food
    }
    if min_food_distance != std::i32::MAX {
        score += 100 / (min_food_distance + 1); // Normal weight
    }
    // Apply a non-linear penalty for being close to an enemy
    if min_enemy_distance != std::i32::MAX {
        score -= 100 / (min_enemy_distance + 1);
    }

    // Include health in the scoring
    score += you.health; // Add health as a positive factor
    if you.health < 50 {
        score -= (50 - you.health) * 2; // Penalize low health more heavily
    }

    
    // print nb of snakes dead
    //println!("nb of snakes dead: {}", nb_of_snakes_dead);
    score += 1500 * nb_of_snakes_dead;

    score
}

fn minimax(
    mut board: &mut Board,
    depth: i32,
    alpha: i32,
    beta: i32,
    maximizing_player_index: usize,
    mut current_player_index: usize,
) -> (i32, String) {
    while board.snakes[current_player_index].body.len() == 0 {
        current_player_index = (current_player_index + 1) % board.snakes.len();
    }
    if depth == 0 {
        let score = evaluate_board(board, maximizing_player_index);
        if PRINT {
            println!(
                "depth: {}, snake id: {}, score: {}",
                depth, current_player_index, score
            );
        }

        return (score, String::from("none"));
    }

    let mut alpha = alpha;
    let mut beta = beta;
    let mut current_best_move = String::from("none");
    let directions = ["up", "down", "left", "right"];
    let mut best_score = if current_player_index == maximizing_player_index {
        i32::MIN
    } else {
        i32::MAX
    };
    let mut move_found = false; // Track if any valid move is found

    for &move_dir in &directions {
        if is_move_safe(board, &board.snakes[current_player_index], move_dir) {
            move_found = true;
            // let mut new_board = board.clone();

            // Simulate move for the current player
            let original_snake = board.snakes[current_player_index].clone();
            let removed_food = simulate_move(&mut board, current_player_index, move_dir);

            if let Some(food) = removed_food {
                board.food.insert(0, food);
            }

            let next_player_index = (current_player_index + 1) % board.snakes.len();
            let (score, _) = minimax(
                &mut board,
                depth - 1,
                alpha,
                beta,
                maximizing_player_index,
                next_player_index,
            );

            board.snakes[current_player_index] = original_snake;

            if PRINT {
                println!(
                    "depth: {}, move: {}, snake id: {}, score: {}, maximizing id: {}",
                    depth, move_dir, current_player_index, score, maximizing_player_index
                );
            }

            if (current_player_index == maximizing_player_index && score > best_score)
                || (current_player_index != maximizing_player_index && score < best_score)
            {
                best_score = score;
                current_best_move = move_dir.to_string();
            }

            if current_player_index == maximizing_player_index {
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

    if !move_found {
        // Handle no safe moves found
        return if current_player_index == maximizing_player_index {
            (i32::MIN, String::from("none"))
        } else {
            (i32::MAX, String::from("none"))
        };
    }

    (best_score, current_best_move)
}

pub fn get_move(_game: &Game, turn: &i32, board: &Board, you: &Battlesnake) -> Value {
    if !unsafe { GAME_STARTED } {
        let head = you.body.first().unwrap();
        
        // Define the middle upper part of the map
        let middle_x_start = board.width / 3;
        let middle_x_end = 2 * board.width / 3;
        let upper_y = board.height / 3;

        // Check if the snake's head is in the middle upper part of the map
        if head.x >= middle_x_start as i32 && head.x <= middle_x_end as i32 && head.y <= upper_y as i32 {
            return json!({ "move": "down" });
        } else {
            return json!({ "move": "up" });
        }
    }
    println!("----------------NEW TURN----------------");
    let depth = 12;
    // let depth = if board.snakes.len() < 3 { 16 } else { 12 }; // Adjust depth based on the number of snakes
    let snakes = &board.snakes; // Add opponents here as well

    let my_snake_index = snakes.iter().position(|s| s.id == you.id).unwrap();

    let mut cloned_board = board.clone();
    let (score, best_move) = minimax(
        &mut cloned_board,
        depth,
        i32::MIN,
        i32::MAX,
        my_snake_index,
        my_snake_index,
    );

    if best_move == "none" {
        println!("No best move found, choosing a random safe move...");
        let safe_moves = ["up", "down", "left", "right"]
            .iter()
            .filter(|&m| is_move_safe(board, you, m))
            .collect::<Vec<_>>();
        let random_move = safe_moves.choose(&mut rand::thread_rng()).unwrap();
        return json!({ "move": random_move });
    }

    info!(
        "MOVE {}: Best move is '{}' with a score of {}",
        turn, best_move, score
    );

    json!({ "move": best_move })
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
