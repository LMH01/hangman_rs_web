use std::{fs, i32::MAX};
use rand::Rng;
use rocket::log::private::info;

const MAX_LIVES: i32 = 10;

pub struct GameManager {
    /// Contains all games that are currently running
    games: Vec<Game>,
    /// All words from wich a random word can be choosen for the game
    words: Vec<String>,
    /// All player ids that are already in use
    player_ids: Vec<i32>,
    /// The current open game where a new player is assigned to
    current_open_game: Option<Game>,
}

impl GameManager {
    pub fn new() -> Self {
        let file = fs::read_to_string("words.txt").expect("Unable to read words file!");
        let words: Vec<String> = file.split("\n").map(String::from).collect();    
        Self {
            games: Vec::new(),
            words,
            player_ids: Vec::new(), 
            current_open_game: None,
        }
    }

    /// Registers a new game
    /// # Params
    /// 'name' the name of the player that registers the new game
    /// # Returns
    /// 'Some<i32>' registration was successfull, user id is returned
    /// 'None' registration failed
    /// 
    /// # result_id
    /// '2' player has been added to existing game and game starts
    /// '3' new game has been created and player is waiting for second player
    pub fn register_game(&mut self, name: String) -> RegisterResult {
        let mut new_game = false;
        let mut game = match self.current_open_game.take() {
            Some(game) => {
                info!("Starting new game.");
                game
            },
            None => {
                info!("Creating new game");
                let game = Game::new(self);
                new_game = true;
                //TODO SSE-Magic "p2ready" "hello there"
                game  
            }
        };
        let player_id = self.free_player_id();
        game.add_player(Player::new(player_id, name));
        self.player_ids.push(player_id);
        if new_game {
            self.current_open_game = Some(game);
            RegisterResult { player_id, result_id: 2}
        } else {    
            self.games.push(game);
            RegisterResult { player_id, result_id: 3}
            //TODO SSE-Magic "p2ready" "hello there"
        }
    }

    /// Reads the words file and returns a random word
    fn random_word(&self) -> String {
        let number = rand::thread_rng().gen_range(0..self.words.len());
        String::from(self.words[number].clone())
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
}

/// Used to represent a result that occurs when [register_game](struct.GameManager.html#method.register_game) is called.
pub struct RegisterResult {
    /// The id of the new player
    pub player_id: i32,
    /// The result that should be sent back to the client
    pub result_id: i32,    
}

enum GameState {
    WATING,
    STARTING,
    RUNNING,
    DONE,
}

pub struct Game {
    players: Vec<Player>,
    word: Word,
    current_player: usize,
    
    /// Stores the state of the game. When the game is started no more players can be added
    game_state: GameState,

    /// Stores the lives left
    lives: i32,
}

impl Game {
    /// Construct a new game with a random word
    fn new(game_manager: &GameManager) -> Self {
        Self {
            players: Vec::new(),
            word: Word::new(&game_manager.random_word()),
            current_player: 0,
            game_state: GameState::WATING,
            lives: MAX_LIVES,
        }
    }

    /// Adds the player to the game.
    /// # Returns
    /// 'true' when the player was added
    /// 'false' when the player was not added because the game was already started
    fn add_player(&mut self, player: Player) -> bool { //I know that i should probably use an result for this usecase
        match self.game_state {
            GameState::WATING => {
                self.players.push(player);
                true
            },
            _ => false,
        }
    }

    /// Starts the game
    fn start(&mut self) {
        self.game_state = GameState::STARTING;
    }

    /// Returns the current word in the following formatting:
    /// If no letters are guessed:   _____
    /// If some letters are guessed: _E___
    /// If all letters are guessed:  HELLO
    /// If the game is lost the whole word is returned;
    pub fn game_string(&self) -> String {
        let mut string = String::new();
        for letter in &self.word.letters {
            match letter.guessed {
                true => string.push(letter.character),
                false => string.push('_'),
            };
        }
        string
    }

    /// Guesses a letter and returns a number to indicate that status:
    /// # Returns
    /// '1' letter was correct and the word is guessed completely
    /// '2' letter was correct
    /// '3' letter was false
    /// '4' letter was false and all lives are gone
    /// '5' letter was not guessed because it is not the players turn
    pub fn guess_letter(&mut self, userid: i32, c: char) -> i32 {
        // check if user is current player
        if self.players[self.current_player].id == userid {
            match self.players.len() == self.current_player {
                true => self.current_player += 1,
                false => self.current_player = 0,
            } 
        } else {
            return 5;
        }
        // guess letters
        let mut something_guessed = false;
        for letter in &mut self.word.letters {
            if letter.character == c {
                letter.guessed = true;
                something_guessed = true;
            }
        }
        if something_guessed && self.solved() {
            //TODO SSE-Magic to send "p2played" 2
            return 2;
        }
        // check lives
        self.lives -= 1;
        if self.lives == 0 || self.solved() {
            self.game_state = GameState::DONE;
            if self.solved() {
                //TODO SSE-Magic to send "p2played" "1"
                return 1;
            } else if self.lives == 0 {
                //TODO SEE-Magic to send "p2played" "4"
                return 4;
            }
        }
        // SSE-Magic to send "p2played" "3"
        3
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
    /// How many lives are left
    pub fn lives(&self) -> i32 {
        self.lives
    }
}

pub struct Player {
    pub id: i32,
    name: String,
}

impl Player {
    pub fn new(id: i32, name: String) -> Self {
        Self { 
            id, 
            name, 
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

#[cfg(test)]
mod tests {
    use super::GameManager;


    #[test]
    fn game_by_player_id_works() {
        let mut game_manager = GameManager::new();
        game_manager.register_game(String::from("Player1"));
        game_manager.register_game(String::from("Player2"));
        assert_eq!(1, game_manager.games.len());
        assert_eq!(2, game_manager.player_ids.len());
        let game = &game_manager.games[0];
        assert_eq!(2, game.players.len());
    }
}