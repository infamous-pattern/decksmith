check:
    ./scripts/check.sh

test:
    cargo test --workspace --locked

test-integration:
    cargo test --workspace --tests --locked

devices:
    cargo run --locked -p decksmithctl -- devices

virtual:
    cargo run --locked -p decksmithd -- --virtual-once
