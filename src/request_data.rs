use std::sync::RwLock;

use rocket::{request::{FromRequest, Outcome}, http::Status};
use serde::{Serialize, Deserialize};

use crate::{game::GameManager, requests::user_id_from_cookies};

/// Errors that can occur when the player tries to authenticate a request
#[derive(Debug)]
pub enum PlayerAuthError {
    /// The transmitted id-cookie is missing
    Missing,
    /// The transmitted id-cookie is invalid
    Invalid,
}

/// Symbolizes the authentication of a player.
/// 
/// A authenticated player is assigned to a game.
#[derive(Clone, Copy)]
pub struct PlayerAuth {
    /// The unique id that identifies this player
    pub player_id: i32,
    /// The game in which the player plays
    pub game_id: i32,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PlayerAuth {
    type Error = PlayerAuthError;

    async fn from_request(request: &'r rocket::Request<'_>) ->  Outcome<Self, Self::Error> {
        let userid = match user_id_from_cookies(request.cookies()) {
            Some(id) => id,
            None => return Outcome::Failure((Status::Forbidden, PlayerAuthError::Missing)),
        };
        let mut game_manager = request.rocket().state::<RwLock<GameManager>>().unwrap().write().unwrap();
        let game = match game_manager.game_by_player_id(userid) {
            Some(game) => game,
            None => return Outcome::Failure((Status::Forbidden, PlayerAuthError::Invalid)),
        };
        Outcome::Success(PlayerAuth { player_id: userid, game_id: game.game_id()})
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
    /// Indicates for what game this request is relevant
    game_id: i32,
    /// Additional data
    data: String,
}

impl EventData {
    pub fn new(player: usize, game_id: i32, data: String) -> Self {
        Self {
            player,
            game_id,
            data, 
        }
    }

    /// # Returns
    /// The game id to which this data event belongs
    pub fn game_id(&self) -> i32 {
        self.game_id
    }
}

/// Used to submit data that is required for the registration to succeed back to the user
#[derive(Serialize, Deserialize)]
pub struct RegistrationData {
    /// The result of the registration.
    /// 
    /// See [RegisterResult](../game/struct.RegisterResult.html) for more information.
    pub result: i32,
    /// The game id to which the user is registered. Used to set the server send event endpoint
    pub game_id: i32,
}

/// Used to get the username from a request formatted as json
#[derive(Deserialize)]
pub struct Username<'a> {
    pub username: &'a str,
}

/// Used to get the character from a request formatted as json
#[derive(Deserialize)]
pub struct Character {
    pub character: char,
}