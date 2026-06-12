# cargo run -- save && \
cargo run -- migrate -p './Sample Data' && \
cargo run -- modify && \
cargo run -- enrich -m writer && \
cargo run -- enrich -m publisher # && \
# cargo run -- update -p './updates.txt' && \
# turso db create mlc-dump --from-file './local.db'
