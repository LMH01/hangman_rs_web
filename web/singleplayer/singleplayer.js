document.addEventListener("DOMContentLoaded", async function(){
    console.info("Initializing wasm");
    await wasm_bindgen('../wasm/hangman_rs_wasm_bg.wasm');
    wasm_bindgen.init();
    let x = await (wasm_bindgen.post_request("api/register"));
    console.log(x);
    let y = await (wasm_bindgen.get_request("api/lives"));
    console.log(y);
});

async function debug() {
    //let z = await (wasm_bindgen.post_request_data("api/submit_char", { character: document.getElementById("user-input").value[0] }));
    let z = await (postData("api/submit_char", { character: document.getElementById("user-input").value[0] }));
    console.log(z);
}