use std::sync::RwLock;

use rocket::{request::{FromRequest, Outcome}, http::Status};
use serde::Deserialize;
use uuid::Uuid;

use crate::{game::GameManager, paths::uuid_from_cookies};

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
    pub player_id: Uuid,
    /// The game in which the player plays
    pub game_id: Uuid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PlayerAuth {
    type Error = PlayerAuthError;

    async fn from_request(request: &'r rocket::Request<'_>) ->  Outcome<Self, Self::Error> {
        let uuid = match uuid_from_cookies(request.cookies()) {
            Ok(uuid) => uuid,
            Err(pae) => return Outcome::Failure((Status::Forbidden, pae))
        };
        let mut game_manager = request.rocket().state::<RwLock<GameManager>>().unwrap().write().unwrap();
        let game = match game_manager.game_by_player_id(uuid) {
            Some(game) => game,
            None => return Outcome::Failure((Status::Forbidden, PlayerAuthError::Invalid)),
        };
        Outcome::Success(PlayerAuth { player_id: uuid, game_id: game.game_id()})
    }
}

/// Used to get the character from a request formatted as json
#[derive(Deserialize)]
pub struct Character {
    pub character: char,
}