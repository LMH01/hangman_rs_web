use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

/// Sends a get request to the specified url.
/// 
/// Response only works correctly if the returned value is formatted in json.
#[wasm_bindgen]
pub async fn get_request(url: String) -> Result<JsValue, JsValue> {
    send_request(url, Method::GET, None).await
}

/// Sends a post request to the specified url.
#[wasm_bindgen]
pub async fn post_request(url: String) -> Result<JsValue, JsValue> {
    send_request(url, Method::POST, None).await
}

/// Sends a POST request to url with the specified java script value as data field.
#[wasm_bindgen]
#[allow(deprecated)]
#[deprecated(note = "This function does not yet work properly, use `postData` located in `utils.js` instead.")]
pub async fn post_request_data(url: String, data: JsValue) -> Result<JsValue, JsValue> {
    send_request(url, Method::POST, Some(&data)).await
}

/// Different types of http requests
enum Method {
    POST,
    GET,
}

/// Sends a request to the url with the specified method. Returns the response as json.
/// 
/// Usage of data field does not yet work properly. Probably a communication issue between js and rust.
/// To use json data please use the function postData located in `utils.js`.
async fn send_request(url: String, method: Method, data: Option<&JsValue>) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    match method {
        Method::POST => opts.method("POST"),
        Method::GET => opts.method("GET"),
    };
    opts.mode(RequestMode::Cors);

    if data.is_some() {
        opts.body(data);
    }

    let request = Request::new_with_str_and_init(&url, &opts)?;

    request.headers().set("Content-Type", "application/json")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.json()?).await?;

    // Send the JSON response back to JS.
    Ok(json)
}