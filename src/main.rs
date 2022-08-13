use std::sync::RwLock;

use game::{GameManager, Game};
use rocket::{http::{ContentType, CookieJar, Cookie}, fs::FileServer, fs::relative, serde::json::Json, State, request::FromRequest, route::Outcome, Shutdown};
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
        .mount("/", routes![events, register, submit_char, lives, game_string, player_number, word])
        .manage(RwLock::new(GameManager::new()))
        .manage(channel::<EventData>(1024).0)
}

#[post("/api/register", data = "<username>")]
fn register(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>, username: Json<Username<'_>>, event: &State<Sender<EventData>>) -> (ContentType, String) {
    println!("Username: {}", username.username);
    let result = game_manager.write().unwrap().register_game(String::from(username.username), event);
    cookies.add(Cookie::new("userid", result.player_id.to_string()));
    (ContentType::Text, result.result_id.to_string())
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

#[get("/api/player_number")]
fn player_number(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>) -> (ContentType, String) {
    let userid = match userid_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("No user id was submitted")),
    };
    let mut game_manager = game_manager.write().unwrap();
    let game = match game_manager.game_by_player_id(userid) {
        Some(game) => game,
        None => return (ContentType::Text, String::from("Invalid user id")),
    };
    (ContentType::Text, game.current_player().to_string())
}

use rocket::tokio::time::{self, Duration};

#[get("/sse")]
async fn events(event: &State<Sender<EventData>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = event.subscribe();
    EventStream! {
        loop {
            info!("Waiting for data");
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };
            info!("Yielding data: {:#?}", &msg);
            yield Event::json(&msg);
        }
    }
}

/// Used to transmit data to the client with server side events
#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
pub struct EventData {
    /// Indicates to which player this request is directed.
    /// This is the player turn number and not the player id.
    /// 
    /// When this is 0 the message is meant to be relevant for all players.
    player: usize,
    /// Additional data
    data: String,
}

impl EventData {
    fn new(player: usize, data: String) -> Self {
        Self {
            player,
            data, 
        }
    }
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
