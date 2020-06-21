#!/bin/bash
cd `dirname $0`;
cd "./wasm_apps";
cd "../core/build";
cargo run;
cd "../../";
cargo build --target thumbv7m-none-eabi;