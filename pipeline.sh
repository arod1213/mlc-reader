cargo run -- migrate && \
cargo run -- modify && \
cargo run -- enrich -m writer && \
cargo run -- enrich -m publisher && \
cargo run -- update -p './updates.txt' && \
turso db create mlc-dump --from-file './local.db'
