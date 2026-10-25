#!/usr/bin/env python3
"""Create deterministic offline workflow snapshots for visual state coverage."""
from __future__ import annotations

import copy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
FIXTURES = ROOT / "fixtures"
BASE = json.loads((FIXTURES / "multi-agent.json").read_text())


def write(name: str, snapshot: dict) -> None:
    snapshot["title"] = f"{snapshot['title']} · {name}"
    snapshot["delivery"] = (
        "仅用于检查多 Agent 状态布局，不代表真实 Agent 执行结果。"
    )
    for stage in snapshot["stages"]:
        observed = sum(node["status"] != "waiting" for node in stage["nodes"])
        settled = sum(node["status"] in {"completed", "failed", "interrupted"} for node in stage["nodes"])
        stage["progress"] = {"settled": settled, "observed": observed} if observed else None
    (FIXTURES / f"multi-agent-{name}.json").write_text(
        json.dumps(snapshot, ensure_ascii=False, indent=2) + "\n"
    )


def running() -> dict:
    value = copy.deepcopy(BASE)
    value["status"] = "running"
    for stage_index, stage in enumerate(value["stages"]):
        stage["status"] = "running" if stage_index == 0 else "waiting"
        for node_index, node in enumerate(stage["nodes"]):
            node["status"] = (
                ("running" if node_index < 2 else "completed")
                if stage_index == 0 else "waiting"
            )
    value["changes"] = []
    return value


def failed() -> dict:
    value = running()
    value["status"] = "failed"
    value["stages"][0]["status"] = "completed"
    for node in value["stages"][0]["nodes"]:
        node["status"] = "completed"
    value["stages"][1]["status"] = "failed"
    value["stages"][1]["nodes"][0]["status"] = "failed"
    value["stages"][1]["nodes"][0]["detail"] = "离线样例：模拟 Agent 失败状态。"
    return value


def interrupted() -> dict:
    value = running()
    value["status"] = "interrupted"
    value["stages"][0]["status"] = "interrupted"
    value["stages"][0]["nodes"][0]["status"] = "completed"
    value["stages"][0]["nodes"][1]["status"] = "interrupted"
    return value


if __name__ == "__main__":
    write("running", running())
    write("failed", failed())
    write("interrupted", interrupted())
