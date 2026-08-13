# MLC Reader

MLC Reader is a Rust toolkit for standing up a queryable MLC BWARM database.

The command-line interface is primarily an operations tool: it downloads BWARM TSV files, loads them into a local or remote `libsql` database, trims the imported data, creates indexes, and builds derived relationship tables.

Once the database is built, the project is intended to be used as a Rust library for querying works, parties, shares, writer relationships, publisher relationships, and derived catalog insights. The binary also includes a few query commands, but those are best treated as inspection and smoke-test helpers rather than the main application interface.

## What It Does

- Downloads the latest MLC BWARM TSV dump over SSH.
- Migrates the TSV files into a `libsql`/SQLite-compatible database.
- Trims and indexes the loaded tables for faster lookup.
- Enriches the database with derived writer, publisher, role, and share statistics.
- Applies PRO affiliation corrections from a JSONL update file.
- Exposes library functions for querying works, parties, relations, and talent-style catalog insights.
- Includes small CLI query helpers for validating the loaded database.

## Requirements

- Rust toolchain
- SQLite-compatible `libsql` database
- MLC BWARM SSH credentials for downloading dumps

## Configuration

The CLI reads configuration from environment variables. A local `.env` file is supported through `dotenv`.

```bash
MLC_DB_URL=file:./mlc.db
MLC_DB_TOKEN=

MLC_HOST=example.mlc-host
MLC_USER=example-user
MLC_PUBLIC_KEY=/path/to/public/key
MLC_PRIVATE_KEY=/path/to/private/key
```

For local databases, set `MLC_DB_URL` to a `file:` URL. For remote `libsql`, set `MLC_DB_URL` to the remote URL and provide `MLC_DB_TOKEN`.

## Build

```bash
cargo build --release
```

The release binary is written to:

```bash
./target/release/mlc-reader
```

During development, you can run CLI commands through Cargo:

```bash
cargo run -- party-search --input "writer name"
```

## Data Pipeline

`pipeline.sh` runs the full import and enrichment flow:

```bash
./pipeline.sh
```

The script currently:

1. Builds the release binary.
2. Downloads BWARM TSV files into `./mlc-bwarm`.
3. Migrates the TSV files into the configured database.
4. Trims selected database content.
5. Creates search indexes.
6. Enriches writer, publisher, and share-derived tables.
7. Applies PRO affiliation updates from `./updates.txt`.

The MLC dump is refreshed roughly every five days, so this script is intended to be run periodically, for example from cron.

## Database CLI Commands

These commands build and maintain the queryable database.

```bash
mlc-reader save --path ./mlc-bwarm
mlc-reader migrate --path ./mlc-bwarm
mlc-reader trim --vacuum
mlc-reader index-search
mlc-reader index-trim
mlc-reader enrich --method writer
mlc-reader enrich --method publisher
mlc-reader enrich --method role
mlc-reader enrich --method share
mlc-reader update --path ./updates.txt
```

Command summary:

- `save`: download BWARM TSV files to disk.
- `migrate`: load BWARM TSV files into the configured database.
- `trim`: remove or compact unused data; `--vacuum` also vacuums the database.
- `index-search`: create search tables and indexes used by query commands.
- `index-trim`: create indexes used by trim/share processing.
- `enrich`: populate derived relation, role, and share-stat tables.
- `update`: apply PRO affiliation updates from a JSONL file.

`update` expects JSONL records shaped like:

```json
{"id":123456,"pro":10}
```

## Library Query API

After the database is populated, applications should generally query it through the library modules instead of shelling out to the CLI.

Primary modules:

- `mlc_reader::mutations::works`: work lookup and work search.
- `mlc_reader::mutations::parties`: party search and talent-style writer discovery.
- `mlc_reader::mutations::relations`: writer collaborator and publisher relation queries.
- `mlc_reader::mutations::society`: PRO affiliation updates.
- `mlc_reader::types`: shared party and role types.

Example shape:

```rust
use libsql::Builder;
use mlc_reader::mutations::works::{self, WorkSearchParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Builder::new_local("file:mlc.db").build().await?;
    let conn = db.connect()?;

    let works = works::search_works(
        &conn,
        WorkSearchParams {
            title: Some("SONG TITLE".to_string()),
            limit: 10,
            ..WorkSearchParams::default()
        },
        true,
    )
    .await?;

    println!("{works:#?}");
    Ok(())
}
```

The query functions expect a populated `libsql::Connection`. They do not open the database themselves, which keeps database configuration in the calling application.

## Query CLI Helpers

The binary includes a small set of query commands for manually checking the database after import or while developing library queries. Every query command requires `MLC_DB_URL` to be configured. Remote databases also require `MLC_DB_TOKEN`.

### Search Parties

Search writers and publishers by name or IPI name number.

```bash
mlc-reader party-search --input "TAYLOR SWIFT"
mlc-reader party-search --input 123456789 --role writer
mlc-reader party-search --input "UNIVERSAL" --role publisher
```

Options:

- `--input`, `-i`: party name or numeric IPI name number
- `--role`, `-r`: optional role filter, either `writer` or `publisher`

Notes:

- Name searches use the `parties_fts` full-text search table.
- Numeric input is interpreted as an IPI name number when valid.
- Results are limited to 30 parties.

### Search Works

Search works by title, artist name, and/or party IPI.

```bash
mlc-reader work-search --name "SONG TITLE"
mlc-reader work-search --artist "ARTIST NAME"
mlc-reader work-search --ipi 123456789
mlc-reader work-search --name "SONG TITLE" --artist "ARTIST NAME" --ipi 123456789
```

Options:

- `--name`, `-n`: exact work title filter
- `--artist`, `-a`: exact release artist filter
- `--ipi`, `-i`: party IPI name number filter

Notes:

- Title and artist inputs are uppercased before querying.
- Results include work metadata, matching releases, parties, and shares.
- The current CLI returns up to 10 works.

### Get Work By ID

Fetch a work and its related releases, parties, and shares by MLC work ID.

```bash
mlc-reader work --id MLC_WORK_ID
```

Options:

- `--id`, `-i`: MLC work ID

### Writer Collaborators

Fetch top collaborators for a writer party ID.

```bash
mlc-reader relation --id 123456
```

Options:

- `--id`, `-i`: writer party ID

Notes:

- This command reads from the derived `writer_relations` table.
- Run `enrich --method writer` before using it on a freshly imported database.
- Results are ordered by collaboration occurrence count and limited to 15 parties.

### Talent Discovery

List writer parties that appear to have notable unassigned or unsigned share patterns.

```bash
mlc-reader talent
```

Notes:

- This command is based on share and party statistics created during enrichment.
- Run `enrich --method share` before relying on the results.

## Output

The current CLI prints Rust debug output from the returned structs. That is useful for development and inspection, but it is not yet a stable machine-readable output format.

## Development Notes

- The CLI entry point is `src/main.rs`.
- Query implementations live under `src/mutations/`.
- Migration and enrichment code lives under `src/migration/`.
- BWARM parsing types live under `src/bwarm/`.
