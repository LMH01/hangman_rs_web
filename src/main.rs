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
        .mount("/", routes![events, register, submit_char, lives, game_string, player_number, word, guessed_letters, delete_game])
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
