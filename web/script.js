var eSource;
var username;
var yourturn;
var playernumber;

async function printWord() {
  var word = await (await
    (fetch("api/game_string", {
      headers: {
        "username": username
      }
    }
    ))).text();
  document.getElementById("gameword").innerHTML = word;
}

async function printLives() {

  var word = await (await
    (fetch("api/lives", {
      headers: {
        "username": username
      }
    }
    ))).text();
  lives = Number(word);
  document.getElementById("lives").innerHTML = "lives:" + word;
}

function getCookie(name) {
  const value = `; ${document.cookie}`;
  const parts = value.split(`; ${name}=`);
  if (parts.length === 2) return parts.pop().split(';').shift();
}

function sse(path) {
  const eventSource = new eventSource("localhost/api/" + path);
  eventSource.addEventListener();
  msg => { return msg; }

}

async function register() {
  username = document.getElementById("input-username").value;
  if (username == "") {
    alert("Please enter a username");
    return;
  }
//  var res = await fetch("api/register", {
//    headers: { "username": username }
//  });
//  var response = await res.text();
  
  var response = await postData('api/register', { username: document.getElementById("input-username").value });
  console.log(response);

  //var response=()
  if (response == "1") {
    alert("Username already exists");
    return;
  }
  //set cookie
  var d = new Date();
  //set cookie
  var d = new Date();
  //expire in 1 hours
  d.setTime(d.getTime() + (1 * /* 24 * */ 60 * 60 * 1000));
  var expires = "expires=" + d.toUTCString();


  if (response == 2) {
    playernumber = 1;

    document.getElementById("login").hidden = true;
    loggedin();
  }
  if (response == "3") {
    playernumber = 2;

    document.getElementById("login").hidden = true;
    startGame();
  }
  //set username in spans

}
async function image() {
  var word = await (await
    (fetch("api/lives", {
      headers: {
        "username": username
      }
    }
    ))).text();
  lives = Number(word);
  document.getElementById("image").hidden = false;
  document.getElementById("image").src = "pictures/" + Math.abs(10 - lives) + ".jpg"
}
function loggedin() {

  document.getElementById("login").hidden = true;
  document.getElementById("waitingroom").hidden = false;
  console.log("logged in");
  //new EventSource('/sse').addEventListener("message", msg => { console.log(msg.data); startGame() });
}

function startGame() {
  console.log("You are " + playernumber);
  printLives();
  printWord();
  image();
  document.getElementById("waitingroom").hidden = true;
  document.getElementById("game").hidden = false;
  //new EventSource('/sse').addEventListener("message", msg => { console.log(msg.data); p2event(msg.data) });
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
    case 1: image();
      gameend(true);
      break;
    case 2: image();
      yourturn = false;
      printWord();
      break;
    case 3: image();
      yourturn = false;
      printLives();
      break;
    case 4: image();
      gameend(false);
      break;
    case 5:
      alert("Not your Turn. Please wait");
      break;

  }

  document.getElementById("input-letter").value = "";
}

function myturn(number) {

  console.log("turn:" + number);
  console.log("playernumber:" + playernumber);
  if (number == playernumber.toString()) {
    console.log("MEEE");
    return true;
  }
  else
    return false;
}

async function gameend(tf) {
  image();
  printWord();
  document.getElementById("game").hidden = true;
  document.getElementById("gameend").hidden = false;
  document.getElementById("wonlost").innerHTML = "Game Over";
  if (tf)
    document.getElementById("wonlost").innerHTML = "You won";
  else
    document.getElementById("wonlost").innerHTML = "You lost";
  
  var response = await fetchData('api/word');
  console.log("Response was: " + response);
  document.getElementById("wordwas").innerHTML = "The Word was: " + response;
}

// Subscribes to the event listener at /sse
function subscribeEvents() {
  function connect() {
    const events = new EventSource("/sse");

    events.addEventListener("message", (env) => {
      var data = env.data;
      console.log("received data: " + JSON.stringify(data));
      console.log("decoded data", JSON.stringify(JSON.parse(data)));
      var msg = JSON.parse(data);
      switch (msg.data) {
        case "game_start": 
          startGame();
          break;
        case "solved":
          gameend(true);
          break;
        case "lost":
          gameend(false);
          break;
        case "letter_correct":
          printWord();
          printLives();
          image();
          if (msg.player != playernumber) {
            document.getElementById("turn").innerHTML = "Your turn! Type one letter. The other Players guess was right.";
          } else
            document.getElementById("turn").innerHTML = "Well Done! Now its the other Players turn.";
          break;
        case "letter_false":
          printLives();
          printWord();
          image();
          if (msg.player != playernumber) {
            document.getElementById("turn").innerHTML = "Your turn! Type one letter. The other Players guess was wrong.";
          } else
            document.getElementById("turn").innerHTML = "Nice Try. Now its the other players turn.";
          break;
      }
    });

    events.addEventListener("open", () => {
      console.log(`connected to event stream at /sse`);
      retryTime = 1;
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

//MAIN
$(document).ready(function () {
  //Login Logic
  //get username from cookie
  let username = getCookie("username");
  if (username == "" || username == null)
    document.getElementById("login").hidden = false;
  else if (!checkLogin(username)) {
    //TODO handle wrong login
  }
  else {
    loggedin();
    //TODO game waiting room
  }
  subscribeEvents();
});
