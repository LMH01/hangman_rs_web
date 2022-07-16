use rocket::http::ContentType;

static mut INDEX: &str = "";
static mut SCRIPT: &str = "";

#[macro_use] extern crate rocket;

#[get("/")]
fn hello() -> String {
    "Hello, World".to_string()
}

#[get("/base")]
fn base() -> (ContentType, &'static str) {
    unsafe {
        (ContentType::HTML, INDEX)
    }
}

#[get("/script.js")]
fn script() -> (ContentType, &'static str) {
    unsafe {
        (ContentType::JavaScript, SCRIPT)
    }
}

#[get("/api/register")]
fn register() -> (ContentType, &'static str) {
    (ContentType::Text, "3")
}

#[launch]
fn rocket() -> _ {
    unsafe {
        INDEX = include_str!("../web/index.html");
        SCRIPT = include_str!("../web/script.js");
    }
    rocket::build().mount("/", routes![hello, base, script, register])
}
