"""Aggregate resolved references into unit-level dependency edges.

A "unit" is the granularity used by the architecture diagrams: one core
module, one service, one UI page or one main-loop stage.
"""
import json
import os
from collections import defaultdict

AUDIT = os.environ.get("ARCH_AUDIT_DIR", "temp/audit")
RAW = os.path.join(AUDIT, "raw.json")
RESOLVED = os.path.join(AUDIT, "resolved.json")
OUT = os.path.join(AUDIT, "units.json")

HE = "crate::host_engine"
WORKSPACE = "crate::workspace"

SPLIT_DEEPER = {
    f"{HE}::services::widget::ui_object::interactives": 1,
    f"{HE}::services::widget::ui_object::surfaces": 1,
    f"{HE}::services::widget::runtime_object": 0,
    f"{HE}::services::lua": 1,
    f"{HE}::runtime": 1,
    f"{HE}::ui::home::settings": 1,
    f"{HE}::ui::home": 1,
    f"{HE}::ui::overlay": 1,
    f"{HE}::ui::exit": 0,
    f"{HE}::core": 1,
    f"{HE}::services": 1,
    f"{HE}::ui": 1,
}


def unit_of(module):
    """Map a module path to its unit path (longest matching split rule)."""
    if module == "crate":
        return "crate"
    if module == WORKSPACE:
        return "workspace"
    if module.startswith(WORKSPACE + "::"):
        parts = module.split("::")
        if len(parts) >= 4 and parts[2] in {"core", "service", "main_loop"}:
            return "::".join(parts[:4])
        return "workspace"
    if not module.startswith(HE):
        return module
    best = None
    for prefix, extra in SPLIT_DEEPER.items():
        if module == prefix or module.startswith(prefix + "::"):
            if best is None or len(prefix) > len(best[0]):
                best = (prefix, extra)
    if best is None:
        parts = module.split("::")
        return "::".join(parts[:3]) if len(parts) >= 3 else module
    prefix, extra = best
    if module == prefix:
        return prefix
    rest = module[len(prefix) + 2:].split("::")
    return prefix + "::" + "::".join(rest[:extra]) if extra else prefix


def layer_of(unit):
    if unit == "workspace":
        return "main_loop"
    if unit.startswith(f"{WORKSPACE}::core::"):
        return "core"
    if unit.startswith(f"{WORKSPACE}::service::"):
        return "service"
    if unit.startswith(f"{WORKSPACE}::main_loop::"):
        return "main_loop"
    if unit in ("crate", HE) or unit.startswith(f"{HE}::boot") or unit.startswith(f"{HE}::shutdown") or unit.startswith(f"{HE}::runtime"):
        return "main_loop"
    if unit.startswith(f"{HE}::core"):
        return "core"
    if unit.startswith(f"{HE}::services"):
        return "service"
    if unit.startswith(f"{HE}::ui"):
        return "ui"
    return "other"


def short(unit):
    if unit.startswith(WORKSPACE + "::"):
        return unit[len(WORKSPACE) + 2:].replace("::", "/")
    if unit == "workspace":
        return "workspace"
    return unit.replace(HE + "::", "").replace("::", "/")


def main():
    raw = json.load(open(RAW, encoding="utf-8"))
    resolved = json.load(open(RESOLVED, encoding="utf-8"))
    edges = defaultdict(lambda: {"count": 0, "items": set(), "sites": []})
    test_edges = defaultdict(int)
    for ref in resolved["references"]:
        if ref["reexport"]:
            continue
        source = unit_of(ref["from"])
        target = unit_of(ref["target_module"])
        if source == target:
            continue
        # A module referencing its own descendant unit is structural (mod.rs glue).
        if ref["test"]:
            test_edges[(source, target)] += 1
            continue
        edge = edges[(source, target)]
        edge["count"] += 1
        if ref["target_item"]:
            edge["items"].add(ref["target_item"])
        if len(edge["sites"]) < 6:
            edge["sites"].append(f'{ref["file"]}:{ref["line"]}')
    units = defaultdict(lambda: {"files": set(), "lines": 0})
    for file in raw["files"]:
        unit = unit_of(file["module"])
        units[unit]["files"].add(file["file"])
        units[unit]["lines"] += file["lines"]
    out_edges = []
    for (source, target), edge in sorted(edges.items()):
        out_edges.append({
            "from": source,
            "to": target,
            "count": edge["count"],
            "items": sorted(edge["items"]),
            "sites": edge["sites"],
            "from_layer": layer_of(source),
            "to_layer": layer_of(target),
        })
    out_units = [
        {"unit": unit, "short": short(unit), "layer": layer_of(unit), "files": sorted(info["files"]), "lines": info["lines"]}
        for unit, info in sorted(units.items())
    ]
    json.dump({"units": out_units, "edges": out_edges}, open(OUT, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
    print("units:", len(out_units), "edges:", len(out_edges))
    by_layer = defaultdict(int)
    for edge in out_edges:
        by_layer[(edge["from_layer"], edge["to_layer"])] += 1
    for key, count in sorted(by_layer.items()):
        print(" ", key, count)


if __name__ == "__main__":
    main()
