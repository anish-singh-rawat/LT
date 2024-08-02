cargo build --release --target wasm32-unknown-unknown --package launchpad_backend

candid-extractor target/wasm32-unknown-unknown/release/launchpad_backend.wasm > src/launchpad_backend/launchpad_backend.did

