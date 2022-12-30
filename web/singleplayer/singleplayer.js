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
        await updatePage();
    } else {
        // some cookie exists, check if cookie is valid
        let status = await (wasm_bindgen.get_request("api/registered")) + '';
        console.log(status);
        switch (status) {
            case 'false':// uuid is invalid
                await register();            
                await updatePage();
                break;
            case 'playing':// player is still playing the game
                updatePage();
                break;
            case 'won':// player has won the game
                break;
            case 'lost':// player has lost the game
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
    updateImage();
}

/**
 * Update the games word
 */
async function updateWord() {
    let word = await (wasm_bindgen.get_request("api/game_string"));
    document.getElementById("word").innerHTML = word;
    document.getElementById("word-placeholder").hidden = true;
    document.getElementById("word").hidden = false;
}

/**
 * Update the guessed characters
 */
async function updateGuessedChars() {

}

/**
 * Update the lives
 */
async function updateLives() {

}

/**
 * Update the image
 */
async function updateImage() {

}