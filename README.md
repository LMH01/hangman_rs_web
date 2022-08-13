# Hangman_rs_web

This is my try at rewriting a project from university to use a rust server.
This project originally used a java server.

Most of the code for the website was copied from the university project.

User authentication is done by using cookies that store a unique user id.

# REST API

The communication between server and web browser is realized by a REST api, these are the available endpoints:

### Note: this list is under construction!

Path|Parameters|Return|Description
-|-|-|-
/api/register|username|Integer in range 1-3, containing status|Register a new user
/api/user_exists|username|boolean|Check if user exists
/api/partner_name|username|String if partner exists, false otherwise|Get partner name
/api/game_state|username|String|Get game state
/api/game_string|username|String|Returns the current word as it is guessed currently
/api/submit_char|username, char|Integer in range 1-5, containing status|Submit a character to the game
/api/lives|username|Integer|Get number of lives left
/api/delete_game|username|boolean|Delete game
/sse|-|-|Server Side Events

## Rocket
This project uses the [web framework rocket](https://github.com/SergioBenitez/Rocket) to realize the server.