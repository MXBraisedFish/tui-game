"""Regenerate the current workspace architecture from Cargo metadata.

Run from any directory with Python 3 and Rust/Cargo installed:
    python architecture/build_current.py

The Rust source extractor lives under architecture/tools so the snapshot does
not depend on ignored temp/audit files. Generated intermediate JSON stays under
temp/p_plan_audit/architecture.
"""

import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "temp" / "p_plan_audit" / "architecture"
EXTRACTOR_MANIFEST = ROOT / "architecture" / "tools" / "extractor" / "Cargo.toml"
RESOLVER = ROOT / "architecture" / "tools" / "resolve.py"
SITE_BUILDER = ROOT / "architecture" / "tools" / "build_site.py"
OUTPUT = ROOT / "architecture" / "data" / "current.js"


def run(command, *, cwd=ROOT, env=None, capture=False):
    result = subprocess.run(
        command,
        cwd=cwd,
        env=env,
        check=False,
        text=True,
        encoding="utf-8",
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )
    if result.returncode:
        detail = "\n".join(part for part in (result.stdout, result.stderr) if part)
        raise RuntimeError(f"command failed ({result.returncode}): {command}\n{detail}")
    return result.stdout if capture else ""


def layer_for(package_name):
    if package_name.startswith("tg-core-"):
        return "core"
    if package_name.startswith("tg-service-"):
        return "service"
    return "main_loop"


def package_slug(package_name):
    return package_name.replace("-", "_")


def package_module(package_name):
    return f"crate::workspace::{layer_for(package_name)}::{package_slug(package_name)}"


def module_path(module, package_root):
    if module == "crate":
        return package_root
    return package_root + module[len("crate"):]


def rebase_path(path, package_root):
    segments = path["segs"]
    if segments and segments[0] == "crate":
        path["segs"] = package_root.split("::") + segments[1:]


def add_module(modules, parent, name, module, *, inline=True, file=None, doc=""):
    modules.append({
        "parent": parent,
        "name": name,
        "module": module,
        "vis": "",
        "line": 0,
        "inline": inline,
        "test": False,
        "file": file,
        "doc": doc,
    })


def flow_data():
    return {
        "title": "当前架构 · 生命周期",
        "columns": ["故障分支", "阶段主流程", "帧循环"],
        "groups": [
            {"id": "g_boot", "title": "启动阶段 boot::prepare()", "ref": "src/host_engine/boot/mod.rs:14", "col": [1, 1], "row": [2, 3]},
            {"id": "g_runtime", "title": "运行阶段 runtime::run()", "ref": "src/host_engine/runtime/mod.rs:7", "col": [1, 1], "row": [4, 8]},
            {"id": "g_frame", "title": "应用层帧循环 app::runtime::run()", "ref": "src/host_engine/app/runtime/mod.rs:360", "col": [2, 2], "row": [6, 6]},
            {"id": "g_shutdown", "title": "退出阶段 shutdown::close()", "ref": "src/host_engine/shutdown/mod.rs:7", "col": [1, 1], "row": [9, 10]},
        ],
        "nodes": [
            {"id": "start", "col": 1, "row": 0, "type": "start", "label": "程序入口 main()", "detail": "入口调用 host_engine::run()；阶段编排集中在 host_engine 根模块。", "ref": "src/main.rs:3"},
            {"id": "engine", "col": 1, "row": 1, "type": "process", "label": "安装崩溃钩子并初始化阶段", "detail": "安装终端恢复钩子，记录 Init 阶段，再调用 boot::prepare()。", "ref": "src/host_engine/mod.rs:23"},
            {"id": "boot", "col": 1, "row": 2, "type": "process", "label": "准备服务、世界与终端", "detail": "EngineServices 和 RuntimeWorld 由应用层拥有；读取终端能力，进入终端后完成语言初始化、包扫描和后台监听。", "ref": "src/host_engine/boot/mod.rs:14"},
            {"id": "boot_result", "col": 1, "row": 3, "type": "decision", "label": "启动阶段是否返回故障？", "detail": "BootOutput 携带已创建的服务、运行时世界和可选 HostFault，供主循环决定运行或进入异常页。", "ref": "src/host_engine/app/boot_output.rs:5"},
            {"id": "boot_fault", "col": 0, "row": 4, "type": "fault", "label": "显示异常页并有序关闭", "detail": "记录启动故障，运行异常退出页，然后进入统一关闭阶段。", "ref": "src/host_engine/mod.rs:28"},
            {"id": "runtime_facade", "col": 1, "row": 4, "type": "process", "label": "运行阶段入口委托应用层", "detail": "runtime::run 只转交 EngineServices 和 RuntimeWorld 给 app::run；HostFault 由主循环捕获。", "ref": "src/host_engine/runtime/mod.rs:7"},
            {"id": "app_runtime", "col": 1, "row": 5, "type": "process", "label": "应用层准备状态与界面对象", "detail": "加载宿主按键和存档设置，创建帧调度器及页面、覆盖层、Lua 事件路由状态。", "ref": "src/host_engine/app/runtime/mod.rs:360"},
            {"id": "frame_loop", "col": 2, "row": 6, "type": "loop", "label": "每帧处理输入、事件、更新与呈现", "detail": "推进时钟和服务；收取异步事件与输入；路由 UI/Lua；更新页面和会话；合成并呈现终端帧；等待下一帧。", "ref": "src/host_engine/app/runtime/mod.rs:560"},
            {"id": "runtime_fault", "col": 0, "row": 7, "type": "decision", "label": "运行阶段是否发生 HostFault？", "detail": "运行阶段的受监督故障进入异常页；正常退出则直接关闭。", "ref": "src/host_engine/mod.rs:49"},
            {"id": "runtime_fault_page", "col": 0, "row": 8, "type": "fault", "label": "显示异常退出页", "detail": "异常页仍由应用层运行，并返回统一 ExitState。", "ref": "src/host_engine/app/runtime/mod.rs:884"},
            {"id": "shutdown", "col": 1, "row": 9, "type": "io", "label": "停止后台工作并恢复终端", "detail": "关闭写入入口、托管线程、音频和异步运行时；停止游戏与屏保会话，释放输入法并退出终端模式。", "ref": "src/host_engine/shutdown/mod.rs:7"},
            {"id": "end", "col": 1, "row": 10, "type": "end", "label": "状态进入 Stopped 并退出", "detail": "RuntimeWorld 状态机标记 Stopped，main 返回。", "ref": "src/host_engine/shutdown/mod.rs:42"},
        ],
        "edges": [
            {"from": "start", "to": "engine"},
            {"from": "engine", "to": "boot"},
            {"from": "boot", "to": "boot_result"},
            {"from": "boot_result", "to": "boot_fault", "kind": "fault", "label": "有"},
            {"from": "boot_result", "to": "runtime_facade", "label": "无"},
            {"from": "boot_fault", "to": "shutdown", "kind": "fault"},
            {"from": "runtime_facade", "to": "app_runtime"},
            {"from": "app_runtime", "to": "frame_loop"},
            {"from": "frame_loop", "to": "runtime_fault"},
            {"from": "runtime_fault", "to": "runtime_fault_page", "kind": "fault", "label": "有"},
            {"from": "runtime_fault", "to": "shutdown", "label": "无"},
            {"from": "runtime_fault_page", "to": "shutdown", "kind": "fault"},
            {"from": "frame_loop", "to": "frame_loop", "kind": "loop", "label": "下一帧"},
            {"from": "shutdown", "to": "end"},
        ],
    }


def main():
    AUDIT.mkdir(parents=True, exist_ok=True)
    target_dir = AUDIT / "extractor_target"
    cargo = shutil.which("cargo") or "cargo"
    python = sys.executable

    metadata_text = run([cargo, "metadata", "--format-version", "1", "--no-deps"], capture=True)
    metadata = json.loads(metadata_text)
    member_ids = set(metadata["workspace_members"])
    packages = [package for package in metadata["packages"] if package["id"] in member_ids]
    packages.sort(key=lambda package: package["name"])
    package_by_name = {package["name"]: package for package in packages}

    run([
        cargo,
        "build",
        "--manifest-path",
        str(EXTRACTOR_MANIFEST),
        "--target-dir",
        str(target_dir),
        "--locked",
    ])
    executable = target_dir / "debug" / ("arch_extractor.exe" if os.name == "nt" else "arch_extractor")
    if not executable.exists():
        raise RuntimeError(f"Rust extractor binary not found: {executable}")

    raw = {key: [] for key in ("files", "mods", "items", "uses", "paths", "errors")}
    workspace_module = "crate::workspace"
    add_module(raw["mods"], "crate", "workspace", workspace_module)
    layers = ("core", "service", "main_loop")
    for layer in layers:
        add_module(raw["mods"], workspace_module, layer, f"{workspace_module}::{layer}")

    package_roots = {package["name"]: package_module(package["name"]) for package in packages}
    descriptions = {}
    dependency_aliases = {}
    module_packages = {}

    for package in packages:
        name = package["name"]
        layer = layer_for(name)
        slug = package_slug(name)
        package_root = package_roots[name]
        layer_root = f"{workspace_module}::{layer}"
        targets = [
            target for target in package["targets"]
            if "lib" in target["kind"] or "bin" in target["kind"]
        ]
        if not targets:
            raise RuntimeError(f"workspace package has no lib/bin target: {name}")

        add_module(raw["mods"], layer_root, slug, package_root, doc=package.get("description") or "")
        root_for_target = {}
        for target in targets:
            target_root = package_root
            if len(targets) > 1:
                target_slug = package_slug(target["name"])
                target_root = f"{package_root}::{target_slug}"
                add_module(raw["mods"], package_root, target_slug, target_root)
            root_for_target[target["src_path"]] = target_root

        for target in targets:
            source = target["src_path"]
            target_root = root_for_target[source]
            extracted_text = run([str(executable), source, str(ROOT)], capture=True)
            extracted = json.loads(extracted_text)
            raw["errors"].extend(extracted["errors"])

            for key in ("files", "mods", "items", "uses", "paths"):
                records = extracted[key]
                for record in records:
                    if "module" in record:
                        record["module"] = module_path(record["module"], target_root)
                    if key == "mods" and "parent" in record:
                        record["parent"] = module_path(record["parent"], target_root)
                    if key == "uses" or key == "paths":
                        rebase_path(record, target_root)
                raw[key].extend(records)

        unit_id = f"{layer}/{slug}"
        package_description = package.get("description") or f"工作区包 {name}。"
        target_summary = ", ".join("/".join(target["kind"]) for target in targets)
        descriptions[unit_id] = {
            "summary": package_description,
            "notes": f"Cargo 包：{name}；生产目标：{target_summary}。",
        }
        module_packages[package_root] = name

        aliases = {}
        for dependency in package["dependencies"]:
            if dependency.get("kind") not in (None, "normal", "build"):
                continue
            dependency_name = dependency["name"]
            if dependency_name not in package_by_name:
                continue
            alias = dependency.get("rename") or dependency_name
            aliases[alias.replace("-", "_")] = dependency_name
        dependency_aliases[name] = aliases

    if raw["errors"]:
        sample = json.dumps(raw["errors"][:12], ensure_ascii=False, indent=2)
        raise RuntimeError(f"extractor reported {len(raw['errors'])} errors:\n{sample}")

    for records in (raw["files"], raw["mods"], raw["items"], raw["uses"], raw["paths"]):
        for record in records:
            module = record.get("module")
            if module:
                owner = max((root for root in package_roots.values() if module == root or module.startswith(root + "::")), key=len, default=None)
                if owner:
                    module_packages[module] = next(name for name, root in package_roots.items() if root == owner)

    environment = os.environ.copy()
    environment["ARCH_AUDIT_DIR"] = str(AUDIT)
    (AUDIT / "raw.json").write_text(json.dumps(raw, ensure_ascii=False), encoding="utf-8")
    (AUDIT / "workspace.json").write_text(json.dumps({
        "packageModules": package_roots,
        "modulePackages": module_packages,
        "dependencies": dependency_aliases,
    }, ensure_ascii=False, indent=2), encoding="utf-8")
    (AUDIT / "flow.json").write_text(json.dumps(flow_data(), ensure_ascii=False, indent=2), encoding="utf-8")
    descriptions_path = AUDIT / "desc" / "B_workspace.json"
    descriptions_path.parent.mkdir(parents=True, exist_ok=True)
    descriptions_path.write_text(json.dumps({"units": descriptions}, ensure_ascii=False, indent=2), encoding="utf-8")

    run([python, str(RESOLVER)], env=environment)
    run([python, str(SITE_BUILDER), "current", str(OUTPUT)], env=environment)

    generated = OUTPUT.read_text(encoding="utf-8")
    match = re.fullmatch(r'.*?window\.ARCH_DATA\["current"\] = (.*);\s*', generated, re.DOTALL)
    if not match:
        raise RuntimeError("generated current.js has an unexpected format")
    data = json.loads(match.group(1))
    expected_packages = {package["name"] for package in packages}
    actual_package_ids = {unit["id"].split("/", 1)[1] for unit in data["units"] if unit["id"].count("/") == 1}
    expected_ids = {package_slug(name) for name in expected_packages}
    if actual_package_ids != expected_ids:
        raise RuntimeError(f"snapshot package coverage mismatch: missing={sorted(expected_ids - actual_package_ids)}, extra={sorted(actual_package_ids - expected_ids)}")
    if not data["stats"]["files"] or not data["stats"]["items"] or not data["edges"]:
        raise RuntimeError("generated snapshot is missing source files, items, or cross-package edges")

    print(f"workspace members: {len(packages)}")
    print(f"production source files: {data['stats']['files']}; lines: {data['stats']['lines']}")
    print(f"package nodes: {data['stats']['units']}; cross-package edges: {data['stats']['edges']}; items: {data['stats']['items']}")
    print(f"dependency checks: {len(data['checks'])}; flow nodes: {len(data['flow']['nodes'])}")
    print(f"wrote {OUTPUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
