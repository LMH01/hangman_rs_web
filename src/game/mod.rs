use self::words::random_word;

mod words;

struct Game {
    players: Vec<Player>,
    word: Word,
}

impl Game {
    /// Construct a new game with a random word
    fn new() -> Self {
        Self {
            players: Vec::new(),
            word: Word::new(&random_word()),
        }
    }

    /// Adds the player to the game
    fn add_player(&mut self, player: Player) {
        self.players.push(player);
    }
}

struct Player {
    id: i32,
    name: String,
}

struct Word {
    letters: Vec<Letter>,
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