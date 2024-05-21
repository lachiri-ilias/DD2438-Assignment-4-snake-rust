use crate::{Battlesnake, Board, Coord, Game};
use log::info;
use rand::seq::SliceRandom;
use serde_json::{json, Value};
use std::collections::HashSet;

const DEBUG: bool = false;

// info is called when you create your Battlesnake on play.battlesnake.com
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "", // TODO: Your Battlesnake Username
        "color": "#c1272d", // TODO: Choose color
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

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
pub fn get_move(_game: &Game, turn: &i32, board: &Board, you: &Battlesnake) -> Value {
    let mut cloned_board: Board = board.clone();
    let depth = 9;
    //println!("board.snakes.len(): {}", board.snakes.len());
    let my_snake_index = board.snakes.iter().position(|s| s.id == you.id).unwrap();

    let (best_move, score) = minimax(
        &mut cloned_board,
        depth,
        my_snake_index,
        my_snake_index,
        i32::MIN,
        i32::MAX,
    );

    if best_move == "none" {
        println!("No best move found, choosing a random safe move...");
        let safe_moves = ["up", "down", "left", "right"];
        let random_move = safe_moves.choose(&mut rand::thread_rng()).unwrap();
        return json!({ "move": random_move });
    }

    info!(
        "MOVE {}: Best move is '{}' with a score of {}",
        turn, best_move, score
    );

    json!({ "move": best_move })
}

pub fn minimax(
    mut board: &mut Board,
    depth: u32,
    current_index: usize,
    my_index: usize,
    mut alpha: i32,
    mut beta: i32,
) -> (String, i32) {
    // if max depth or only 1 snake left, return score
    if depth == 0
        || board.snakes[my_index].health == 0
        || board.snakes[current_index].health == 0
        || board.snakes.iter().filter(|&s| s.body.len() > 0).count() < 2
    {
        let score = evaluate_board(board, my_index);
        return (String::from("none"), score);
    }

    let mut best_move = String::from("none");
    let mut best_score: i32 = if current_index == my_index {
        i32::MIN
    } else {
        i32::MAX
    };

    let dirs = ["left", "up", "right", "down"];

    for dir in dirs {
        let old_snakes = board.snakes.clone();
        let removed_food = simulate_move(&mut board, current_index, dir);

        // recursive call
        let new_current_index = (current_index + 1) % board.snakes.len();
        let (_, score) = minimax(board, depth - 1, new_current_index, my_index, alpha, beta);

        // print
        if DEBUG {
            print_board(board, &board.snakes[my_index]);
        }

        // reset board to its past state
        board.snakes = old_snakes;
        if let Some(f) = removed_food {
            board.food.push(f);
        }

        if DEBUG {
            println!(
                "snake index: {}, my index: {}, curr depth: {}, move: {}, score: {}",
                current_index, my_index, depth, dir, score
            );
        }

        // change minimax variables
        if current_index == my_index {
            if score > best_score {
                best_score = score;
                best_move = String::from(dir);
                alpha = score;
            }
        } else {
            if score < best_score {
                best_score = score;
                best_move = String::from(dir);
                beta = score;
            }
        }
        if beta <= alpha {
            break;
        }
    }
    (best_move, best_score)
}

pub fn evaluate_board(board: &Board, my_index: usize) -> i32 {
    // if my snake is dead, return the minimum value
    if board.snakes[my_index].body.len() == 0 {
        return i32::MIN;
    }
    let mut score: i32 = 0;
    let nb_of_dead_snakes = board
        .snakes
        .iter()
        .filter(|&s| s.body.len() == 0 && s.name != "L7aya")
        .count() as i32;
    //println!("nb of dead snakes: {}", nb_of_dead_snakes);
    score += nb_of_dead_snakes * 500;
    score += board.snakes[my_index].health;
    score += board.snakes[my_index].body.len() as i32 * 100; // the longer the better
    if board.snakes[my_index].health > 95 {
        score += 100;
    }
    let mut min_food_distance = std::i32::MAX;
    for food in &board.food {
        let food_distance = (food.x - board.snakes[my_index].body[0].x).abs()
            + (food.y - board.snakes[my_index].body[0].y).abs();
        if food_distance < min_food_distance {
            min_food_distance = food_distance;
        }
    }

    if min_food_distance != std::i32::MAX {
        score += 100 / (min_food_distance + 1);
    }

    let head = board.snakes[my_index].body[0];
    if head.x < 2 || head.x > board.width - 3 || head.y < 2 || head.y > board.height - 3 {
        score -= 100; // Penalize being too close to walls
    }

    // Evaluate space around the snake head
    let directions = [(0, 1), (0, -1), (-1, 0), (1, 0)];
    let mut safe_moves = 0;
    for (dx, dy) in directions {
        let next = Coord {
            x: head.x + dx,
            y: head.y + dy,
        };
        if next.x >= 0
            && next.x < board.width
            && next.y >= 0
            && next.y < board.height
            && !board.snakes.iter().any(|s| s.body.contains(&next))
        {
            safe_moves += 1;
        }
    }

    score += safe_moves * 50; // Reward for having more escape routes
    if DEBUG {
        println!(
            "score: {}, health: {}, body len: {}, min food dist: {}, nb dead: {}",
            score,
            board.snakes[my_index].health,
            board.snakes[my_index].body.len(),
            min_food_distance,
            nb_of_dead_snakes
        );
    }

    score
}

pub fn simulate_move(board: &mut Board, snake_index: usize, action: &str) -> Option<Coord> {
    let (dx, dy) = match action {
        "up" => (0, 1),
        "down" => (0, -1),
        "left" => (-1, 0),
        "right" => (1, 0),
        _ => (0, 0),
    };

    // update the head's position
    let new_head = Coord {
        x: board.snakes[snake_index].body[0].x + dx,
        y: board.snakes[snake_index].body[0].y + dy,
    };

    if new_head.x < 0 || new_head.x >= board.width || new_head.y < 0 || new_head.y >= board.height {
        board.snakes[snake_index].health = 0;
        board.snakes[snake_index].body.clear();
        return None;
    }
    // variables used:
    let mut curr_snake_dies = false;
    let mut other_snake_dies = false;
    let mut other_snake_index = -1;

    // check for collisions with walls
    if new_head.x < 0 || new_head.x >= board.width || new_head.y < 0 || new_head.y >= board.height {
        curr_snake_dies = true;
    }

    // check for collisions with snakes bodies
    if !curr_snake_dies {
        for i in 0..board.snakes.len() {
            for c_index in 1..board.snakes[i].body.len() {
                let c = &board.snakes[i].body[c_index];
                if c.x == new_head.x && c.y == new_head.y {
                    curr_snake_dies = true;
                }
            }
        }
    }

    // check for head to head collisions

    if !curr_snake_dies {
        let binding = [
            (new_head.x + 1, new_head.y),
            (new_head.x - 1, new_head.y),
            (new_head.x, new_head.y + 1),
            (new_head.x, new_head.y - 1),
        ];
        let surrounding_positions: Vec<_> = IntoIterator::into_iter(binding)
            .filter(|&pos| {
                pos.0 >= 0 && pos.0 < board.width && pos.1 >= 0 && pos.1 < board.height as i32
            })
            .filter(|&pos| {
                pos != (
                    board.snakes[snake_index].body[0].x,
                    board.snakes[snake_index].body[0].y,
                )
            })
            .collect();
        for i in 0..board.snakes.len() {
            if i != snake_index && board.snakes[i].body.len() > 0 {
                let c = &board.snakes[i].body[0];
                if surrounding_positions
                    .iter()
                    .any(|(x, y)| c.x == *x && c.y == *y)
                {
                    if board.snakes[i].body.len() > board.snakes[snake_index].body.len() {
                        curr_snake_dies = true;
                    } else if board.snakes[i].body.len() == board.snakes[snake_index].body.len() {
                        curr_snake_dies = true;
                        other_snake_dies = true;
                        other_snake_index = i as i32;
                    } else {
                        other_snake_dies = true;
                        other_snake_index = i as i32;
                    }
                }
            }
        }
    }

    // check for food eaten
    let mut ate_food = false;
    let mut food_index = -1;
    if !curr_snake_dies {
        for i in 0..board.food.len() {
            if board.food[i].x == new_head.x && board.food[i].y == new_head.y {
                ate_food = true;
                food_index = i as i32;
                break;
            }
        }
    }

    // change the board
    if curr_snake_dies && other_snake_dies {
        board.snakes[snake_index].health = 0;
        board.snakes[snake_index].body.clear();
        board.snakes[other_snake_index as usize].health = 0;
        board.snakes[other_snake_index as usize].body.clear();
    } else if curr_snake_dies {
        board.snakes[snake_index].health = 0;
        board.snakes[snake_index].body.clear();
    } else if other_snake_dies {
        board.snakes[other_snake_index as usize].health = 0;
        board.snakes[other_snake_index as usize].body.clear();
    } else if ate_food {
        board.snakes[snake_index].health = 100;
        board.snakes[snake_index].body.insert(0, new_head);
        let removed_food = board.food[food_index as usize].clone();
        board.food.remove(food_index as usize);
        return Some(removed_food);
    } else {
        // nothing happened, simple snake move, no food eaten, no collision
        board.snakes[snake_index].health -= 1;
        if board.snakes[snake_index].health == 0 {
            board.snakes[snake_index].body.clear();
        } else {
            board.snakes[snake_index].body.insert(0, new_head);
            board.snakes[snake_index].body.pop();
        }
    }
    None
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
