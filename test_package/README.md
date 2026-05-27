# Package Scanner Fixtures

Use `test_package` as the `root_dir` for `PackageService::scan_all`.

Expected scan result with the current manifest parser:

- `scripts/game/valid_minefield` is valid and should appear in `games()`.
- `scripts/screensaver/valid_clock` is valid and should appear in `screensavers()`.
- `scripts/game/missing_entry_game` is invalid because `entry` is missing.
- `scripts/boss/wrong_type_boss_declares_game` is invalid because it is inside `scripts/boss` but declares `"type": "game"`.
- `data/mod/screensaver/invalid_api_range` is invalid because `api.min` is greater than `api.max`.
- `data/mod/boss/unknown_type_boss` is invalid because `"type": "boos"` is not supported.

Expected counts:

- games: 1
- screensavers: 1
- bosses: 0
- errors: 4
