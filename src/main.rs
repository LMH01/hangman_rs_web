use std::sync::RwLock;

use game::{GameManager, Game};
use rocket::{http::{ContentType, CookieJar, Cookie}, fs::FileServer, fs::relative, serde::json::Json, State, request::FromRequest, route::Outcome};
use serde::Deserialize;
mod game;

#[macro_use] extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("web")))
        .mount("/", routes![register, submit_char, lives, game_string])
        .manage(RwLock::new(GameManager::new()))
}

#[post("/api/register", data = "<username>")]
fn register(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>, username: Json<Username<'_>>) -> (ContentType, String) {
    println!("Username: {}", username.username);
    let result = game_manager.write().unwrap().register_game(String::from(username.username));
    cookies.add(Cookie::new("userid", result.player_id.to_string()));
    (ContentType::Text, result.result_id.to_string())
}

#[post("/api/submit_char", data = "<character>")]
fn submit_char(cookies: &CookieJar<'_>, game_manager: &State<RwLock<GameManager>>, character: Json<Character>) -> (ContentType, String) {
    let userid = match userid_from_cookies(cookies) {
        Some(id) => id,
        None => return (ContentType::Text, String::from("No user id was submitted")),
    };
    let mut game_manager = game_manager.write().unwrap();
    let game = match game_manager.game_by_player_id(userid) {
        Some(game) => game,
        None => return (ContentType::Text, String::from("Invalid user id")),
    };
    (ContentType::Text, game.guess_letter(userid, character.character).to_string())
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
