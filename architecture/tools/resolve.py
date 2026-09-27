"""Resolve raw extractor output into module-level dependency data.

Reads temp/audit/raw.json, resolves every internal `use` and qualified path
to the module that actually defines the referenced item (following
re-exports), and writes temp/audit/resolved.json.
"""
import json
import os
import sys
from collections import defaultdict

AUDIT = os.environ.get("ARCH_AUDIT_DIR", "temp/audit")
RAW = os.path.join(AUDIT, "raw.json")
OUT = os.path.join(AUDIT, "resolved.json")

DEF_KINDS = {"fn", "struct", "enum", "trait", "const", "static", "type", "macro", "union"}


def load():
    with open(RAW, encoding="utf-8") as handle:
        return json.load(handle)


class Resolver:
    def __init__(self, raw):
        self.modules = {"crate"}
        self.children = defaultdict(dict)  # module -> {name: child module}
        self.defs = defaultdict(set)  # module -> item names
        self.bindings = defaultdict(dict)  # module -> {alias: segs}
        self.globs = defaultdict(list)  # module -> [segs]
        for module in raw["mods"]:
            self.modules.add(module["module"])
            self.children[module["parent"]][module["name"]] = module["module"]
        for item in raw["items"]:
            if item["kind"] in DEF_KINDS and item["parent"] is None:
                self.defs[item["module"]].add(item["name"])
        for use in raw["uses"]:
            if use["local"]:
                continue
            if use["alias"] == "*":
                self.globs[use["module"]].append(use["segs"][:-1])
            elif use["alias"] != "_":
                self.bindings[use["module"]][use["alias"]] = use["segs"]

    @staticmethod
    def parent(module):
        return module.rsplit("::", 1)[0] if "::" in module else None

    def start(self, first, module):
        """Resolve the first path segment to a module or a binding target."""
        if first == "crate":
            return ("module", "crate")
        if first == "self":
            return ("module", module)
        if first == "super":
            return ("module", self.parent(module))
        return None

    def resolve(self, segs, module, depth=0):
        """Return (defining_module, item_name) or ("module", module_path) or None."""
        if depth > 20 or not segs:
            return None
        head = self.start(segs[0], module)
        index = 1
        if head is None:
            # A name bound in this module: child module, definition or import.
            name = segs[0]
            if name in self.children[module]:
                head = ("module", self.children[module][name])
            elif name in self.defs[module]:
                return ("item", module, name)
            elif name in self.bindings[module]:
                target = self.resolve(self.bindings[module][name], module, depth + 1)
                if target is None:
                    return None
                if target[0] == "item":
                    return target
                head = target
            else:
                for glob in self.globs[module]:
                    glob_target = self.resolve(glob, module, depth + 1)
                    if glob_target and glob_target[0] == "module":
                        found = self.lookup(glob_target[1], name, depth + 1)
                        if found:
                            if found[0] == "item":
                                return found
                            head = found
                            break
                if head is None:
                    return None
        current = head[1]
        while index < len(segs):
            name = segs[index]
            if name == "super":
                current = self.parent(current)
                index += 1
                continue
            if name == "self":
                index += 1
                continue
            found = self.lookup(current, name, depth + 1)
            if found is None:
                return None
            if found[0] == "item":
                return found
            current = found[1]
            index += 1
        return ("module", current)

    def lookup(self, module, name, depth):
        if module is None:
            return None
        if name in self.children[module]:
            return ("module", self.children[module][name])
        if name in self.defs[module]:
            return ("item", module, name)
        if name in self.bindings[module]:
            return self.resolve(self.bindings[module][name], module, depth + 1)
        for glob in self.globs[module]:
            glob_target = self.resolve(glob, module, depth + 1)
            if glob_target and glob_target[0] == "module" and glob_target[1] != module:
                found = self.lookup(glob_target[1], name, depth + 1)
                if found:
                    return found
        return None


def main():
    raw = load()
    resolver = Resolver(raw)
    references = []
    workspace_path = os.path.join(AUDIT, "workspace.json")
    workspace = None
    if os.path.exists(workspace_path):
        with open(workspace_path, encoding="utf-8") as handle:
            workspace = json.load(handle)
    unresolved = defaultdict(int)
    internal_roots = {"crate", "super", "self"}
    for use in raw["uses"]:
        segs = use["segs"]
        if segs[-1] == "*":
            segs = segs[:-1]
        if not segs:
            continue
        if segs[0] not in internal_roots and segs[0] not in resolver.children[use["module"]] and segs[0] not in resolver.bindings[use["module"]]:
            continue
        target = resolver.resolve(segs, use["module"])
        if target is None:
            unresolved["::".join(use["segs"])] += 1
            continue
        references.append({
            "from": use["module"],
            "file": use["file"],
            "line": use["line"],
            "test": use["test"],
            "via": "use",
            "reexport": use["vis"] != "" and not use["local"],
            "target_module": target[1],
            "target_item": target[2] if target[0] == "item" else None,
        })
    for path in raw["paths"]:
        segs = path["segs"]
        if len(segs) < 2:
            continue
        first = segs[0]
        module = path["module"]
        if first not in internal_roots and first not in resolver.children[module] and not (first in resolver.bindings[module] and resolver.bindings[module][first][0] in internal_roots):
            continue
        target = resolver.resolve(segs, module)
        if target is None:
            unresolved["::".join(segs)] += 1
            continue
        references.append({
            "from": module,
            "file": path["file"],
            "line": path["line"],
            "test": path["test"],
            "via": "path",
            "reexport": False,
            "target_module": target[1],
            "target_item": target[2] if target[0] == "item" else None,
        })

    if workspace:
        for use in raw["uses"]:
            source_package = workspace["modulePackages"].get(use["module"])
            if source_package is None or not use["segs"]:
                continue
            target_package = workspace["dependencies"].get(source_package, {}).get(use["segs"][0])
            if target_package is None:
                continue
            references.append({
                "from": use["module"],
                "file": use["file"],
                "line": use["line"],
                "test": use["test"],
                "via": "workspace_use",
                "reexport": use["vis"] != "" and not use["local"],
                "target_module": workspace["packageModules"][target_package],
                "target_item": next((part for part in reversed(use["segs"][1:]) if part != "*"), None),
            })

        for path in raw["paths"]:
            source_package = workspace["modulePackages"].get(path["module"])
            if source_package is None or not path["segs"]:
                continue
            target_package = workspace["dependencies"].get(source_package, {}).get(path["segs"][0])
            if target_package is None:
                continue
            references.append({
                "from": path["module"],
                "file": path["file"],
                "line": path["line"],
                "test": path["test"],
                "via": "workspace_path",
                "reexport": False,
                "target_module": workspace["packageModules"][target_package],
                "target_item": path["segs"][-1],
            })
    result = {
        "references": references,
        "unresolved": sorted(unresolved.items(), key=lambda pair: -pair[1]),
    }
    with open(OUT, "w", encoding="utf-8") as handle:
        json.dump(result, handle, ensure_ascii=False)
    print("references:", len(references))
    print("unresolved (top 40):")
    for name, count in result["unresolved"][:40]:
        print(f"  {count:4d} {name}")


if __name__ == "__main__":
    sys.exit(main())
