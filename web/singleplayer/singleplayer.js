document.addEventListener("DOMContentLoaded", async function(){
    console.info("Initializing wasm");
    await wasm_bindgen('../wasm/hangman_rs_wasm_bg.wasm');
    wasm_bindgen.init();
    preparePage();
    //let y = await (wasm_bindgen.get_request("api/lives"));
    //console.log(y);
});

async function debug() {
    //let z = await (wasm_bindgen.post_request_data("api/submit_char", { character: document.getElementById("user-input").value[0] }));
    let z = await (postData("api/submit_char", { character: document.getElementById("user-input").value[0] }));
    console.log(z);
}

/**
 * Prepares the singleplayer page by doing the following:
 * - Check uuid and reconstruct page state if uuid exists and is valid
 * - Register new user session and set page state
 */
async function preparePage() {
    // Check uuid
    let uuid = getCookie("uuid");
    if (uuid == "" || uuid == null) {
        await register();
        updatePage();
    } else {
        // some cookie exists, check if cookie is valid
        let status = await (wasm_bindgen.get_request("api/registered")) + '';
        switch (status) {
            case 'false':// uuid is invalid
                await register();            
                updatePage();
                break;
            case 'playing':// player is still playing the game
                updatePage();
                break;
            case 'won':// player has won the game
                updatePage();
                gameEnd(true);
                break;
            case 'lost':// player has lost the game
                updatePage();
                gameEnd(false);
                break;
        }
    }
}

/**
 * Registeres with the server, this will always set a new uuid
 */
async function register() {
    uuid = await (wasm_bindgen.post_request("api/register"));
    console.log('uuid: ' + uuid);
}

/**
 * Updates all pages elements and hides the placeholders.
 */
async function updatePage() {
    updateWord();
    updateGuessedChars();
    updateLives();
}

/**
 * Update the games word
 */
async function updateWord() {
    let word = await (wasm_bindgen.get_request("api/game_string"));
    document.getElementById("word").innerHTML = word;
    document.getElementById("word").hidden = false;
    document.getElementById("word-placeholder").hidden = true;
}

/**
 * Update the guessed characters
 */
async function updateGuessedChars() {
    let guessed_chars = await (wasm_bindgen.get_request("api/guessed_letters"));
    document.getElementById("guessed-letters").innerHTML = guessed_chars;
    document.getElementById("guessed-letters").hidden = false;
    document.getElementById("guessed-letters-placeholder").hidden = true;
}

/**
 * Update the lives.
 * Automatically updates the image.
 */
async function updateLives() {
    let lives = await (wasm_bindgen.get_request("api/lives"));
    document.getElementById("lives-left").innerHTML = lives;
    document.getElementById("lives-left").hidden = false;
    document.getElementById("lives-left-placeholder").hidden = true;
    updateImage(lives);
}

/**
 * Update the image
 * @param {int} lives_left - The amount of lives left
 */
async function updateImage(lives_left) {
    document.getElementById("image").src = "pictures/" + Math.abs(10 - Number(lives_left)) + ".jpg"
    document.getElementById("image").hidden = false;
    document.getElementById("image-placeholder").hidden = true;
}

/**
 * Sends the content of the text field to the server to make a guess
 */
async function guess() {
    var response = await postData('api/guess', document.getElementById("user-input").value);
    console.log(response);
    switch (response) {
      case 1: 
        updatePage();
        gameEnd(true);
        break;
      case 2: 
        updatePage();
        break;
      case 3: 
        updatePage();
        break;
      case 4: 
        updatePage();
        gameEnd(false);
        break;
      case 5:
        alert("This character was already submitted");
        document.getElementById("user-input").value = "";
        break;
    }
    document.getElementById("user-input").value = "";
}

async function gameEnd(status) {
    document.getElementById("input-container").hidden = true;
    if (status) {
        document.getElementById("game-won-container").hidden = false;
        document.getElementById("game-lost-container").hidden = true;
    } else {
        document.getElementById("game-won-container").hidden = true;
        document.getElementById("game-lost-container").hidden = false;
        let word = await (wasm_bindgen.get_request("api/word"));
        document.getElementById("word").innerHTML = word;
    }
    document.getElementById("game-over-container").hidden = false;
}