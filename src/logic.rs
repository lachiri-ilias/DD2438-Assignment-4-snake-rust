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
    
    if my_neck.x < my_head.x { // Neck is left of head, don't move left
        is_move_safe.insert("left", false);

    } else if my_neck.x > my_head.x { // Neck is right of head, don't move right
        is_move_safe.insert("right", false);

    } else if my_neck.y < my_head.y { // Neck is below head, don't move down
        is_move_safe.insert("down", false);
    
    } else if my_neck.y > my_head.y { // Neck is above head, don't move up
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

        if my_body.iter().any(|segment| segment.x == new_x && segment.y == new_y) {
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

    let food = &board.food;  // Get a reference to the food positions on the board
   
    let closest_food = food.iter().min_by_key(|item| (item.x - my_head.x).abs() + (item.y - my_head.y).abs());
    if let Some(food) = closest_food {
        let mut min_distance = std::i32::MAX;
        let mut chosen = "up";  // Default direction

        for &(dx, dy, dir) in &[(-1, 0, "left"), (1, 0, "right"), (0, -1, "down"), (0, 1, "up")] {
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
