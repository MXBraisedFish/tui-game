# B8 presenter baseline

Date: 2026-09-28. Detailed measurements and rerun scripts live under
`E:\Code\tg-test\b8-render-baseline\`.

## Setup

- Windows Terminal 1.24.260710001, maximized, default profile `命令提示符`, Maple Mono NF CN; the shell command was launched with `pwsh` while using that default profile's appearance.
- The benchmark process reported a terminal viewport of 179×40 cells. Each size/mode/FPS point ran for 10 seconds against the actual `TerminalService` stdout writer.
- Logical frames were 80×24, 160×50 and 240×80. Rows interleaved ASCII, wide CJK cells and rich bold/underline/reverse styles. Modes were static, one-cell change and forced full redraw; the latter two toggle the first cell.
- The benchmark paced calls toward 30, 60 and 120 FPS. It records frame-call p50/p95/max, achieved FPS, bytes, Rust `Write::write` calls, flushes, and test-process CPU time from `GetProcessTimes` (single-core equivalent). CPU time excludes the separate Windows Terminal process. Write counts are calls at the Rust `Write` interface, not kernel syscalls.

The maximized viewport fit 80×24. It did not fit 160×50 vertically or 240×80 in either dimension. The larger logical frames were still written to the live terminal, but the terminal clipped/scrolled part of their output. Treat those rows as exploratory stream-load measurements; B8.3 remains open until the target viewport condition is settled. This also explains why these numbers must not be described as matching-window results.

## Full-redraw results

| Logical frame | Target FPS | Achieved FPS | p50 frame write | p95 frame write | Test-process CPU | Bytes / 10 s |
|---|---:|---:|---:|---:|---:|---:|
| 80×24 | 30 | 29.69 | 1.48 ms | 1.99 ms | 2.19% | 4,473,414 |
| 80×24 | 60 | 58.75 | 1.47 ms | 2.05 ms | 4.22% | 8,856,456 |
| 80×24 | 120 | 115.20 | 1.52 ms | 2.17 ms | 10.62% | 17,366,486 |
| 160×50 | 30 | 29.70 | 5.74 ms | 6.82 ms | 9.06% | 18,355,788 |
| 160×50 | 60 | 58.76 | 5.94 ms | 6.90 ms | 19.05% | 36,340,752 |
| 160×50 | 120 | 115.32 | 6.02 ms | 7.00 ms | 35.29% | 71,321,816 |
| 240×80 | 30 | 29.67 | 14.88 ms | 17.95 ms | 24.19% | 45,022,824 |
| 240×80 | 60 | 58.23 | 15.12 ms | 18.23 ms | 42.92% | 88,378,136 |
| 240×80 | 120 | 65.60 | 14.98 ms | 17.25 ms | 45.02% | 99,747,536 |

Static and local-update rows, including their write counts, are in
`windows_terminal_baseline.csv`. The output-writing cost grows with frame size; on the 240×80 full redraw, the target of 120 FPS was not reached. This is a baseline observation, not a claimed optimization result.

## Reproduction

Run `run_full.ps1` from the test directory. It launches the ignored release benchmark in a maximized Windows Terminal window, writes partial CSV results after each of the 27 points, and records the exit code. `TG_B8_SECONDS_PER_TARGET` can be reduced for a smoke run. The raw CSV is `windows_terminal_baseline.csv`.

The earlier ConPTY smoke run measured 80×24 for 1 second per target, but its fixed 80×24 viewport was too small and its output was forwarded through the tool PTY. It is retained only as a launch-path check; it is excluded from the baseline table.

## B8.4 frame-buffered output comparison

Date: 2026-09-28. The first presenter optimization now encodes each frame into a reusable `Vec<u8>`, then calls `write_all` once and flushes once. Per-cell diffing, ANSI bytes, cursor ordering, and frame commit-after-flush behavior are retained. A short write is retried by `write_all`; any write or flush error keeps the prior frame and requests a full redraw on retry.

The before and after runs used the same maximized Windows Terminal default profile, Maple Mono NF CN, 179×40 viewport, benchmark harness, and 10-second target segments. The release run produced 27/27 rows and exit code 0. Full output and rerun script are in `E:\Code\tg-test\b8-render-buffered\`. The final error-recovery-only source adjustment was made after this performance run; it is outside the successful write path measured here.

| Logical frame | Target FPS | Achieved FPS before → after | p95 ms before → after | Test-process CPU before → after | Rust writes/frame before → after | Bytes/frame before → after |
|---|---:|---:|---:|---:|---:|---:|
| 80×24 | 120 | 115.20 → 115.20 | 2.17 → 1.20 (-44.6%) | 10.62% → 4.53% | 8,553 → 4 | 15,062 → 15,062 |
| 160×50 | 120 | 115.32 → 115.53 | 7.00 → 3.60 (-48.6%) | 35.29% → 17.33% | 33,909 → 16 | 61,804 → 61,804 |
| 240×80 | 120 | 65.60 → 115.60 | 17.25 → 7.82 (-54.6%) | 45.02% → 37.50% | 82,285 → 38 | 151,592 → 151,592 |

Each measurement still flushes once per frame. Bytes per frame are unchanged; the higher total byte count on the 240×80 after run comes from producing 1,156 frames rather than 658 in ten seconds. The 240×80 window still exceeds the physical viewport, so treat that row as same-window stream-load comparison, not visible 240×80 acceptance. 160×50 is also taller than the 40-row viewport. The 80×24 frame fits.

Static 80×24 at target 60 FPS stayed at 1 Rust write and 1 flush per frame (589 frames in each run); one-cell updates fell from 11 Rust writes to 1. These counts measure the `Write` interface, not kernel syscalls. The unchanged ANSI byte counts and clear p95/CPU improvement make style-state reuse unnecessary for this pass; avoid adding a second stateful optimization without new evidence. The full-frame 240×80 p95 is now below the 8.33 ms target interval, although the terminal still did not display that logical size in its viewport.

The success-path comparison used the same default console/profile. No screenshot-based flicker judgment or manual IME composition was part of B8.4. B8.5's user-visible flicker assessment remains separate from these presenter timings.
