/**
 * Submits a post request to the url
 */
async function postData(url = '', data = {}) {
  return postData(url, data = {}, null); // parses JSON response into native JavaScript objects
}

// This function is a workaround until I figgure out how I can send post request with json body content using wasm-bindgen from rust.
// Example POST method implementation:
// Copied from https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch
/**
 * Submits a post request to the url
 * @param {String} url The url to which the post request should be sent
 * @param {String} data Data formatted as json string
 * @param {Map} additional_headers Additional headers that should be added to the request
 * @returns The response formatted as json
 */
async function postData(url = '', data = {}, additional_headers) {//TODO rename to postRequest
    const headers = new Headers;
    headers.append('Content-Type', 'application/json');
    if (additional_headers != undefined || additional_headers != null) {
      for (const [key, value] of additional_headers) {
        headers.append(key, value);
      }
    }
    // Default options are marked with *
    const response = await fetch(url, {
      method: 'POST', // *GET, POST, PUT, DELETE, etc.
      mode: 'cors', // no-cors, *cors, same-origin
      cache: 'no-cache', // *default, no-cache, reload, force-cache, only-if-cached
      credentials: 'same-origin', // include, *same-origin, omit
      headers,
      redirect: 'follow', // manual, *follow, error
      referrerPolicy: 'no-referrer', // no-referrer, *no-referrer-when-downgrade, origin, origin-when-cross-origin, same-origin, strict-origin, strict-origin-when-cross-origin, unsafe-url
      body: JSON.stringify(data) // body data type must match "Content-Type" header
    });
    return response.json(); // parses JSON response into native JavaScript objects
}

/**
 * @param {String} name The name of the cookie
 * @returns The cookie for the specified name
 */
function getCookie(name) {
  const value = `; ${document.cookie}`;
  const parts = value.split(`; ${name}=`);
  if (parts.length === 2) return parts.pop().split(';').shift();
}