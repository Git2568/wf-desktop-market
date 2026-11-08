# Riven analysis market data

## Scope

The analysis service keeps the verified rules/reference data separate from live market data. `riven_analyze_stats` is local-only and does not wait for HTTP. `riven_analyze_market` combines two optional sources: the Digital Extremes weekly PC aggregate and a current Warframe.market listing batch.

The weekly source is `https://www-static.warframe.com/repos/weeklyRivensPC.json`. Its `pop` field is retained as the publisher's relative popularity index; it is not converted into a trade count, frequency, or price score. The two `rerolled` groups remain separate. Matching uses the trimmed, case-insensitive English catalog name and no aliases. `Last-Modified` is retained as source metadata; local fetch time is never presented as the statistics period.

The market comparison is deliberately bounded: it requests the current batch for one weapon through the existing MarketService gates, then keeps only visible, open listings with a positive explicit buyout. It reports the requested sample size, quoted price bounds, transparent attribute counts, and exact positive/negative set matches. It does not claim a whole-market top-N, a transaction price, seller history, or a valuation formula. Missing or failed weekly/market data is returned as a partial response with an error and stale indication when an older cache is used.

## Persistence and IPC

The service uses `app_local_data_dir/riven-analysis.sqlite3` and performs SQLite/file work in `spawn_blocking`. Embedded reference data is inserted on first use; a corrupt or unavailable local cache never prevents use of the verified embedded reference. Weekly data is considered fresh for 24 hours; listing samples are retained in the service cache for five minutes. Weekly and listing requests are started concurrently, with separate in-flight locks and a second cache check to deduplicate callers. Only a user analysis action or an explicit settings update can trigger network refresh; startup does not poll. Reference refresh revalidates/reimports the verified embedded tables and does not claim to update all game rules from the network. A refresh failure leaves the previous valid local data available and reports a stale warning.

Commands:

- `riven_analyze_stats({ request })`
- `riven_analyze_market({ request })`
- `riven_analysis_data_status()`
- `riven_refresh_analysis_data({ force })`

All command failures are returned as readable strings. No seller details are included in the analysis response.

## Verification boundary

The official weekly endpoint and response metadata were rechecked on 2026-11-08 (Asia/Shanghai); the captured response was HTTP 200, 148364 bytes, 811 rows, two Torid reroll groups, and `Last-Modified: Mon, 02 Nov 2026 01:00:04 GMT`. The parser accepts the endpoint's restricted JavaScript-object notation (bare keys, single-quoted strings, JSON scalar values including `null`) without evaluation. The captured feed passed the complete 811-row parser/validator smoke, including null compatibility rows and Torid grouping. This documents source shape and semantics only; it is not a performance benchmark or a claim that the current batch represents completed trades.
