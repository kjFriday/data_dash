# Provenance

This file records where each data file came from. Sources are identified by
their SHA-256 and relative path in the staged copies listed in
the owner's private staging manifest (`_extracted/MANIFEST.tsv`, kept locally and not published). The source root folder names are not
repeated here because they contain terms that are excluded from this project.

| File | Source (staged relpath, SHA-256) | Change | Result SHA-256 |
|------|----------------------------------|--------|----------------|
| `data/numbers.dat` | `gen/numbers.dat`, `55F46AB4DD92532A693C4D13A08C0E564E355AFAFE3A82921979214C49748043` | Removed header line 3 (the `# asked` line). All other bytes are unchanged (UTF-8, CRLF). | `83067C5851040D3B9FD7CFBA75FC83C105382B35536B1DE055CF9E31D448FA93` |
| `data/tracks.dat` | `fortran/tracks.dat`, `C81ECAD206BF3DDB7F4BE2749DBF5895A5DEE402E76398514FF08090F51C81B7` | None; byte-identical copy. | same as source |
| `data/README.md` | Written new. | The `tracks.dat` column order (T, X, Y, ID) comes from the `READ` statement of the original Fortran reader. The counts and ranges were computed from the two files above. | - |

Licence: `numbers.dat` is Wikidata content, CC0 1.0, as its header states.

## Copied in 0.2.0 (same portfolio, AGPL-3.0)

| File | From | Changes |
|---|---|---|
| `src/host/{mod,api,assets,install,paths}.rs`, `build.rs`, `tests/icon.rs` | the ticket tracker's `src/host`, `build.rs`, `tests/icon.rs` | Imports only; Data Dash passes read-only tables (`src/main.rs`). The installer and paths are themselves ports of the CRUD hub's `installer.py` and `paths.py`. |
| `src/host/textsafe.rs` | the ticket tracker's `core::validate::clean` (the hub's textsafe rules) | Moved to its own module, with a test. |
