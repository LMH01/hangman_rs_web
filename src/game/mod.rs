use std::{fs, collections::{HashMap, HashSet, LinkedList}};
use rand::Rng;
use uuid::Uuid;

use self::base_game::Game;

/// Contains all base components that are required to run a game
pub mod base_game;

/// Determines how many lives players have when playing a game.
/// 
/// Should not be set higher than 10 because images will fail to load.
const MAX_LIVES: i32 = 7;

/// The maximum amount of active games at the same time.
/// 
/// If this number is reached the oldest game is deleted to make space for a new game.
const MAX_ACTIVE_GAMES: usize = 1000;

/// Used to manage all currently running games.
/// 
/// One `GameManager` instance is managed by rocket and given to each request handler.
pub struct GameManager {
    /// Contains all games that are currently running.
    /// The key is the user id and the value is the game where the user is assigned to.
    games: HashMap<Uuid, Game>,// TODO replace Game with RwLock<Game>
    /// All words from which a random word can be chosen for a game
    words: Vec<String>,
    /// All player ids that are already in use. 
    /// 
    /// A player id uniquely identifies the given player. 
    /// 
    /// It is also used to authorize the player against the server.
    player_ids: HashSet<Uuid>,
    /// This list is used to remove the oldest uuid once the [MAX_ACTIVE_GAMES](constant.MAX_ACTIVE_GAMES.html) limit is reached
    /// and a new game is registered.
    player_id_history: LinkedList<Uuid>,
    /// All game ids that are currently in use.
    game_ids: HashSet<Uuid>,
}

impl GameManager {
    /// Create a new `GameManager`
    pub fn new() -> Self {
        let file = fs::read_to_string("words.txt").expect("Unable to read words file!");
        let words: Vec<String> = file.split('\n').map(|s| String::from(s).to_uppercase()).collect();    
        Self {
            games: HashMap::new(),
            words,
            player_ids: HashSet::new(), 
            player_id_history: LinkedList::new(),
            game_ids: HashSet::new(),
        }
    }

    /// Registers a new game
    /// # Returns
    /// [RegisterResult](struct.RegisterResult.html) the result of the registration
    pub fn register_game(&mut self) -> RegisterResult {
        let game_id = self.free_game_id();
        let player_id = self.free_player_id();
        let game = Game::new(self, game_id, player_id);
        self.player_id_history.push_back(player_id);
        // Verify active game limit
        if self.player_id_history.len() > MAX_ACTIVE_GAMES {
            // Game limit is reached
            let player_id_to_delete = self.player_id_history.pop_front().unwrap();
            self.delete_game(player_id_to_delete);
        }
        self.games.insert(player_id, game);
        RegisterResult {player_id}
    }

    /// Reads the words file and returns a random word
    fn random_word(&self) -> String {
        let number = rand::thread_rng().gen_range(0..self.words.len());
        let mut transformed_word = String::new();
        for c in self.words[number].clone().chars() {
            match c {
                'Ä' => transformed_word.push_str("AE"),
                'Ö' => transformed_word.push_str("OE"),
                'Ü' => transformed_word.push_str("UE"),
                _ => transformed_word.push(c),
            }
        }
        transformed_word
    }
   
    /// # Returns
    /// 
    /// `Some(&mut Game)` when the game was found where the user is playing in
    /// 
    /// `None` the player id does not appear to be assigned to a game
    pub fn game_by_player_id(&mut self, id: Uuid) -> Option<&mut Game> {
        self.games.get_mut(&id)
    }

    /// Deletes the game for the specified user.
    /// 
    /// This will also delete all users that are assigned to that game and free the user ids. 
    /// This means that the user_id no longer recognized by the server and requests that require a user_id to be set will fail with a 403 http response.
    /// # Returns
    /// `true` game was deleted
    /// 
    /// `false` no game found for user
    pub fn delete_game(&mut self, id: Uuid) -> bool {
        if let Some(game) = self.games.remove(&id) {
            self.game_ids.remove(&game.game_id());
            self.player_ids.remove(&id);
            return true
        }
        false
    }

    /// Returns a free game id. The returned game id is placed in the `game_ids` set.
    fn free_game_id(&mut self) -> Uuid {
        let mut game_id = Uuid::new_v4();
        loop {
            if self.game_ids.insert(game_id) {
                break;
            }
            game_id = Uuid::new_v4();
        }
        game_id
    }

    /// Returns a free player id. The returned player id is placed in the 'player_ids' set.
    fn free_player_id(&mut self) -> Uuid {
        let mut player_id = Uuid::new_v4();
        loop {
            if self.player_ids.insert(player_id) {
                break;
            }
            player_id = Uuid::new_v4();
        }
        player_id
    }
}

/// Used to represent a result that occurs when [register_game](struct.GameManager.html#method.register_game) is called.
pub struct RegisterResult {
    /// The id of the new player
    pub player_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::{GameManager, MAX_ACTIVE_GAMES};


    #[test]
    fn test_max_game_limit() {
        let mut game_manager = GameManager::new();
        let first_uuid = game_manager.register_game().player_id;
        for _i in 1..=MAX_ACTIVE_GAMES {
            game_manager.register_game();
        }
        let last_uuid = game_manager.register_game().player_id;
        assert!(game_manager.game_by_player_id(first_uuid).is_none());
        assert!(game_manager.game_by_player_id(last_uuid).is_some());
    }
}