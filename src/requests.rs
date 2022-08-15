
use std::sync::{RwLock, RwLockWriteGuard};
use crate::{request_data::{Username, RegistrationData, Character, EventData, PlayerAuth}, game::{GameManager, base_game::Game}};
use rocket::{http::{ContentType, CookieJar, Cookie, uri::fmt::FromUriParam, Status}, fs::FileServer, fs::relative, serde::json::Json, State, request::{FromRequest, self, Outcome}, Shutdown, response::Redirect};
use rocket::response::stream::{EventStream, Event};
use rocket::serde::{Serialize, Deserialize};
use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};
use rocket::tokio::select;

use self::Utils::game_by_player_auth;

/// Register a new player to the server
/// 
/// A new cookie is set that will be used to authorize the user against the server in subsequent request to endpoints required to play the game.
/// 
/// This cookie is deleted when the game ends.
/// 
/// # Requires
/// The user needs to send a username formatted in a json string in the post request body.
/// 
/// # Return
/// [RegistrationData](../request_data/struct.RegistrationData.html) indicating the outcome of the registration
#[post("/api/register", data = "<username>")]
pub fn register(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>, username: Json<Username<'_>>, event: &State<Sender<EventData>>) -> Json<RegistrationData> {
    println!("Username: {}", username.username);
    let result = game_manager.write().unwrap().register_game(String::from(username.username), event);
    cookies.add(Cookie::new("userid", result.player_id.to_string()));
    Json(RegistrationData {result: result.result_id, game_id: result.game_id})
}

/// Submits a character to the game
/// 
/// # Requires
/// The user needs to send a character formatted in a json string in the post request body.
/// # Return
/// The result of [Game::guess_letter](../game/base_game/struct.Game.html#method.guess_letter)
#[post("/api/submit_char", data = "<character>")]
pub fn submit_char(game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth, character: Json<Character>, event: &State<Sender<EventData>>) -> (ContentType, String) {
    let mut game_manager = game_manager.write().unwrap();
    let game = game_by_player_auth(&mut game_manager, player_auth).unwrap();
    (ContentType::Text, game.guess_letter(player_auth.player_id, character.character, event).to_string())
}

/// The amount of lives left
/// 
/// See [Game::lives](../game/base_game/struct.Game.html#method.lives)
#[get("/api/lives")]
pub fn lives(game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth) -> (ContentType, String) {
    let mut game_manager = game_manager.write().unwrap();
    let game = game_by_player_auth(&mut game_manager, player_auth).unwrap();
    (ContentType::Text, game.lives().to_string())
}

/// The game string
/// 
/// See [Game::game_string](../game/base_game/struct.Game.html#method.game_string)
#[get("/api/game_string")]
pub fn game_string(game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth) -> (ContentType, String) {
    let mut game_manager = game_manager.write().unwrap();
    let game = game_by_player_auth(&mut game_manager, player_auth).unwrap();
    (ContentType::Text, game.game_string())
}

/// The correct word if the game has ended
/// 
/// See [Game::word](../game/base_game/struct.Game.html#method.word)
#[get("/api/word")]
pub fn word(game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth) -> (ContentType, String) {
    let mut game_manager = game_manager.write().unwrap();
    let game = game_by_player_auth(&mut game_manager, player_auth).unwrap();
    let ret = match game.word() {
        Some(word) => word,
        None => String::from("Unable to return word: Game has to end first!"),
    };
    (ContentType::Text, ret)
}

/// Delete the game the player is playing in
/// 
/// This removes the cookie that is used to authenticate the player against the server and completely delete the game from the server.
/// 
/// # Warning
/// The game will be deleted directly, the player will not have to confirm that the game should be deleted!
#[get("/api/delete_game")]
pub fn delete_game(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth, event: &State<Sender<EventData>>) -> (ContentType, String) {
    // I know that in this way the user does not have to confirm the deletion of the game.
    let mut game_manager = game_manager.write().unwrap();
    game_manager.delete_game(player_auth.player_id);
    // Delete cookie
    cookies.remove(Cookie::named("userid"));
    // Send event to users
    let _x = event.send(EventData::new(0, player_auth.game_id, String::from("game_deleted")));
    (ContentType::Text, String::from("Game has been deleted, users have been reset"))
}

/// All guessed letters
/// 
/// See [Game::guessed_letters](../game/base_game/struct.Game.html#method.guessed_letters)
#[get("/api/guessed_letters")]
pub fn guessed_letters(game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth) -> (ContentType, String) {
    let mut game_manager = game_manager.write().unwrap();
    let game = game_by_player_auth(&mut game_manager, player_auth).unwrap();
    (ContentType::Text, game.guessed_letters())
}

/// The names of the teammate
/// 
/// See [Game::teammates](../game/base_game/struct.Game.html#method.teammates)
#[get("/api/teammates")]
pub fn teammates(game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth) -> (ContentType, String) {
    let mut game_manager = game_manager.write().unwrap();
    let game = game_by_player_auth(&mut game_manager, player_auth).unwrap();
    (ContentType::Text, game.teammates(player_auth.player_id))
}

/// Check if its the players turn
/// 
/// # Returns
/// `true` if it is the players turn
/// 
/// `false` if it is not the players turn
#[get("/api/is_players_turn")]
pub fn is_players_turn(game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth) -> (ContentType, String) {
    let mut game_manager = game_manager.write().unwrap();
    let game = game_by_player_auth(&mut game_manager, player_auth).unwrap();
    (ContentType::Text, game.is_players_turn(player_auth.player_id).to_string())
}

/// The game id to which the player is registered
#[get("/api/game_id")]
pub fn game_id(game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth) -> (ContentType, String) {
    let mut game_manager = game_manager.write().unwrap();
    let game = game_by_player_auth(&mut game_manager, player_auth).unwrap();
    (ContentType::Text, game.game_id().to_string()) 
}

/// The players turn position
#[get("/api/player_turn_position")]
pub fn player_turn_position(game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth) -> (ContentType, String) {
    let mut game_manager = game_manager.write().unwrap();
    let game = game_by_player_auth(&mut game_manager, player_auth).unwrap();
    match game.player_turn_position(player_auth.player_id) {
        Some(position) => (ContentType::Text, position.to_string()),
        None => (ContentType::Text, String::from("Invalid user id")),
    }
}

/// Check if the submitted `user_id` is valid and the user is assigned to a game
/// 
/// # Return
/// `false` when the `user_id` is invalid
/// 
/// `registered` when the user exists and is waiting for a game to start
/// 
/// `playing` when the user exists and is playing in a game
/// 
/// `won` if the game has ended and was won but is not yet deleted
/// 
/// `lost` if the game has ended and was lost but is not yet deleted
#[get("/api/registered")]
pub fn registered(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> (ContentType, String) {
    let userid = match user_id_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("false")),
    };
    let mut game_manager = game_manager.write().unwrap();
    match game_manager.game_by_player_id(userid) {
        Some(game) => {
            match game.completed() {
                Some(win) => {
                    if win {
                        return (ContentType::Text, String::from("win"));
                    } else {
                        return (ContentType::Text, String::from("lost"));
                    }
                },
                None => return (ContentType::Text, String::from("playing"))
            }
        },
        None => {
            if game_manager.id_taken(userid) {
                return (ContentType::Text, String::from("registered"));
            } else {
                return (ContentType::Text, String::from("false"))
            }
        },
    };
}

/// Server send events
/// 
/// For each game a separate sse stream exists, these streams are accessed by submitting a get request to `/sse/<game_id>`.
/// 
/// This makes it possible to have multiple games run in parallel without interferences in the sse streams.
/// 
/// Only sse events that match the `game_id` will be transmitted back.
#[get("/sse/<game_id>")]
pub async fn events(event: &State<Sender<EventData>>, mut end: Shutdown, game_id: i32) -> EventStream![] {
    let mut rx = event.subscribe();
    EventStream! {
        loop {
            info!("Waiting for data, game_id is {}", game_id);
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };
            let msg_game_id = msg.game_id();
            info!("{} | {}", msg_game_id, game_id);
            if msg_game_id == game_id {
                info!("Yielding data: {:#?}", &msg);
                yield Event::json(&msg);
            }
        }
    }
}

/// Retrieves the user id from the `userid` cookie
/// 
/// # Returns
/// 'Some(i32)' when the id was found
/// 'None' when the user id was not found or the cookie was not set
pub fn user_id_from_cookies(cookies: &CookieJar<'_>) -> Option<i32> {
    match cookies.get("userid") {
        Some(cookie) => Some(cookie.value().parse().unwrap()),
        None => None,
    }
}

/// Some utility functions
mod Utils {
    use std::sync::RwLockWriteGuard;

    use crate::{game::{GameManager, base_game::Game}, request_data::PlayerAuth};

    /// Returns the game a player is assigned to by using the `player_auth`
    pub fn game_by_player_auth<'a>(game_manager: &'a mut RwLockWriteGuard<GameManager>, player_auth: PlayerAuth) -> Option<&'a mut Game> {
        match game_manager.game_by_player_id(player_auth.player_id) {
            Some(game) => Some(game),
            None => None,
        }
    }
}