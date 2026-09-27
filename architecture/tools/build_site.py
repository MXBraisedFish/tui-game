"""Build the architecture visualization data file.

Usage:
  python architecture/build_current.py

This internal builder receives its input directory through ARCH_AUDIT_DIR.
Inputs: raw.json, resolved.json, flow.json,
desc/*.json (optional item descriptions and findings from audit agents).
"""
import datetime
import glob
import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(__file__))
from units import unit_of, layer_of, short  # noqa: E402

AUDIT = os.environ.get("ARCH_AUDIT_DIR", "temp/audit")

LAYERS = [
    {"id": "main_loop", "name": "主程序 / 应用", "desc": "程序入口、boot / runtime / shutdown 阶段编排与应用层、终端 UI"},
    {"id": "ui", "name": "界面（应用层）", "desc": "首页、设置页、覆盖层等 TUI 页面"},
    {"id": "service", "name": "服务模块", "desc": "crates/service/ 下的各项服务"},
    {"id": "core", "name": "核", "desc": "crates/core/ 下的基础类型与状态"},
]

KIND_GROUP = {
    "const": "常量", "static": "常量", "assoc_const": "常量",
    "trait": "接口", "trait_fn": "接口",
    "struct": "类型", "enum": "类型", "type": "类型", "union": "类型",
    "fn": "函数", "method": "方法", "macro": "函数",
}


def load_json(path, default=None):
    if not os.path.exists(path):
        return default
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def strongly_connected(graph, nodes):
    index, low, stack, on_stack, result = {}, {}, [], set(), []
    counter = [0]
    sys.setrecursionlimit(10000)

    def visit(node):
        index[node] = low[node] = counter[0]
        counter[0] += 1
        stack.append(node)
        on_stack.add(node)
        for target in graph.get(node, ()):
            if target not in index:
                visit(target)
                low[node] = min(low[node], low[target])
            elif target in on_stack:
                low[node] = min(low[node], index[target])
        if low[node] == index[node]:
            component = []
            while True:
                member = stack.pop()
                on_stack.discard(member)
                component.append(member)
                if member == node:
                    break
            result.append(component)

    for node in sorted(nodes):
        if node not in index:
            visit(node)
    return result


def main():
    snapshot, output = sys.argv[1], sys.argv[2]
    raw = load_json(f"{AUDIT}/raw.json")
    resolved = load_json(f"{AUDIT}/resolved.json")
    flow = load_json(f"{AUDIT}/flow.json")

    descriptions, unit_info, agent_findings = {}, {}, []
    for path in sorted(glob.glob(f"{AUDIT}/desc/B*.json")):
        data = load_json(path, {})
        descriptions.update(data.get("items", {}))
        unit_info.update(data.get("units", {}))
        agent_findings.extend(data.get("findings", []))

    # Units and their files.
    units = {}
    for file in raw["files"]:
        unit = unit_of(file["module"])
        entry = units.setdefault(unit, {
            "id": short(unit), "module": unit, "layer": layer_of(unit),
            "lines": 0, "files": [], "items": [], "deps": {}, "dependents": {},
        })
        entry["lines"] += file["lines"]
        entry["files"].append({"path": file["file"], "lines": file["lines"]})

    # Edges between units (production code only; re-export glue excluded).
    edges = defaultdict(lambda: {"count": 0, "items": set(), "sites": []})
    for ref in resolved["references"]:
        if ref["reexport"] or ref["test"]:
            continue
        source, target = unit_of(ref["from"]), unit_of(ref["target_module"])
        if source == target:
            continue
        edge = edges[(source, target)]
        edge["count"] += 1
        if ref["target_item"]:
            edge["items"].add(ref["target_item"])
        if len(edge["sites"]) < 8:
            edge["sites"].append(f'{ref["file"]}:{ref["line"]}')
    for (source, target), edge in edges.items():
        units[source]["deps"][short(target)] = edge["count"]
        units[target]["dependents"][short(source)] = edge["count"]

    # Items (production only).
    for item in raw["items"]:
        if item["test"] or item["kind"] == "impl":
            continue
        unit = units[unit_of(item["module"])]
        item_id = f'{item["file"]}:{item["line"]}:{item["name"]}'
        unit["items"].append({
            "k": item["kind"], "g": KIND_GROUP.get(item["kind"], "其他"), "n": item["name"],
            "p": item["parent"], "t": item["trait"], "v": item["vis"],
            "d": descriptions.get(item_id) or (item["doc"].split("\n")[0] if item["doc"] else ""),
            "f": item["file"], "l": item["line"],
        })

    unit_list = []
    for unit in sorted(units.values(), key=lambda entry: (entry["layer"], entry["id"])):
        info = unit_info.get(unit["id"], {})
        unit["summary"] = info.get("summary", "")
        unit["notes"] = info.get("notes", "")
        unit["files"].sort(key=lambda file: file["path"])
        unit_list.append(unit)

    edge_list = [{
        "from": short(source), "to": short(target), "count": edge["count"],
        "items": sorted(edge["items"]), "sites": edge["sites"],
    } for (source, target), edge in sorted(edges.items())]

    # Automatic architecture checks.
    layer_by_id = {unit["id"]: unit["layer"] for unit in unit_list}
    checks = []
    for edge in edge_list:
        source_layer, target_layer = layer_by_id[edge["from"]], layer_by_id[edge["to"]]
        rule = None
        if source_layer == "core" and target_layer == "core":
            rule = "核依赖了另一个核（规则：核与核之间只能合并或拆分，不能互相依赖）"
        elif source_layer == "core":
            rule = "核依赖了核以外的模块（规则：核没有依赖，独立存在）"
        elif source_layer == "service" and target_layer in ("ui", "main_loop"):
            rule = "服务模块反向依赖了界面或主循环"
        elif source_layer == "ui" and target_layer == "main_loop":
            rule = "界面依赖了主循环"
        if rule:
            checks.append({"type": "layer", "from": edge["from"], "to": edge["to"], "rule": rule,
                           "count": edge["count"], "items": edge["items"][:12], "sites": edge["sites"]})
    graph = defaultdict(set)
    for edge in edge_list:
        if layer_by_id[edge["from"]] == "service" and layer_by_id[edge["to"]] == "service":
            graph[edge["from"]].add(edge["to"])
    service_nodes = {unit["id"] for unit in unit_list if unit["layer"] == "service"}
    cycles = [sorted(component) for component in strongly_connected(graph, service_nodes) if len(component) > 1]
    for component in cycles:
        members = set(component)
        inner = [edge for edge in edge_list if edge["from"] in members and edge["to"] in members]
        mutual = sorted({tuple(sorted((edge["from"], edge["to"]))) for edge in inner
                         if any(other["from"] == edge["to"] and other["to"] == edge["from"] for other in inner)})
        checks.append({"type": "cycle", "members": component, "edges": len(inner),
                       "mutual": [list(pair) for pair in mutual],
                       "rule": "服务模块之间存在循环依赖（规则：出现双向依赖说明需要提取方法或合并服务）"})

    # Module tree.
    lines_by_module = {file["module"]: file["lines"] for file in raw["files"]}
    file_by_module = {file["module"]: file["file"] for file in raw["files"]}
    children = defaultdict(list)
    for module in raw["mods"]:
        if module["test"]:
            continue
        children[module["parent"]].append(module["module"])

    def build(module):
        name = module.split("::")[-1]
        node = {"name": name, "module": module, "unit": short(unit_of(module)),
                "layer": layer_of(unit_of(module)), "lines": lines_by_module.get(module, 0),
                "file": file_by_module.get(module)}
        kids = sorted(children.get(module, []))
        if kids:
            node["children"] = [build(kid) for kid in kids]
        node["total"] = node["lines"] + sum(kid["total"] for kid in node.get("children", []))
        return node

    tree = build("crate")

    production_items = sum(len(unit["items"]) for unit in unit_list)
    described = sum(1 for unit in unit_list for item in unit["items"] if item["d"])
    data = {
        "snapshot": snapshot,
        "generatedAt": datetime.datetime.now().strftime("%Y-%m-%d %H:%M"),
        "stats": {
            "files": len(raw["files"]),
            "lines": sum(file["lines"] for file in raw["files"]),
            "units": len(unit_list),
            "edges": len(edge_list),
            "items": production_items,
            "described": described,
            "findings": len(agent_findings),
        },
        "layers": LAYERS,
        "units": unit_list,
        "edges": edge_list,
        "checks": checks,
        "findings": agent_findings,
        "tree": tree,
        "flow": flow,
    }
    os.makedirs(os.path.dirname(output), exist_ok=True)
    with open(output, "w", encoding="utf-8") as handle:
        handle.write("window.ARCH_DATA = window.ARCH_DATA || {};\n")
        handle.write(f"window.ARCH_DATA[{json.dumps(snapshot)}] = ")
        json.dump(data, handle, ensure_ascii=False, separators=(",", ":"))
        handle.write(";\n")
    print(f"wrote {output}: {data['stats']}, checks={len(checks)}, cycles={len(cycles)}")


if __name__ == "__main__":
    main()
