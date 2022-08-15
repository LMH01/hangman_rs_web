# Hangman_rs_web

This is my try at rewriting a project from university to use a rust server.
This project originally used a java server.

Most of the code for the website was copied from the university project.

User authentication is done by using cookies that store a unique user id.

When the page is reloaded while in a game the game state is restored.

# REST API

The communication between server and web browser is realized by a REST api, these are the available endpoints:

### Note: All endpoints except `/api/register` and `/api/registered` can only be accessed when a valid `userid` cookie is set.

Path|Parameters|Return|Description
-|-|-|-
/api/register|username|RegistrationData|Registers a player to the server
/api/submit_char| character | integer in range 1-5|Submits a character for the game
/api/lives| - | string | The number of lives left
/api/game_string| - | string | The game string
/api/word| - | string | The correct word once the game has ended
/api/delete_game| - | string | Deletes the game the user is playing in
/api/guessed_letters| - | string | All already guessed letters
/api/teammates| - | string | Names of the teammates
/api/is_players_turn| - | boolean | Checks if it is the players turn
/api/game_id| - | string | The id of the game where the player is playing in
/api/player_turn_position| - | string | The players position in the turn order
/api/registered| - | String | Checks if the user is registered to a game
/sse/<game_id>|-|-|Server Side Events

### For a more detailed explanation see the documentation that can be build by running `cargo doc`.

## Rocket
This project uses the [web framework rocket](https://github.com/SergioBenitez/Rocket) to realize the server.