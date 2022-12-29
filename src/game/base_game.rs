use std::collections::HashMap;

use uuid::Uuid;

use super::{GameManager, MAX_LIVES};

/// Representation of a game
pub struct Game {
    /// The players that are assigned to the game
    players: HashMap<Uuid, Player>,
    /// The word that should be guessed
    word: Word,
    /// Stores the state of the game. When the game is started no more players can be added
    game_state: GameState,
    /// Stores the lives left
    lives: i32,
    /// The id of this game.
    game_id: Uuid,
    /// All letters that where guessed
    guessed_letters: Vec<Letter>,
}

impl Game {
    /// Construct a new game with a random word and an assigned player.
    pub fn new(game_manager: &GameManager, game_id: Uuid, player_id: Uuid) -> Self {
        let mut guessed_letters = Vec::new();
        for c in b'a'..=b'z' {
            guessed_letters.push(Letter::new((c as char).to_uppercase().to_string().chars().next().unwrap()));
        }
        let mut players = HashMap::new();
        players.insert(player_id, Player::new(player_id));
        Self {
            players,
            word: Word::new(&game_manager.random_word()),
            game_state: GameState::Running,
            lives: MAX_LIVES,
            game_id,
            guessed_letters,
        }
    }

    /// Returns the current word in the following formatting:
    /// 
    /// If no letters are guessed:   _____
    /// 
    /// If some letters are guessed: _E___
    /// 
    /// If all letters are guessed:  HELLO
    /// 
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
    /// `Option(String)` when the word was guessed correctly, `string` is the word
    /// 
    /// `None` when the word is not yet guessed
    pub fn word(&self) ->  Option<String> {
        match self.game_state {
            GameState::Done(_win) => Some(self.word.get()),
            _ => None,
        }
    }

    /// Guesses a letter and returns a number to indicate that status
    /// # Returns
    /// `1` when the letter was correct and the word is guessed completely
    /// 
    /// `2` when letter was correct
    /// 
    /// `3` when letter was false
    /// 
    /// `4` when letter was false and all lives are gone
    /// 
    /// '5' when the letter was already guessed
    pub fn guess_letter(&mut self, c: char) -> i32 {
        let c = c.to_uppercase().to_string().chars().next().unwrap();
        // Update guessed letters vector
        if !self.add_letter_guessed(c) {
            return 5
        } 
        // guess letters
        let mut something_guessed = false;
        for letter in &mut self.word.letters {
            if letter.character == c {
                letter.guessed = true;
                something_guessed = true;
            }
        }
        if something_guessed && !self.solved() {
            return 2;
        }
        // check lives
        self.lives -= 1;
        if self.lives == 0 || self.solved() {
            if self.solved() {
                self.lives += 1; //Increment lives to get the amount of lives that where left when the game was won
                self.game_state = GameState::Done(true);
                return 1;
            } else if self.lives == 0 {
                self.game_state = GameState::Done(false);
                return 4;
            }
        }
        3
    }

    /// Adds the input letter to the list of guessed characters
    /// # Returns
    /// - `true` letter was not yet guessed
    /// - `false` letter was already guessed
    fn add_letter_guessed(&mut self, c: char) -> bool {
        for letter in &mut self.guessed_letters {
            if letter.character == c {
                if letter.guessed {
                    return false
                } else {
                    letter.guessed = true;
                    return true
                }
            }
        }
        true
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
    /// `true` when the word has been guessed successfully
    fn solved(&self) -> bool {
        for letter in &self.word.letters {
            if !letter.guessed {
                return false;
            }
        }
        true
    }

    ///// # Returns
    ///// `Some(Player)` when the player was found
    ///// 
    ///// `None` when the player with the id does not exist
    //fn player_by_id(&self, id: Uuid) -> Option<&Player> {
    //    self.players.get(&id)
    //}

    /// # Returns
    /// How many lives are left
    pub fn lives(&self) -> i32 {
        self.lives
    }

    /// # Returns
    /// The game id of this game
    pub fn game_id(&self) -> Uuid {
        self.game_id
    }

    /// Returns the names of the teammates of the player with the id
    pub fn teammates(&self, player_id: Uuid) -> String {
        let mut s = String::new();
        let mut first_player = true;
        for (k, v) in &self.players {
            if *k != player_id {
                if first_player {
                    first_player = false;
                } else {
                    s.push_str(", ");
                }
                s.push_str(&v.name);
            }
        }
        s
    }

    /// Checks if the game has been completed
    /// # Returns
    /// `None` when the game is still running
    /// 
    /// `Some(bool)` when the game has been completed. Boolean indicates if the game was won (`true`) or lost (`false`).
    /// 
    /// `true` when the game has been completed
    /// 
    /// `false` when the game is still running
    pub fn completed(&self) -> Option<bool> {
        match self.game_state {
            GameState::Done(win) => Some(win),
            _ => None,
        }
    }
}

/// The different states a game can be in
enum GameState {
    /// Symbolizes that the game is running.
    Running,
    /// Symbolizes that this game is over. 
    /// 
    /// Boolean value determines if the game was won (`true`) or lost (`false`).
    Done(bool),// Add state to done that signifies if the game was won or lost: DONE(boolean)
}

/// Player in a game
#[derive(Eq, PartialEq)]
pub struct Player {
    /// Unique number with which the player is identified by the server
    pub id: Uuid,
    /// The name of the player (currently not used)
    pub name: String,
}

impl Player {
    /// Create a new player
    pub fn new(id: Uuid) -> Self {
        Self { 
            id,
            name: String::from("steve"),
        }
    }
}

/// Word that should be guessed
struct Word {
    pub letters: Vec<Letter>,
}

impl Word {
    /// Create a new word
    /// # Params
    /// `word` the word that this type should represent
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

/// Letter in a word
struct Letter {
    /// Character of this letter
    character: char,
    /// If the character has been guessed
    guessed: bool,
}

impl Letter {
    /// Create a new character
    fn new(character: char) -> Self {
        Self { 
            character, 
            guessed: false 
        }
    }
}