document.addEventListener("DOMContentLoaded", async function(){
    console.info("Initializing wasm");
    await wasm_bindgen('../wasm/hangman_rs_wasm_bg.wasm');
    wasm_bindgen.init();
});
