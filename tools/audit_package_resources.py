from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OFFICIAL_DIR = ROOT / "games" / "official"
SRC_DIR = ROOT / "src"

FORBIDDEN_SCRIPT_SNIPPETS = (
    "assets/lang",
    "assets/wordle",
    "scripts/game/",
    "dofile(",
    "require(",
    "io.open(",
)

TEXT_FIELDS = ("name", "description", "detail", "best_none")


def is_lang_key(value: object) -> bool:
    return isinstance(value, str) and "." in value and "/" not in value and "\\" not in value


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8-sig"))


def audit_official_package(package_dir: Path) -> list[str]:
    issues: list[str] = []
    game_manifest = package_dir / "game.json"
    package_manifest = package_dir / "package.json"
    if not package_manifest.exists() or not game_manifest.exists():
        issues.append(f"{package_dir}: missing package.json or game.json")
        return issues

    game = load_json(game_manifest)

    for field in TEXT_FIELDS:
        value = game.get(field)
        if value is None:
            continue
        if not is_lang_key(value):
            issues.append(f"{game_manifest}: field '{field}' must be a language key, got {value!r}")

    lang_dir = package_dir / "assets" / "lang"
    for filename in ("en_us.json", "zh_cn.json"):
        if not (lang_dir / filename).exists():
            issues.append(f"{package_dir}: missing package-local language file assets/lang/{filename}")

    for script_path in (package_dir / "scripts").rglob("*.lua"):
        text = script_path.read_text(encoding="utf-8-sig")
        for snippet in FORBIDDEN_SCRIPT_SNIPPETS:
            if snippet in text:
                issues.append(f"{script_path}: forbidden snippet {snippet!r}")

    return issues


def audit_host_special_cases() -> list[str]:
    issues: list[str] = []
    for path in SRC_DIR.rglob("*.rs"):
        text = path.read_text(encoding="utf-8-sig")
        if "GamePackageSource::Official" in text and any(
            api_name in text for api_name in ("translate", "read_text", "read_json", "read_bytes", "load_helper")
        ):
            issues.append(f"{path}: official package special-casing near runtime/resource API")
    return issues


def main() -> int:
    issues: list[str] = []
    for package_dir in sorted(OFFICIAL_DIR.iterdir()):
        if package_dir.is_dir():
            issues.extend(audit_official_package(package_dir))
    issues.extend(audit_host_special_cases())

    if issues:
        print("Package resource audit failed:")
        for issue in issues:
            print(f" - {issue}")
        return 1

    print("Package resource audit passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
