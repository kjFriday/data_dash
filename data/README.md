# data_dash mock data

Static fixtures for building dashboard tiles before the live APIs are wired
up. Both files are plain text and need no parser beyond splitting lines and
fields. Comment lines start with `#`. Both files use CRLF line endings.

## numbers.dat

The integers 1 to 100 as Wikidata records them. Wikidata content is CC0 1.0.
The header comments give the query endpoint, the response size and hash, and
the selection rule.

Tab-separated, one row per number, no header row:

| # | Column      | Type   | Meaning                                                                    |
|---|-------------|--------|----------------------------------------------------------------------------|
| 1 | `n`         | int    | The number, 1 to 100.                                                      |
| 2 | `item`      | string | Wikidata item ID, for example `Q199` for 1.                                |
| 3 | `sitelinks` | int    | How many Wikimedia sites link the item (73 to 221; mean about 99).         |
| 4 | `classes`   | string | Every class the item is an instance of (P31), separated by `\|`.           |

Shape: 100 rows, 144 distinct classes, 3 to 59 classes per row.

Class tags reflect what Wikidata editors have recorded, not mathematical
truth. For example, only 10 rows carry `deficient number`. Label charts as
"tagged on Wikidata".

**Intended charts**

- **Category counts (horizontal bar):** split `classes` on `|`, count rows per
  class, show the top 15. The top classes are `positive integer` (100),
  `composite number` (74), `odd number` and `even number` (50 each),
  `harshad number` (33) and `prime number` (25).
- **Prime, composite or neither (donut):** 25 prime, 74 composite, 1 neither.
- **Sitelinks by number (line or bar, x = `n`):** shows popularity falling
  away after the single digits, with small bumps at the multiples of 10 and
  a spike at 100 (142).
- **KPI tiles:** row count, distinct classes, median sitelinks, and the most
  linked item.
- **Data table:** all 100 rows, with a class filter to exercise search and
  pagination.

## tracks.dat

48 position samples over time: three moving tracks and one stationary
point.

Space-separated, one sample per row, no header row:

| # | Column | Type  | Meaning                                                   |
|---|--------|-------|-----------------------------------------------------------|
| 1 | `t`    | float | Time step from 0 within each track; track 7 every 2 steps. |
| 2 | `x`    | float | Horizontal position, -280 to 280.                         |
| 3 | `y`    | float | Vertical position, -180 to 172.                           |
| 4 | `id`   | int   | Track ID: 1, 2 and 3 move; 7 stays at (60, 40).           |

Rows per track: 1 = 15, 2 = 14, 3 = 11, 7 = 8.

**Intended charts**

- **Track plot (scatter, x against y, coloured by `id`):** connect each
  track's points in `t` order to draw its path. Track 1 drifts left to right,
  track 2 climbs diagonally, track 3 arcs, and track 7 stays at one point.
  Fix the axes at about -300..300 by -200..200 so the tracks keep their
  proportions.
- **Position over time (line, x = `t`):** `x` and `y` per track, as small
  multiples.
- **Samples per track (bar):** a simple categorical check for the chart
  components.
