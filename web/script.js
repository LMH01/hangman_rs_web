var eSource;
var username;
var yourTurn;
var playerTurnPosition;
var gameId;

async function updateWord() {
  var response = await fetchData('api/game_string');
  document.getElementById("gameword").innerHTML = response;
}

async function updateLives() {
  var response = await fetchData('api/lives');
  document.getElementById("lives").innerHTML = 'lives: ' + response;
}

async function register() {
  username = document.getElementById("input-username").value;
  if (username == "") {
    alert("Please enter a username");
    return;
  }
  
  var response = await postData('api/register', { username: document.getElementById("input-username").value });
  console.log(response);
  
  if (response.result == 2) {
    playerTurnPosition = 0;

    document.getElementById("login").hidden = true;
    loggedIn();
  }
  if (response.result == "3") {
    playerTurnPosition = 1;

    document.getElementById("login").hidden = true;
    document.getElementById("turn").innerHTML = "Game started, its the other Players turn.";
    fetchTeammates();
    startGame();
  }

  gameId = response.game_id;
  subscribeEvents(gameId);
}

async function updateImage() {
  var response = await fetchData('api/lives');
  document.getElementById("image").hidden = false;
  document.getElementById("image").src = "pictures/" + Math.abs(10 - Number(response)) + ".jpg"
}

async function updateGuessedLetters() {
  var response = await fetchData('api/guessed_letters');
  document.getElementById("guessed_letters").innerHTML = 'Guessed letters: ' + response;
}

async function fetchTeammates() {
  var response = await fetchData('api/teammates');
  document.getElementById("teammates").innerHTML = 'Teammate: ' + response;
}

function loggedIn() {
  document.getElementById("login").hidden = true;
  document.getElementById("waitingroom").hidden = false;
}

function startGame() {
  console.log("You are " + playerTurnPosition);
  updateLives();
  updateWord();
  updateImage();
  document.getElementById("waitingroom").hidden = true;
  document.getElementById("game").hidden = false;
}

async function gameInput() {

  console.log(document.getElementById("input-letter").value);

  if (!(document.getElementById("input-letter").value.length === 1) || !(document.getElementById("input-letter").value.match(/[a-zA-Z]/))) {
    alert("Input only one character");
    document.getElementById("input-letter").value = "";
    return;
  }
  console.log("Input: " + $("#input-letter").val());
  console.log("Uppercase: " + $("#input-letter").val().toUpperCase());
  console.log();
  if (($("#gameword").text().indexOf($("#input-letter").val().toUpperCase())) != -1) {
    alert("This character was already submitted");
    document.getElementById("input-letter").value = "";
    return;
  }
  document.getElementById("turn").value = "Wait for your turn";

  var response = await postData('api/submit_char', { character: document.getElementById("input-letter").value[0] });
  console.log(response)
  switch (response) {
    case 1: updateImage();
      gameEnd(true);
      break;
    case 2: updateImage();
      yourTurn = false;
      updateWord();
      break;
    case 3: updateImage();
      yourTurn = false;
      updateLives();
      break;
    case 4: updateImage();
      gameEnd(false);
      break;
    case 5:
      alert("Not your Turn. Please wait");
      break;
  }

  updateGuessedLetters();
  document.getElementById("input-letter").value = "";
}

function myturn(number) {

  console.log("turn:" + number);
  console.log("playernumber:" + playerTurnPosition);
  if (number == playerTurnPosition.toString()) {
    console.log("MEEE");
    return true;
  }
  else
    return false;
}

async function gameEnd(won) {
  updateWord();
  document.getElementById("game").hidden = true;
  document.getElementById("gameend").hidden = false;
  document.getElementById("wonlost").innerHTML = "Game Over";
  if (won) {
    document.getElementById("wonlost").innerHTML = "You won";
  } else {
    updateImage();
    document.getElementById("wonlost").innerHTML = "You lost";
  }
  
  var response = await fetchData('api/word');
  console.log("Response was: " + response);
  document.getElementById("wordwas").innerHTML = "The Word was: " + response;
}

async function deleteGame() {
  var response = await fetchData('api/delete_game');
  console.log(response);
  location.reload();
}

// Reloads required elements when the player reloads the page and a game is running
async function reconnect() {
  fetchTeammates();
  updateGuessedLetters();
  var isPlayersTurn = await fetchData('api/is_players_turn');
  switch (isPlayersTurn) {
    case 'true':
      document.getElementById("turn").innerHTML = "Its your turn.";
      break;
    case 'false':
      document.getElementById("turn").innerHTML = "Its the other players turn.";
      break;
  }
  gameId = await fetchData('api/game_id');
  playerTurnPosition = await fetchData('api/player_turn_position');
  startGame();
  subscribeEvents(gameId);
}

// Subscribes to the event listener at /sse
function subscribeEvents(gameId = '') {
  function connect() {
    const events = new EventSource("/sse/" + gameId);

    events.addEventListener("message", (env) => {
      var data = env.data;
      console.log("received data: " + JSON.stringify(data));
      console.log("decoded data", JSON.stringify(JSON.parse(data)));
      var msg = JSON.parse(data);
      switch (msg.data) {
        case "game_start": 
          document.getElementById("turn").innerHTML = "Game started, its your turn.";
          fetchTeammates();
          startGame();
          break;
        case "solved":    
          console.info("Game has ended, closing event stream for /sse/" + gameId);
          events.close();
          gameEnd(true);
          break;
        case "lost":
          gameEnd(false);
          break;
        case "letter_correct":
          updateWord();
          updateLives();
          updateImage();
          updateGuessedLetters();
          if (msg.player == playerTurnPosition) {
            document.getElementById("turn").innerHTML = "Your turn! Type one letter. The other Players guess was right.";
          } else          startGame();
            document.getElementById("turn").innerHTML = "Well done, that was correct! Now its the other Players turn.";
          break;
        case "letter_false":
          updateLives();
          updateWord();
          updateImage();
          updateGuessedLetters();
          if (msg.player == playerTurnPosition) {
            document.getElementById("turn").innerHTML = "Your turn! Type one letter. The other Players guess was wrong.";
          } else
            document.getElementById("turn").innerHTML = "Nice try, but that was wrong. Now its the other players turn.";
          break;
      }
    });

    events.addEventListener("open", () => {
      console.log(`connected to event stream at /sse/` + gameId);
    });

    events.addEventListener("error", () => {
      console.error("connection to event stream at /sse/" + gameId + " lost");
      console.info("Closing event stream for /sse/" + gameId);
      events.close();
    });
  }

  connect();
}

// Example POST method implementation:
// Copied from https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch
async function postData(url = '', data = {}) {
  // Default options are marked with *
  const response = await fetch(url, {
    method: 'POST', // *GET, POST, PUT, DELETE, etc.
    mode: 'cors', // no-cors, *cors, same-origin
    cache: 'no-cache', // *default, no-cache, reload, force-cache, only-if-cached
    credentials: 'same-origin', // include, *same-origin, omit
    headers: {
      'Content-Type': 'application/json'
      // 'Content-Type': 'application/x-www-form-urlencoded',
    },
    redirect: 'follow', // manual, *follow, error
    referrerPolicy: 'no-referrer', // no-referrer, *no-referrer-when-downgrade, origin, origin-when-cross-origin, same-origin, strict-origin, strict-origin-when-cross-origin, unsafe-url
    body: JSON.stringify(data) // body data type must match "Content-Type" header
  });
  return response.json(); // parses JSON response into native JavaScript objects
}

// Submits a get request to the url
async function fetchData(url = '') {
  var word = await (await(fetch(url, {}))).text();
  return word;
}

function getCookie(name) {
  const value = `; ${document.cookie}`;
  const parts = value.split(`; ${name}=`);
  if (parts.length === 2) return parts.pop().split(';').shift();
}

//MAIN
$(document).ready(async function () {
  //Login Logic
  //get username from cookie
  let username = getCookie("userid");
  console.log(login);
  if (username == "" || username == null) {
    document.getElementById("login").hidden = false;
  } else {
    var result = await fetchData('api/registered');
    console.log('Checking registration status: ' + result);
    switch (result) {
        case 'false':
          document.getElementById("login").hidden = false;
          break;
        case 'registered':
          loggedIn();
          break;
        case 'playing':
          reconnect();  
          break;
        case 'win':
          gameEnd(true);
          break;
        case 'lost':
          gameEnd(false);
          break;
    }
  }
});
