#!/usr/bin/env python3
"""Extract a small, auditable UI contract from a pinned ZCode checkout."""

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def css_block(css, selector):
    match = re.search(rf"{re.escape(selector)}\s*\{{(?P<body>.*?)\}}", css, re.DOTALL)
    return match.group("body") if match else ""


def css_variables(body):
    return {
        name: value.strip()
        for name, value in re.findall(r"^[ \t]*(--[\w-]+):\s*([^;]+);", body, re.MULTILINE)
    }


def token(variables, name):
    return variables.get(name)


def resolve(value, variables):
    if value is None:
        return None
    seen = set()
    while value.startswith("var("):
        match = re.fullmatch(r"var\((--[\w-]+)\)", value)
        if not match or match.group(1) in seen:
            break
        seen.add(match.group(1))
        value = variables.get(match.group(1), value)
    return value


def extract(source):
    source = source.resolve()
    commit = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    package = json.loads((source / "package.json").read_text())
    css = (source / "packages/ui/src/styles.css").read_text()
    root = css_variables(css_block(css, ":root"))
    root.update(css_variables(css_block(css, "@theme")))
    theme = css_variables(css_block(css, ".theme-zai-light"))
    theme.update({name: value for name, value in root.items() if name not in theme})
    dark = css_variables(css_block(css, ".theme-zai-dark"))
    dark.update({name: value for name, value in root.items() if name not in dark})
    license_hash = sha256(source / "LICENSE")
    return {
        "schema_version": 1,
        "source": "zai-org/ZCode",
        "commit": commit,
        "version": package.get("version"),
        "license": "Apache-2.0",
        "license_sha256": license_hash,
        "ui_source": "packages/ui/src/styles.css",
        "desktop_source": "packages/desktop",
        "tokens": {
            "font_size": token(theme, "--ui-font-size"),
            "light_background": resolve(token(theme, "--color-background"), theme),
            "light_sidebar": resolve(token(theme, "--color-sidebar"), theme),
            "light_panel": resolve(token(theme, "--color-panel"), theme),
            "light_border": resolve(token(theme, "--color-border"), theme),
            "light_selected": resolve(token(theme, "--color-selected"), theme),
            "light_primary": resolve(token(theme, "--color-primary"), theme),
            "light_foreground": resolve(token(theme, "--color-foreground"), theme),
            "light_input": resolve(token(theme, "--color-input"), theme),
            "light_focus": resolve(token(theme, "--color-input-border-focused"), theme),
            "dark_background": resolve(token(dark, "--color-background"), dark),
            "dark_sidebar": resolve(token(dark, "--color-sidebar"), dark),
            "dark_panel": resolve(token(dark, "--color-panel"), dark),
            "dark_border": resolve(token(dark, "--color-border"), dark),
            "dark_selected": resolve(token(dark, "--color-selected"), dark),
            "dark_primary": resolve(token(dark, "--color-primary"), dark),
            "dark_foreground": resolve(token(dark, "--color-foreground"), dark),
            "dark_input": resolve(token(dark, "--color-input"), dark),
            "dark_focus": resolve(token(dark, "--color-input-border-focused"), dark),
            "settings_sidebar_wide": "268px",
            "settings_sidebar_compact": "68px",
            "settings_header": "48px",
            "settings_control": "32px",
            "settings_radius": "12px",
        },
        "implementation": {
            "target": "codex-native/crates/pi-desktop",
            "renderer": "GPUI native; no copied React/Electron runtime",
            "codex_behavior_reference": "local reverse contract and captured Codex 21477 frames",
        },
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = extract(args.source)
    with args.output.open("x") as stream:
        json.dump(result, stream, indent=2)
        stream.write("\n")
    print(json.dumps(result, indent=2))
