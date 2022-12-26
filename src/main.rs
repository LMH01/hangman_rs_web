use std::sync::{RwLock};

use game::GameManager;
use request_data::EventData;
use rocket::{fs::{FileServer, relative}, tokio::sync::broadcast::channel, Config};

use crate::requests::*;

/// The underlying game, contains logic and components that are required to run the game
mod game;
/// All requests that the server can handle
/// 
/// All requests that interact with games require a player authentication that is set when the player registers for a game.
/// This authentication is done by setting a cookie that is checked each time the player interacts with the server endpoints.
/// When the cookie is invalid or not set the connection is refused.
mod requests;
/// Different data types that are required to process requests
mod request_data;

#[macro_use] extern crate rocket;

/// Start server
#[launch]
fn rocket() -> _ {
    let config = Config::figment().merge(("port", 11511));
    //rocket::build()
    rocket::custom(config)
        .mount("/", FileServer::from(relative!("web")))
        .mount("/", routes![events, register, registered, submit_char, lives, game_string, player_turn_position, word, guessed_letters, teammates, is_players_turn, game_id, delete_game])
        .manage(RwLock::new(GameManager::new()))
        .manage(channel::<EventData>(1024).0)
}

// TODO:  
//  - Rename variables (especially in the JS code)
//  - Rename everything that contains user to player
