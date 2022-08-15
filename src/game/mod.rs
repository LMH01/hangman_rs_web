use std::fs;
use rand::Rng;
use rocket::{log::private::info, State, tokio::sync::broadcast::Sender};

use crate::EventData;

use self::base_game::{Game, Player};

/// Contains all base components that are required to run a game
pub mod base_game;

/// Determines how many lives players have when playing a game.
/// 
/// Should not be set higher than 10 because images will fail to load.
const MAX_LIVES: i32 = 10;

/// Used to manage all currently running games.
/// 
/// One `GameManager` instance is managed by rocket and given to each request handler.
pub struct GameManager {
    /// Contains all games that are currently running
    games: Vec<Game>,
    /// All words from which a random word can be chosen for a game
    words: Vec<String>,
    /// All player ids that are already in use. 
    /// 
    /// A player id uniquely identifies the given player. 
    /// 
    /// It is also used to authorize the player against the server.
    player_ids: Vec<i32>,
    /// The current open game where a new player is assigned to
    current_open_game: Option<Game>,
    /// The currently highest numbered game id.
    max_game_id: i32,
}

impl GameManager {
    /// Create a new `GameManager`
    pub fn new() -> Self {
        let file = fs::read_to_string("words.txt").expect("Unable to read words file!");
        let words: Vec<String> = file.split('\n').map(|s| String::from(s).to_uppercase()).collect();    
        Self {
            games: Vec::new(),
            words,
            player_ids: Vec::new(), 
            current_open_game: None,
            max_game_id: 0,
        }
    }

    /// Registers a new game
    /// # Params
    /// `name` the name of the player that registers the new game
    /// # Returns
    /// [RegisterResult](struct.RegisterResult.html) the result of the registration
    pub fn register_game(&mut self, name: String, event: &State<Sender<EventData>>) -> RegisterResult {
        // Determine if a game is already open or if a new one should be created
        let mut new_game = false;
        let mut game = match self.current_open_game.take() {
            Some(game) => {
                info!("Starting new game.");
                let _e = event.send(EventData::new(0, game.game_id(), String::from("game_start")));
                game
            },
            None => {
                info!("Creating new game");
                let new_game_id = self.free_game_id();
                let game = Game::new(self, new_game_id);
                new_game = true;
                game  
            }
        };
        // Add player
        let player_id = self.free_player_id();
        self.player_ids.push(player_id);
        game.add_player(Player::new(player_id, name, self.player_ids.len()-1));
        // Transmit result
        let game_id = game.game_id();
        if new_game {
            self.current_open_game = Some(game);
            RegisterResult { player_id, result_id: 2, game_id}
        } else {    
            self.games.push(game);
            RegisterResult { player_id, result_id: 3, game_id}
        }
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

    /// Returns a free player id
    fn free_player_id(&self) -> i32 {
        let mut number = rand::thread_rng().gen_range(0..=i32::MAX);
        while self.player_ids.contains(&number) {
            number = rand::thread_rng().gen_range(0..=i32::MAX);
        }
        number
    }
   
    /// # Returns
    /// 
    /// `Some(&mut Game)` when the game was found where the user is playing in
    /// 
    /// `None` the player id does not appear to be assigned to a game
    pub fn game_by_player_id(&mut self, id: i32) -> Option<&mut Game> {
        for game in &mut self.games {
            for player in game.players() {
                if player.id == id {
                    return Some(game);
                }
            }            
        }
        None
    }

    /// Deletes the game for the specified user.
    /// 
    /// This will also delete all users that are assigned to that game and free the user ids. 
    /// This means that the user_is no longer recognized by the server and requests that require a user_id to be set will fail with a 403 http response.
    /// # Returns
    /// `true` game was deleted
    /// 
    /// `false` no game found for user
    pub fn delete_game(&mut self, id: i32) -> bool {
        for (index, game) in &mut self.games.iter().enumerate() {
            for player in game.players() {
                if player.id == id {
                    let mut player_ids = Vec::new();
                    for delete_player in game.players() {
                        player_ids.push(delete_player.id);
                    }
                    self.clear_player_ids(player_ids);
                    self.games.remove(index);
                    return true;
                }
            }
        }
        false
    }

    /// Removes the player ids from the `player_ids` vector.
    fn clear_player_ids(&mut self, ids: Vec<i32>) {
        let mut indices = Vec::new();
        for (index, e_id) in self.player_ids.iter().enumerate() {
            if ids.contains(e_id) {
                indices.push(index);
            }
        }
        indices.reverse();
        for index in indices {
            self.player_ids.remove(index);
        }
    }

    /// Returns a free game id and increments the max game id value
    fn free_game_id(&mut self) -> i32 {
        self.max_game_id += 1;
        self.max_game_id
    }

    /// Checks if the `id` has been assigned to a player
    /// # Returns
    /// `true` id is assigned to user
    /// 
    /// `false` id is free
    pub fn id_taken(&self, id: i32) -> bool {
        self.player_ids.contains(&id)
    }
}

/// Used to represent a result that occurs when [register_game](struct.GameManager.html#method.register_game) is called.
pub struct RegisterResult {
    /// The id of the new player
    pub player_id: i32,
    /// The result that should be sent back to the client
    /// 
    /// Is one of the following:
    /// 
    /// '2' - player has been added to an existing game and game starts
    /// 
    /// '3' - new game has been created and player is waiting for second player
    pub result_id: i32,
    /// The game id to which the user is registered
    pub game_id: i32,
}
