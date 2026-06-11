cargo run -- migrate
cargo run -- modify
cargo run -- enrich
cargo run -- update -p './update.txt'

turso db create mlc-dump --from-file './local.db'
