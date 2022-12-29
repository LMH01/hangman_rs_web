#!/bin/bash
#This script builds the web assembly targets and copies them to /web/wasm
#wasm-pack is required for the build process
cd ./wasm
(
    exec wasm-pack build --target no-modules
    if [ &? -ne 0]
    then
        exit
    fi
)
cd ..
if ! [[ -f "web/wasm" ]]
then 
    mkdir web/wasm
fi
yes | cp -rf ./wasm/pkg/hangman_rs_wasm.js ./web/wasm
yes | cp -rf ./wasm/pkg/hangman_rs_wasm_bg.wasm ./web/wasm
echo "wasm files have been generated and copied to /web/wasm"