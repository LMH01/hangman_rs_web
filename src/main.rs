use std::sync::{RwLock};

use game::GameManager;
use request_data::EventData;
use rocket::{fs::{FileServer, relative}, tokio::sync::broadcast::channel};

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
    rocket::build()
        .mount("/", FileServer::from(relative!("web")))
        .mount("/", routes![events, register, registered, submit_char, lives, game_string, player_turn_position, word, guessed_letters, teammates, is_players_turn, game_id, delete_game])
        .manage(RwLock::new(GameManager::new()))
        .manage(channel::<EventData>(1024).0)
}

// TODO:  
//  1. Authentifikation umbauen, (um http headers zu benutzen (bin ich mit mittlerweile nicht mehr so sicher, ich glaube, dass cookies keine schlechte Idee sind))
//  2. SSE handling umbauen, dass jeder nur noch das bekommt, was für ihn relevant ist (Nicht wirklich nötig, wird bei Acquire_rs dann vielleicht eingebaut)
//  4. Code dokumentieren, vor allem die Funktionen in der Main, da dokumentieren, was genau zurück kommt 
//    (ggf. kann ich darauf ja dann in der Readme verweisen und das gebuildete rust doc dazu mit in das Repo packen)
//  4.1 cargo clippy ausführen und Warnungen beheben
//  5. README überarbeiten und die REST Pfade richtig dokumentieren

//  3. Code aufräumen, nicht benötigtes weg löschen, variablen umbenennen (vor allem in JavaScript Teil)
//  3.1 Debug prints aufräumen (sowohl server, als auch client)
//  3.2 Alles, was user beinhaltet in player umbenennen
