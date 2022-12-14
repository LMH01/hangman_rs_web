# Hangman_rs_web

As of 29.12.2022 the multiplayer part has been removed from the game. It will probably be added back once the new singeplayer mode works.
To play in multiplayer mode and view more information about my university project please checkout the branch [uni-state](https://github.com/LMH01/hangman_rs_web/tree/uni-state).
This branch contains the state in which the project was in when I initially reworked the project from university to use a rust server.

User authentication is done by using cookies that store a unique user id.

When the page is reloaded while in a game the game state is restored.

## Building and running
To build and run the server do the following:

1. Clone the repository and cd into the main directory
2. Make sure that the `rust toolchain` and `wasm-pack` are installed
3. Run `./build_wasm.sh`
4. Run `cargo run`

This will start the server which can be accessed under `127.0.0.1:11511`.

## WebAssembly
WebAssembly is used to write as little JavaScript as possible. The Rust code that is compiled to WebAssembly can be found [here](wasm/).

## REST API

The communication between server and web browser is realized by a REST api, these are the available endpoints:

### Note: All endpoints except `/api/register`, `/api/registered` and `/singleplayer` can only be accessed when a valid `uuid` cookie is set.

Path|Parameters|Return|Description
-|-|-|-
/singleplayer| - |singleplayer html page|Returns the html page for singleplayer mode
/api/register|username|RegistrationData|Registers a player to the server
/api/guess| string | integer in range 1-5|Submits a character for the game
/api/lives| - | string | The number of lives left
/api/game_string| - | string | The game string
/api/word| - | string | The correct word once the game has ended
/api/delete_game| - | string | Deletes the game the user is playing in
/api/guessed_letters| - | string | All already guessed letters
/api/teammates| - | string | Names of the teammates
/api/game_id| - | string | The id of the game where the player is playing in
/api/registered| - | String | Checks if the user is registered to a game

### For a more detailed explanation see the documentation that can be build by running `cargo doc`.

## Rocket
This project uses the [web framework rocket](https://github.com/SergioBenitez/Rocket) to realize the server.