use std::sync::RwLock;

use game::{GameManager, Game};
use rocket::{http::{ContentType, CookieJar, Cookie, uri::fmt::FromUriParam}, fs::FileServer, fs::relative, serde::json::Json, State, request::FromRequest, route::Outcome, Shutdown, response::Redirect};
use rocket::response::stream::{EventStream, Event};
use rocket::serde::{Serialize, Deserialize};
use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};
use rocket::tokio::select;

mod game;

#[macro_use] extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("web")))
        .mount("/", routes![events, register, registered, submit_char, lives, game_string, player_turn_position, word, guessed_letters, teammates, is_players_turn, game_id, delete_game])
        .manage(RwLock::new(GameManager::new()))
        .manage(channel::<EventData>(1024).0)
}

#[post("/api/register", data = "<username>")]
fn register(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>, username: Json<Username<'_>>, event: &State<Sender<EventData>>) -> Json<RegistrationData> {
    println!("Username: {}", username.username);
    let result = game_manager.write().unwrap().register_game(String::from(username.username), event);
    cookies.add(Cookie::new("userid", result.player_id.to_string()));
    Json(RegistrationData {result: result.result_id, game_id: result.game_id})
}

#[post("/api/submit_char", data = "<character>")]
fn submit_char(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>, character: Json<Character>, event: &State<Sender<EventData>>) -> (ContentType, String) {
    let userid = match userid_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("No user id was submitted")),
    };
    let mut game_manager = game_manager.write().unwrap();
    let game = match game_manager.game_by_player_id(userid) {
        Some(game) => game,
        None => return (ContentType::Text, String::from("Invalid user id")),
    };
    (ContentType::Text, game.guess_letter(userid, character.character, event).to_string())
}

#[get("/api/lives")]
fn lives(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> (ContentType, String) {
    let userid = match userid_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("No user id was submitted")),
    };
    let mut game_manager = game_manager.write().unwrap();
    let game = match game_manager.game_by_player_id(userid) {
        Some(game) => game,
        None => return (ContentType::Text, String::from("Invalid user id")),
    };
    (ContentType::Text, game.lives().to_string())
}

#[get("/api/game_string")]
fn game_string(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> (ContentType, String) {
    let userid = match userid_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("No user id was submitted")),
    };
    let mut game_manager = game_manager.write().unwrap();
    let game = match game_manager.game_by_player_id(userid) {
        Some(game) => game,
        None => return (ContentType::Text, String::from("Invalid user id")),
    };
    (ContentType::Text, game.game_string())
}

#[get("/api/word")]
fn word(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> (ContentType, String) {
    let userid = match userid_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("No user id was submitted")),
    };
    let mut game_manager = game_manager.write().unwrap();
    let game = match game_manager.game_by_player_id(userid) {
        Some(game) => game,
        None => return (ContentType::Text, String::from("Invalid user id")),
    };
    let ret = match game.word() {
        Some(word) => word,
        None => String::from("Unable to return word: Game has to end first!"),
    };
    (ContentType::Text, ret)
}

#[get("/api/delete_game")]
fn delete_game(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>, event: &State<Sender<EventData>>) -> (ContentType, String) {
    // I know that in this way the user does not have to confirm the deletion of the game.
    let userid = match userid_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("User_id not found, game has probably already been deleted")),
    };
    let mut game_manager = game_manager.write().unwrap();
    let game = match game_manager.game_by_player_id(userid) {
        Some(game) => game,
        None => return (ContentType::Text, String::from("Invalid user id")),
    };
    let game_id = game.game_id();
    game_manager.delete_game(userid);
    // Delete cookie
    cookies.remove(Cookie::named("userid"));
    // Send event to users
    let _x = event.send(EventData::new(0, game_id, String::from("game_deleted")));
    (ContentType::Text, String::from("Game has been deleted, users have been reset"))
}

#[get("/api/guessed_letters")]
fn guessed_letters(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> (ContentType, String) {
    let userid = match userid_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("No user id was submitted")),
    };
    let mut game_manager = game_manager.write().unwrap();
    let game = match game_manager.game_by_player_id(userid) {
        Some(game) => game,
        None => return (ContentType::Text, String::from("Invalid user id")),
    };
    (ContentType::Text, game.guessed_letters())
}

/// Returns the names of the teammates
#[get("/api/teammates")]
fn teammates(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> (ContentType, String) {
    let userid = match userid_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("No user id was submitted")),
    };
    let mut game_manager = game_manager.write().unwrap();
    let game = match game_manager.game_by_player_id(userid) {
        Some(game) => game,
        None => return (ContentType::Text, String::from("Invalid user id")),
    };
    (ContentType::Text, game.teammates(userid))
}

/// Check if the submitted `user_id` is valid and the user is assigned to a game
/// # Return
/// 'false' `user_id` is invalid
/// 'registered' user exists and is waiting for a game to start
/// 'playing' user exists and is playing in a game
/// 'won' if the game has ended and was won but is not yet deleted
/// `lost` if the game has ended and was lost but is not yet deleted
#[get("/api/registered")]
fn registered(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> (ContentType, String) {
    let userid = match userid_from_cookies(cookies) {
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

/// Check if its the players turn
/// # Returns
/// `true` if it is the players turn
/// `false` if it is not the players turn
#[get("/api/is_players_turn")]
fn is_players_turn(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> (ContentType, String) {
    let userid = match userid_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("No user id was submitted")),
    };
    let mut game_manager = game_manager.write().unwrap();
    let game = match game_manager.game_by_player_id(userid) {
        Some(game) => game,
        None => return (ContentType::Text, String::from("Invalid user id")),
    };
    (ContentType::Text, game.is_players_turn(userid).to_string())
}

/// Returns the game id to which the player is registered if they are registered to a game
#[get("/api/game_id")]
fn game_id(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> (ContentType, String) {
    let userid = match userid_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("No user id was submitted")),
    };
    let mut game_manager = game_manager.write().unwrap();
    let game = match game_manager.game_by_player_id(userid) {
        Some(game) => game,
        None => return (ContentType::Text, String::from("Invalid user id")),
    };
    (ContentType::Text, game.game_id().to_string()) 
}

/// # Returns
/// The players turn position
#[get("/api/player_turn_position")]
fn player_turn_position(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> (ContentType, String) {
    let userid = match userid_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("No user id was submitted")),
    };
    let mut game_manager = game_manager.write().unwrap();
    let game = match game_manager.game_by_player_id(userid) {
        Some(game) => game,
        None => return (ContentType::Text, String::from("Invalid user id")),
    };
    match game.player_turn_position(userid) {
        Some(position) => (ContentType::Text, position.to_string()),
        None => (ContentType::Text, String::from("Invalid user id")),
    }
}

#[get("/sse/<game_id>")]// TODO Umbauen, sodass es zu /sse/<game_id>/<player_number> wird, sodass jeder spieler nur noch events bekommt, die für ihn interessant sind. Außerdem: Die Fehler fixen
async fn events(event: &State<Sender<EventData>>, mut end: Shutdown, game_id: i32) -> EventStream![] {
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

// TODO use request guards to verify user_id as http header -> replaces the current cookie solution: https://api.rocket.rs/v0.4/rocket/request/struct.State.html (within request guards section)

/// Used to transmit data to the client with server side events
#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
pub struct EventData {
    /// Indicates to which player this request is directed.
    /// This is the player turn number and not the player id.
    /// 
    /// When this is 0 the message is meant to be relevant for all players.
    player: usize,
    /// Indicates for what game this request is relevant
    game_id: i32,
    /// Additional data
    data: String,
}

impl EventData {
    fn new(player: usize, game_id: i32, data: String) -> Self {
        Self {
            player,
            game_id,
            data, 
        }
    }

    /// # Returns
    /// The game id to which this data event belongs
    fn game_id(&self) -> i32 {
        self.game_id
    }
}

/// Used to submit data that is required for the registration to succeed back to the user
#[derive(Serialize, Deserialize)]
struct RegistrationData {
    /// The result of the registration
    result: i32,
    /// The game id to which the user is registered. Used to set the server send event endpoint
    game_id: i32,
}

#[derive(Deserialize)]
struct Username<'a> {
    username: &'a str,
}

#[derive(Deserialize)]
struct Character {
    character: char,
}

/// Retrieves the user id from the 'userid' cookie.
/// # Returns
/// 'Some(i32)' when the id was found
/// 'None' when the user id was not found or the cookie was not set
fn userid_from_cookies(cookies: &CookieJar<'_>) -> Option<i32> {
    match cookies.get("userid") {
        Some(cookie) => Some(cookie.value().parse().unwrap()),
        None => None,
    }
}
