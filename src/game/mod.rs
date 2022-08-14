use std::{fs, i32::MAX};
use rand::Rng;
use rocket::{log::private::info, State, tokio::sync::broadcast::Sender};

use crate::EventData;

const MAX_LIVES: i32 = 10;

pub struct GameManager {
    /// Contains all games that are currently running
    games: Vec<Game>,
    /// All words from which a random word can be chosen for the game
    words: Vec<String>,
    /// All player ids that are already in use. A player id uniquely identifies the given player. It is also used to authorize the player against the server.
    player_ids: Vec<i32>,
    /// The current open game where a new player is assigned to
    current_open_game: Option<Game>,
    /// The currently highest numbered game id.
    max_game_id: i32,
}

impl GameManager {
    pub fn new() -> Self {
        let file = fs::read_to_string("words.txt").expect("Unable to read words file!");
        let words: Vec<String> = file.split("\n").map(|s| String::from(s).to_uppercase()).collect();    
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
    /// 'name' the name of the player that registers the new game
    /// # Returns
    /// 'Some<i32>' registration was successful, user id is returned
    /// 'None' registration failed
    /// 
    /// # result_id
    /// '2' player has been added to existing game and game starts
    /// '3' new game has been created and player is waiting for second player
    pub fn register_game(&mut self, name: String, event: &State<Sender<EventData>>) -> RegisterResult {
        // Determine if a game is already open or if a new one should be created
        let mut new_game = false;
        let mut game = match self.current_open_game.take() {
            Some(game) => {
                info!("Starting new game.");
                let _e = event.send(EventData::new(0, game.game_id, String::from("game_start")));
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
        let game_id = game.game_id;
        if new_game {
            self.current_open_game = Some(game);
            RegisterResult { player_id, result_id: 2, game_id: game_id}
        } else {    
            self.games.push(game);
            RegisterResult { player_id, result_id: 3, game_id: game_id}
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
    /// 'Some(&mut Game)' when the game was found where the user is playing in
    /// 
    /// 'None' the player id does not appear to be assigned to a game
    pub fn game_by_player_id(&mut self, id: i32) -> Option<&mut Game> {
        for game in &mut self.games {
            for player in &game.players {
                if player.id == id {
                    return Some(game);
                }
            }            
        }
        None
    }

    /// Deletes the game for the specified user.
    /// This will also delete all users that are assigned to that game and free the user ids.
    /// # Returns
    /// 'true' game was deleted
    /// 'false' no game found for user
    pub fn delete_game(&mut self, id: i32) -> bool {
        for (index, game) in &mut self.games.iter().enumerate() {
            for player in &game.players {
                if player.id == id {
                    let mut player_ids = Vec::new();
                    for delete_player in &game.players {
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
    /// `false` id is free
    pub fn id_taken(&self, id: i32) -> bool {
        if self.player_ids.contains(&id) {
            true 
        } else {
            false
        }
    }
}

/// Used to represent a result that occurs when [register_game](struct.GameManager.html#method.register_game) is called.
pub struct RegisterResult {
    /// The id of the new player
    pub player_id: i32,
    /// The result that should be sent back to the client
    pub result_id: i32,
    /// The game id to which the user is registered
    pub game_id: i32,
}

enum GameState {
    /// Symbolizes that the game has not yet started
    Waiting,
    /// Symbolizes that this game is over. Boolean value determines if the game was won (`true`) or lost (`false`).
    Done(bool),// Add state to done that signifies if the game was won or lost: DONE(boolean)
}

pub struct Game {
    /// The players that are assigned to the game
    players: Vec<Player>,
    /// The word that should be guessed
    word: Word,
    /// The index of the current player
    current_player: usize,
    /// Stores the state of the game. When the game is started no more players can be added
    game_state: GameState,
    /// Stores the lives left
    lives: i32,
    /// The id of this game. Used to determine to whom server send events should be sent
    game_id: i32,
    /// All letters that where guessed
    guessed_letters: Vec<Letter>,
}

impl Game {
    /// Construct a new game with a random word
    fn new(game_manager: &GameManager, game_id: i32) -> Self {
        let mut guessed_letters = Vec::new();
        for c in b'a'..=b'z' {
            guessed_letters.push(Letter::new((c as char).to_uppercase().to_string().chars().next().unwrap()));
        }
        Self {
            players: Vec::new(),
            word: Word::new(&game_manager.random_word()),
            current_player: 0,
            game_state: GameState::Waiting,
            lives: MAX_LIVES,
            game_id,
            guessed_letters,
        }
    }

    /// Adds the player to the game.
    /// # Returns
    /// 'true' when the player was added
    /// 'false' when the player was not added because the game was already started
    fn add_player(&mut self, player: Player) -> bool { //I know that i should probably use an result for this use case
        match self.game_state {
            GameState::Waiting => {
                self.players.push(player);
                true
            },
            _ => false,
        }
    }

    /// Returns the current word in the following formatting:
    /// If no letters are guessed:   _____
    /// If some letters are guessed: _E___
    /// If all letters are guessed:  HELLO
    /// If the game is lost the whole word is returned;
    pub fn game_string(&self) -> String {
        let mut string = String::new();
        let mut first_letter = true;
        for letter in &self.word.letters {
            if !first_letter {
                string.push(' ');
            } else {
                first_letter = false;
            }
            match letter.guessed {
                true => string.push(letter.character),
                false => string.push('_'),
            };
        }
        string
    }

    /// This function can be used to retrieve the word without the white spaces after the word was guessed or the game has ended.
    /// # Returns
    /// 'Option(String)' when the word was guessed correctly, string is the word
    /// 'None' when the word is not yet guessed
    pub fn word(&self) ->  Option<String> {
        match self.game_state {
            GameState::Done(_win) => Some(self.word.get()),
            _ => None,
        }
    }

    /// Guesses a letter and returns a number to indicate that status:
    /// # Returns
    /// '1' letter was correct and the word is guessed completely
    /// '2' letter was correct
    /// '3' letter was false
    /// '4' letter was false and all lives are gone
    /// '5' letter was not guessed because it is not the players turn
    pub fn guess_letter(&mut self, user_id: i32, c: char, event: &State<Sender<EventData>>) -> i32 {
        let c = c.to_uppercase().to_string().chars().next().unwrap();
        let next_player = match self.players.len()-1 == self.current_player {
            true => 0,
            false => self.current_player + 1,
        };
        // check if user is current player
        if self.players[self.current_player].id == user_id {
            match self.players.len()-1 == self.current_player {
                true => self.current_player = 0,
                false => self.current_player += 1,
            } 
        } else {
            return 5;
        }
        // Update guessed letters vector
        self.add_letter_guessed(c); 
        // guess letters
        let mut something_guessed = false;
        for letter in &mut self.word.letters {
            if letter.character == c {
                letter.guessed = true;
                something_guessed = true;
            }
        }
        if something_guessed && !self.solved() {
            let _x = event.send(EventData::new(next_player, self.game_id, String::from("letter_correct")));
            return 2;
        }
        // check lives
        self.lives -= 1;
        if self.lives == 0 || self.solved() {
            if self.solved() {
                self.lives += 1; //Increment lives to get the amount of lives that where left when the game was won
                let _x = event.send(EventData::new(0, self.game_id, String::from("solved")));
                self.game_state = GameState::Done(true);
                return 1;
            } else if self.lives == 0 {
                let _x = event.send(EventData::new(0, self.game_id, String::from("lost")));
                self.game_state = GameState::Done(false);
                return 4;
            }
        }
        let _x = event.send(EventData::new(next_player, self.game_id, String::from("letter_false")));
        3
    }

    /// Adds the input letter to the list of guessed characters
    fn add_letter_guessed(&mut self, c: char) {
        for letter in &mut self.guessed_letters {
            if letter.character == c {
                letter.guessed = true;
            }
        }
    }

    /// Returns a string containing all guessed letters.
    /// 
    /// Output may be something like this: `A B D F`
    pub fn guessed_letters(&self) -> String {
        let mut s = String::new();
        let mut first_letter = true;
        for letter in &self.guessed_letters {
            if first_letter {
                first_letter = false;
            } else {
                s.push(' ');
            }
            if letter.guessed {
                s.push(letter.character);
            }
        }
        s
    }

    /// # Returns
    /// 'true' when the word has been guessed successfully
    fn solved(&self) -> bool {
        for letter in &self.word.letters {
            if !letter.guessed {
                return false;
            }
        }
        true
    }

    /// # Returns
    /// 'Some(Player)' when the player was found
    /// 'None' when the player with the id does not exist
    fn player_by_id(&self, id: i32) -> Option<&Player> {
        for player in &self.players {
            if player.id == id {
                return Some(player);
            }
        }
        None
    }

    /// # Returns
    /// How many lives are left
    pub fn lives(&self) -> i32 {
        self.lives
    }

    /// # Returns
    /// The current player index
    pub fn current_player(&self) -> usize {
        self.players[self.current_player].turn_position
    }

    /// # Returns
    /// The game id of this game
    pub fn game_id(&self) -> i32 {
        self.game_id
    }

    /// Returns the names of the teammates of the player with the id
    pub fn teammates(&self, player_id: i32) -> String {
        let mut s = String::new();
        let mut first_player = true;
        for player in &self.players {
            if player.id != player_id {
                if first_player {
                    first_player = false;
                } else {
                    s.push_str(", ");
                }
                s.push_str(&player.name);
            }
        }
        s
    }

    /// Checks if it is the players turn
    pub fn is_players_turn(&self, player_id: i32) -> bool {
        self.players[self.current_player].id == player_id
    }

    /// # Return
    /// `Some(usize)` the position of the player in the turn order
    /// `None` the player with the id was not found
    pub fn player_turn_position(&self, player_id: i32) -> Option<usize> {
        match self.player_by_id(player_id) {
            Some(player) => return Some(player.turn_position),
            None => None,
        }
    }

    /// Checks if the game has been completed
    /// # Returns
    /// 'None' when the game is still running
    /// 'Some(bool)' when the game has been completed. Boolean indicates if the game was won (`true`) or lost (`false`).
    /// `true` when the game has been completed
    /// `false` when the game is still running
    pub fn completed(&self) -> Option<bool> {
        match self.game_state {
            GameState::Done(win) => Some(win),
            _ => None,
        }
    }
}

#[derive(PartialEq)]
pub struct Player {
    /// Unique number with which the player is identified by the server
    pub id: i32,
    /// The place in the turn order for this player
    pub turn_position: usize,
    /// The name of this player
    name: String,
}

impl Player {
    pub fn new(id: i32, name: String, turn_position: usize) -> Self {
        Self { 
            id, 
            name,
            turn_position,
        }
    }
}

struct Word {
    pub letters: Vec<Letter>,
}

impl Word {
    fn new(word: &str) -> Self {
        let mut letters = Vec::new();
        for c in word.chars() {
            letters.push(Letter::new(c));
        }
        Self { 
            letters 
        }
    }

    /// # Return
    /// The word that this type represents
    fn get(&self) -> String {
        let mut s = String::new();
        for l in &self.letters {
            s.push(l.character);
        }
        s
    }
}

struct Letter {
    character: char,
    guessed: bool,
}

impl Letter {
    fn new(character: char) -> Self {
        Self { 
            character, 
            guessed: false 
        }
    }
}