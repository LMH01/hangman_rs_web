use rocket::{http::{ContentType, CookieJar, Cookie}, fs::FileServer, fs::relative, serde::json::Json};
use serde::Deserialize;
mod game;

#[macro_use] extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", FileServer::from(relative!("web"))).mount("/", routes![register])
}

#[post("/api/register", data = "<username>")]
fn register(cookies: &CookieJar<'_>, username: Json<Username<'_>>) -> (ContentType, &'static str) {
    println!("Username: {}", username.username);
    cookies.add(Cookie::new("userid", username.username.to_string()));
    (ContentType::Text, "3")
}

#[derive(Deserialize)]
struct Username<'a> {
    username: &'a str,
}
