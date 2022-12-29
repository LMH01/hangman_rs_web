
use std::{sync::RwLock, path::Path};
use crate::{request_data::{Character, PlayerAuth, PlayerAuthError}, game::GameManager};
use rocket::{http::{ContentType, CookieJar, Cookie}, serde::json::Json, State, fs::NamedFile};
use uuid::Uuid;

use self::utils::game_by_player_auth;

/// Returns the singleplayer html page
#[get("/singleplayer")]
pub async fn singleplayer() -> Option<NamedFile> {
    NamedFile::open(Path::new("web/singleplayer/singleplayer.html")).await.ok()
}

/// Register a new player to the server
/// 
/// A new cookie is set that will be used to authorize the user against the server in subsequent request to endpoints required to play the game.
/// 
/// This cookie is deleted when the game ends.
/// 
/// # Requires
/// Nothing
/// 
/// # Return
/// Uuid that is required to authenticate subsequent requests to the server.
#[post("/api/register")]
pub fn register(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> Json<String> {
    let result = game_manager.write().unwrap().register_game();
    cookies.add(Cookie::new("uuid", result.player_id.to_string()));
    Json(result.player_id.to_string())
}

/// Submits a character to the game
/// 
/// # Requires
/// The user needs to send a character formatted in a json string in the post request body.
/// 
/// # Return
/// The result of [Game::guess_letter](../game/base_game/struct.Game.html#method.guess_letter)
#[post("/api/submit_char", data = "<character>")]
pub fn submit_char(game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth, character: Json<Character>) -> (ContentType, String) {
    let mut game_manager = game_manager.write().unwrap();
    let game = game_by_player_auth(&mut game_manager, player_auth).unwrap();
    (ContentType::Text, game.guess_letter(character.character).to_string())
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
pub fn delete_game(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth) -> (ContentType, String) {
    // I know that in this way the user does not have to confirm the deletion of the game.
    let mut game_manager = game_manager.write().unwrap();
    game_manager.delete_game(player_auth.player_id);
    // Delete cookie
    cookies.remove(Cookie::named("uuid"));
    // Send event to users
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

/// The game id to which the player is registered
#[get("/api/game_id")]
pub fn game_id(game_manager: &State<RwLock<GameManager>>, player_auth: PlayerAuth) -> (ContentType, String) {
    let mut game_manager = game_manager.write().unwrap();
    let game = game_by_player_auth(&mut game_manager, player_auth).unwrap();
    (ContentType::Text, game.game_id().to_string()) 
}

/// Check if the submitted `uuid` is valid and the user is assigned to a game
/// 
/// # Return
/// `false` when the `uuid` is invalid
/// 
/// `playing` when the user exists and is playing in a game
/// 
/// `won` if the game has ended and was won but is not yet deleted
/// 
/// `lost` if the game has ended and was lost but is not yet deleted
#[get("/api/registered")]
pub fn registered(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> (ContentType, String) {
    let userid = match uuid_from_cookies(cookies) {
        Ok(id) => id,
        Err(_err) => return (ContentType::Text, String::from("false")),
    };
    let mut game_manager = game_manager.write().unwrap();
    match game_manager.game_by_player_id(userid) {
        Some(game) => {
            match game.completed() {
                Some(win) => {
                    if win {
                        (ContentType::Text, String::from("win"))
                    } else {
                        (ContentType::Text, String::from("lost"))
                    }
                },
                None => (ContentType::Text, String::from("playing"))
            }
        },
        None => (ContentType::Text, String::from("false")),
    }
}

/// Retrieves the user id from the `uuid` cookie.
/// 
/// # Returns
/// - `Some(Uuid)` when the id was found and is formatted as a valid uuid.
/// - `Err(PlayerAuthError)` when the user id was not found or the cookie was not set
pub fn uuid_from_cookies(cookies: &CookieJar<'_>) -> Result<Uuid, PlayerAuthError> {
    match cookies.get("uuid").map(|cookie| cookie.value().parse::<String>().unwrap()) {
        Some(id) => {
            match Uuid::parse_str(&id) {
                Ok(uuid) => return Ok(uuid),
                Err(_err) => return Err(PlayerAuthError::Invalid),
            }
        },
        None => Err(PlayerAuthError::Missing),
    }
}

/// Some utility functions
mod utils {
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