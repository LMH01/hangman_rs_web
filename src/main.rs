use std::sync::{RwLock};

use game::GameManager;
use rocket::{fs::{FileServer, relative}, Config};

use crate::paths::*;

/// The underlying game, contains logic and components that are required to run the game
mod game;
/// All paths for which a request handler is registered.
/// 
/// All requests that interact with games require a player authentication that is set when the player registers for a game.
/// This authentication is done by setting a cookie that is checked each time the player interacts with the server endpoints.
/// When the cookie is invalid or not set the connection is refused.
mod paths;
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
        .mount("/", routes![singleplayer, register, registered, submit_char, lives, game_string, word, guessed_letters, teammates, game_id, delete_game])
        .manage(RwLock::new(GameManager::new()))
}
