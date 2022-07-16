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
  var res = await fetch("api/register", {
    headers: { "username": username }
  });
  var response = await res.text();

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
  new EventSource('http://localhost/sse').addEventListener("p2ready", msg => { console.log(msg.data); startGame() });
}

function playersturn() {

  new EventSource('http://localhost/sseyour').addEventListener("yourturn", msg => { console.log(msg.data); gameInput() });

}

function startGame() {
  console.log("You are " + playernumber);
  printLives();
  printWord();
  image();
  document.getElementById("waitingroom").hidden = true;
  document.getElementById("game").hidden = false;
  new EventSource('http://localhost/sse').addEventListener("p2played", msg => { console.log(msg.data); p2event(msg.data) });
}

async function p2event(data) {
  switch (data) {
    case "1": image();
      gameend(true);
      break;
    case "2":
      printWord();
      printLives();
      image();
      if (myturn(await (await (fetch("api/playernumber", { headers: { "username": username } }))).text())) {
        console.log("AAAAAAAAAAAAAAAAAAAAAAAA");
        document.getElementById("turn").innerHTML = "Your turn! Type one letter. The other Players guess was right.";
      } else
        document.getElementById("turn").innerHTML = "Well Done! Now its the other Players turn.";
      break;
    case "3":
      printLives();
      printWord();
      image();
      if (myturn(await (await (fetch("api/playernumber", { headers: { "username": username } }))).text())) {
        document.getElementById("turn").innerHTML = "Your turn! Type one letter. The other Players guess was wrong.";
      } else
        document.getElementById("turn").innerHTML = "Nice Try. Now its the other players turn.";
      break;
    case "4":
      image();
      gameend(false);
      break;
  }
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

  var response = await
    (await (fetch("api/submit_char", {
      headers: {
        "username": username
        , "char": document.getElementById("input-letter").value[0]
      }
    }))).text();
  console.log(response)
  switch (response) {
    case "1": image();
      gameend(true);
      break;
    case "2": image();
      yourturn = false;
      printWord();
      break;
    case "3": image();
      yourturn = false;
      printLives();
      break;
    case "4": image();
      gameend(false);
      break;
    case "5":
      alert("Not your Turn. Please wait");
      break;

  }

  document.getElementById("input-letter").value = "";
}

function myturn(numbar) {

  console.log("turn:" + numbar);
  console.log("playernumber:" + playernumber);
  if (numbar == playernumber.toString()) {
    console.log("MEEE");
    return true;
  }
  else
    return false;
}

async function gameend(tf) {
  printWord();
  document.getElementById("game").hidden = true;
  document.getElementById("gameend").hidden = false;
  document.getElementById("wonlost").innerHTML = "Game Over";
  if (tf)
    document.getElementById("wonlost").innerHTML = "You won";
  else
    document.getElementById("wonlost").innerHTML = "You lost";
  document.getElementById("wordwas").innerHTML = "The Word was: " + //document.getElementById("gameword").innerHTML;
    await (await (await
      (fetch("api/game_string", {
        headers: {
          "username": username
        }
      }
      ))).text());
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
});

