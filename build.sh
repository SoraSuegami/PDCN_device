#!/bin/bash
cd "./wasm_apps";
cargo build --target wasm32-unknown-unknown --release;
cd "../";
cargo build --target thumbv7m-none-eabi;