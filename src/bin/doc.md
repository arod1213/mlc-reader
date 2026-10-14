# MLC Reader API Spec

What Resonance needs from `mlc-reader` to power the deal calculator writer split, the publisher cap table, the Schedule A check and writer catalog import. Inputs and outputs per endpoint, the share math, and the data fixes underneath.

**For** Aidan**From** Gabriel / Resonance**Dataset today** BWARM dump 2026-09-04**Date** 2026-10-10

New new endpoint or field Fix exists today but returns wrong or empty data

## What Resonance will call it for

| Resonance feature | Input from the user | Endpoint | What we show |
| --- | --- | --- | --- |
| Deal calculator: Fill from MLC | Song ISRC + chosen writer | `GET /api/works?isrc=` | Writer's collection % when they have their own publisher slice; otherwise "individual split not in MLC" |
| Cap table card | Song ISRC or ISWC | `GET /api/works?isrc=` | Who collects, who controls, writers, dispute flag |
| Schedule A check | Up to 200 ISRCs from an uploaded Schedule A | `POST /api/works/lookup` | Per row: match, writer missing, share differs, not in MLC |
| Writer catalog import (Ryder's Sheet) | Writer name or IPI | `GET /api/parties` then `GET /api/parties/{ipi}/works` | Every work the writer is on, with ISRCs and their slice |
| Song title lookup | Title, optional artist | `GET /api/works?title=` | Candidate works to pick from |

## Conventions

- **Transport.** Keep listening on loopback only. Resonance reaches it through a dedicated Cloudflare tunnel on `mlc.artistpg.com` behind Cloudflare Access, so the service itself needs no auth. Keep the existing `/api/*` paths and response fields; everything here is additive.
- **Errors.** JSON body `{"error":{"code":"invalid_isrc","message":"…"}}`. Use 400 for bad input (bad ISRC/ISWC/IPI checksum), 404 for an unknown id, 422 for a valid but unsupported combination, 503 when the DB is unavailable. Never answer an invalid request with `200 []`; Resonance treats `[]` as "MLC has nothing".
- **Identifiers.** ISRC: accept any case and dashes/spaces, normalise to 12 uppercase chars. ISWC: accept `T-929.744.911-7` or `T9297449117`. IPI: digits only, leading zeros allowed, checksum-validated (as today). Return identifiers normalised.
- **Paging.** `limit` (default 25, max 100) plus an opaque `cursor`; responses carry `next_cursor` (null at the end) and `total` when cheap to compute. Today `offset` is a page index; keep it working, but prefer cursors for new endpoints.
- **Dataset stamp.** Every response includes header `X-MLC-Dataset: 2026-09-04` (the BWARM dump date). Resonance keys its cache on it and shows it as "MLC data as of".
- **Percentages.** Numbers 0–100, up to 2 decimals. `null` means "MLC gives no number" (writer rows); `0` means MLC says zero. Never coerce empty to 0.

## Endpoints

GET/healthNew fields

Liveness plus what data is loaded.

```
{ "status": "ok", "service": "mlc-reader-api", "version": "0.2.0",
  "dataset": "2026-09-04", "built_at": "2026-10-11T06:40:00Z",
  "counts": { "works": 41210345, "shares": 193000000, "writer_shares": 127000000, "parties": 6100000 } }
```

`counts.writer_shares` lets us alert if a rebuild ever drops writers again.

GET/api/worksFixNew fields

Look up works. Exactly one mode per request, in this precedence: `isrc` > `iswc` > `ipi` > `title`/`artist`.

isrc

Exact, indexed. Works today, but writers are missing (see data fixes).

iswc

Exact, indexed.

ipi

Works where this party holds any share (writer or publisher). Today returns `[]` for writers.

title

Full-text, case-insensitive, prefix-friendly (`blinding lig` should match). Must work **without** `ipi`; today it forces `ipi = 0` and always returns `[]`.

artist

Optional filter alongside `title`; case-insensitive match on release artist name.

release_limit

Max releases embedded per work, default 25, max 200, newest first, one per ISRC. This is also the fix for the \~6 s Blinding Lights lookup (19k linked recordings).

limit, cursor

As in conventions.

Returns `{"works":[Work], "next_cursor":null}`. Keep returning a bare array if that's easier for compatibility, but then add `X-Next-Cursor`.

GET/api/works/{work_id}

One `Work` by MLC work id (e.g. `BA7H8C`). 404 if unknown. Accepts `release_limit`.

POST/api/works/lookupNew

Batch lookup for the Schedule A check, so we don't make 200 round trips.

```
// request
{ "isrcs": ["USUG11904206", "USSM12209777", "QZWH32601120"], "release_limit": 0 }

// response
{ "results": {
    "USUG11904206": { …Work… },
    "USSM12209777": { …Work… },
    "QZWH32601120": null },
  "invalid": [] }
```

Max 200 ISRCs per call. `null` = valid ISRC, no work in MLC. Malformed ISRCs go in `invalid` instead of failing the batch. Target under 5 s for 200.

GET/api/partiesFix

Find a writer or publisher. `input` = name or IPI, `role` = `writer` | `publisher`, `limit`.

```
[ { "id": 4015891, "name": "MARTIN MAX SANDBERG", "first_name": "MARTIN MAX", "last_name": "SANDBERG",
    "ipi": 293094253, "pro": "STIM", "role": "writer", "work_count": 1412 } ]
```

Name search works today. Fix: `role` and `pro` are NULL on every party, so the `role` filter does nothing. Add `work_count` so we can rank "Max Martin" (the writer) above namesakes.

GET/api/parties/{ipi}/worksNew

A party's catalog: every work they hold a share in. Powers writer catalog import and the publisher side of the cap table. Params: `role` (writer | publisher, filter on how they appear), `limit` (max 100), `cursor`.

```
{ "party": { "id": 4015891, "name": "MARTIN MAX SANDBERG", "ipi": 293094253 },
  "works": [
    { "work_id": "BA7H8C", "title": "BLINDING LIGHTS", "iswc": "T9297449117",
      "isrcs": ["USUG11904206", "…"], "isrc_count": 161,
      "role": "ComposerLyricist",
      "slice": { "kind": "own", "publisher": "WC MUSIC CORP.", "collected_pct": 8.5 }
      // or { "kind": "shared", "publisher": "…", "collected_pct": 42.49, "writer_count": 3 }
      // or { "kind": "none" } when no publisher row links to this writer
    } ],
  "next_cursor": "…", "total": 1412 }
```

`isrcs` capped at 25 per work (with `isrc_count`); we fetch the rest via `/api/works/{id}` when needed.

GET/api/parties/{id}/publishersFixGET/api/parties/{id}/writersFix

Keep them; today always `[]` because `publisher_relations` has 0 rows. Please also accept an explicit `?ipi=` instead of guessing whether a numeric id is an IPI (about 1 in 100 ids pass the IPI checksum by chance).

## The Work object

Existing fields stay. New fields are marked.

```
{
  "id": "BA7H8C",
  "title": "BLINDING LIGHTS",
  "iswc": "T9297449117",
  "duration_ms": null,
  "in_dispute": false,                 // FIX: data is "true"/"false" lowercase; today always false
  "release_count": 19419,              // NEW
  "releases": [                        // FIX: today [] whenever any joined row is null or an ISRC is empty
    { "id": 123, "title": "After Hours", "artist_name": "THE WEEKND", "label_name": "Republic",
      "distro_name": "UMG", "isrc": "USUG11904206", "release_date": "2020-03-20" }  // FIX: date is null today
  ],
  "parties": [
    { "id": 970690, "full_name": "UNIVERSAL MUSIC WORKS", "ipi": 832425062, "pro": "ASCAP",
      "shares": [
        { "id": "PSA_…", "party_id": 970690, "role": "RightsAdministrator",
          "share": 19.24,                         // null for writer rows
          "share_type": "MusicalWorkCollectionShare", // NEW (today mislabeled as rights_type)
          "rights_type": "MechanicalRight",      // FIX: real column 6
          "territory": "US",                     // FIX: today holds the validity start date
          "use_type": "Stream",                  // NEW
          "validity_start": "2025-07-21",        // NEW
          "preceding_ids": ["PSC_…"]            // NEW: column 9 split on "|"
        } ] },
    { "id": 5550001, "full_name": "ABEL TESFAYE", "ipi": 123456789, "pro": "SOCAN",
      "shares": [ { "id": "PSC_…", "role": "ComposerLyricist", "share": null, "preceding_ids": [] } ] }
                                            // FIX: writer rows exist again
  ],
  "cap_table": { … }                     // NEW, see below
}
```

**Illustrative values.** Names, IPIs and percentages above show the shape, not verified data.

## cap_table: the share math

Compute this server-side so every client gets the same answer. How the BWARM share file links up:

- Writer rows are `PSC_` rows with an empty percentage.
- Original publisher (OP) rows are `PSC_` rows (often `0.0`) whose `Preceding` points at the writer row(s) they represent.
- Administrator / sub-publisher rows are `PSA_` rows that carry the US Stream collection % and point at an OP row.
- `Preceding` can list several ids joined by `|` (55,049 of the first 2M share rows do).

```
"cap_table": {
  "basis": "US digital mechanical collection share (Stream); not ownership",
  "collects": [                        // who receives money: positive shares summed per party
    { "party_id": 926076, "name": "WC MUSIC CORP.", "ipi": 392888203, "pct": 42.49,
      "as": ["OriginalPublisher", "RightsAdministrator", "SubPublisher"] } ],
  "controls": [                        // whose slice it is: walk PSA -> OP -> writer(s)
    { "op_party_id": 926076, "op_name": "WC MUSIC CORP.", "collected_pct": 42.49,
      "writers": [ { "party_id": 4015891, "name": "MARTIN MAX SANDBERG", "ipi": 293094253 } ],
      "administered_by": [ { "party_id": 716372, "name": "KOBALT MUSIC PUB AMERICA INC", "pct": 25.0 } ] } ],
  "writers": [
    { "party_id": 4015891, "name": "MARTIN MAX SANDBERG", "ipi": 293094253, "role": "ComposerLyricist",
      "split": { "kind": "own", "collected_pct": 8.5 } },
    { "party_id": 5550001, "name": "ABEL TESFAYE", "ipi": 123456789, "role": "ComposerLyricist",
      "split": { "kind": "shared", "collected_pct": 42.49, "writer_count": 3 } },
    { "party_id": 5550002, "name": "…", "split": { "kind": "none" } } ],
  "total_pct": 100.0,
  "unlinked_pct": 0.0                  // positive % whose chain reaches no writer
}
```

1. **Collects** = sum of positive `share` per party. Positive shares across a work should sum to about 100.
2. **Controls**: for each positive row, follow `preceding_ids` up to its OP row, then to the writer row(s) the OP precedes. Credit the % to that OP slice.
3. **Never add an OP and its own administrator together.** If any PSA row points at an OP, count the PSA %; count the OP's own % only when nothing points at it.
4. **Writer split**: `own` only when an OP slice links to exactly one writer. If it links to several writers, return `shared` with the group total and `writer_count`, and **no per-writer number**. MLC never publishes writer percentages and we won't guess. `none` when no slice reaches the writer.
5. Positive % that reaches no writer goes in `unlinked_pct`.

## Data fixes underneath

Root causes found read-only on the VPS copy (`/opt/mlc-reader`, da326ab plus uncommitted `src/api.rs`). The writer and search gaps are not query problems alone: writer shares were deleted at import, so the DB needs a rebuild from the retained dump in `/srv/resonance-deal-sandbox-data/20260907T013000Z/bwarm/` (no download needed).

| Problem | Where | Fix |
| --- | --- | --- |
| Share columns read from the wrong positions (rights_type ← 5, territory ← 7, preceding ← 8) | `src/bwarm/share.rs:92-95` | Header is Id, Work, Party, Role, Pct, RightShareType(5), RightsType(6), ValidityStart(7), ValidityEnd(8), Preceding(9), Territory(10), UseType(11) |
| Empty writer % stored as 0.0 | `share.rs:104` | Store NULL |
| All writer shares deleted (127M of 193M rows) | `migration/trim.rs:48-58` | Never trim writer roles; only trim 0.0 non-writer rows nothing points at |
| `Preceding` with several ids breaks joins | `trim.rs:48`, `migration/mod.rs:275` | Split on `\|` into a link table `share_precedents(share_id, preceding_id)` |
| Title/artist search always empty | `api.rs:151-161`, `mutations/works.rs:85` | Make `party_ipi` optional; add FTS5 on title |
| `publisher_relations` empty; `role`/`pro` NULL | `migration/mod.rs` enrich, `assign_roles` | Join through the link table; fix the writer/publisher inversion; fill `pro` |
| Releases silently `[]`; date null; dispute always false | `works.rs:113-127, 163-167` | Filter nulls, skip bad ISRCs, cap with `release_limit`; emit date text; case-insensitive "true" |
| HTTP server only exists on the VPS | uncommitted `src/api.rs` | Commit it to the repo |

## How we'll check it

- `/health` `counts.writer_shares` is in the tens of millions (it is effectively 0 today).
- `GET /api/works?isrc=USUG11904206` (Blinding Lights) lists the writers, `cap_table.collects` sums to about 100, publisher totals match today's live reader, and the call returns in under 1.5 s at p95.
- `GET /api/works?ipi=293094253` returns Max Martin's works; `GET /api/parties/293094253/works` pages through them.
- `GET /api/works?title=blinding lights` returns the work without an IPI.
- `POST /api/works/lookup` with 200 ISRCs returns in under 5 s, with `null` for unknown ones.
- A work with three writers under one publisher returns `"kind":"shared"` and no per-writer number.

Priority if time is short: 1) the data fixes and rebuild, 2) `cap_table` on `/api/works`, 3) `/api/parties/{ipi}/works`, 4) batch lookup, 5) title search.
