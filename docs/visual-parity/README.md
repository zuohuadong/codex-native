# Native visual parity

ZCode 3.16.2 at commit `3c5b227649371507ddc4eb46ac5133afe6db1cab` is the current visual authority, implemented in pure GPUI. The commands below and `contract.json` are the **historical Codex 21477 capture harness**. They must not be used to claim ZCode parity. A source-build identity adapter and reviewed matching ZCode/native frames are still required. See `zcode-reference.md` for source evidence.

The comparison is deliberately strict. Each capture needs a JSON sidecar containing the pinned reference identity, viewport, scale, theme, locale, state, fixture hash, platform, crop, timestamp, image hash, and artifact hash. The tool refuses to resize, auto-align, mask, or accept a screenshot captured from a different release.

Probe the installed app before each capture. The probe must report `IDENTITY_MATCH`:

```sh
python3 scripts/visual_parity/probe_reference.py \
  --app /Applications/ChatGPT.app \
  --contract docs/visual-parity/contract.json \
  --output docs/visual-parity/installed-reference-21477-YYYY-MM-DD.json
```

Capture metadata is created only when the target window stays visible, its geometry and artifact hash remain unchanged, and the window belongs to the expected app/binary. The capture helper preserves a raw window image when macOS supplies transparent rounded corners and uses an explicit white matte for comparison.

Create a fixture such as:

```json
{"theme":"light","locale":"en-US","state":"settings"}
```

Then capture the reference and candidate independently:

```sh
python3 scripts/visual_parity/capture.py --contract docs/visual-parity/contract.json \
  --fixture fixture-settings.json --window-id CODEX_WINDOW_ID --role reference \
  --output docs/visual-parity/runs/21477/codex-settings

python3 scripts/visual_parity/capture.py --contract docs/visual-parity/contract.json \
  --fixture fixture-settings.json --window-id VOLT_WINDOW_ID --role candidate \
  --binary /tmp/codex-native-target/debug/volt \
  --output docs/visual-parity/runs/21477/volt-settings
```

Inspect each saved PNG before comparing. The capture tool records declarations as `UNREVIEWED`; it does not infer the page from a keypress. After confirming the actual page, theme, language, data and scroll position against the fixture, set `review_status` to `VERIFIED` and `reviewed_image_sha256` to the inspected `image_sha256`. A wrong page must be marked `REJECTED`, not approved because the keypress was sent. This review records a visual observation, not permission for a remote operation.

Compare matching 21477 captures:

```sh
python3 scripts/visual_parity/compare.py \
  --contract docs/visual-parity/contract.json \
  --reference reference.json \
  --candidate candidate.json \
  --output docs/visual-parity/runs/2026-10-24-settings
```

Exit code `0` means decoded RGB pixels are identical, `1` means pixels differ, and `2` means the comparison is blocked by missing or inconsistent evidence. A passing frame is only a per-state visual result; interaction, accessibility, IME, backend, and release acceptance remain separate. A metadata file is an auditable declaration, not cryptographic proof that the capture operator selected the right fixture.

Generate the 12-state × 3-size × 2-theme × 2-locale capture matrix:

```sh
python3 scripts/visual_parity/matrix.py --contract docs/visual-parity/contract.json \
  --captures docs/visual-parity/runs/21477 --output docs/visual-parity/matrix-21477.json
```

Raw captures and reports under `runs/` are local and ignored by Git because native windows may contain account/workspace content. The small metadata inventory is retained. Local file links in this inventory require the original captures. Each run/output must use a new name.
