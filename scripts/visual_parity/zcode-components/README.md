# Conversation workflow component reference

This is a development-only reference harness. It imports the pinned ZCode
`WorkflowCardHeader`, `WorkflowTimeline`, theme CSS and providers directly.
It is not bundled into the Rust/GPUI application.

Run `python3 scripts/visual_parity/zcode-components/prepare.py /tmp/codex-native-zcode-reference`
from the native repository, then start Vite using the printed command from the
reference checkout (its dependencies must already be installed). Open port 5188
with `?theme=light` or `?theme=dark`.

The native `multi-agent-running.json` fixture is adapted into timeline display
data. This does not exercise the reference runtime or its graph projection.
The wrapper copies the digest surface classes, but is not `WorkflowRunDigest`;
header actions are diagnostic placeholders. Do not use this harness as proof
of working Stop, child transcript navigation, collapse or full desktop parity.

Capture with reduced motion, wait for `document.fonts.ready` and the
`workflow-card-header` test ID, then save at 1180×760 and 480×760 for both themes.
Record source revision, fixture hash, viewport, theme, and these limitations.
The 2026-10-24 captures are under
`docs/visual-parity/runs/zcode-3.16.2/component-reference-20261024/`.

Observed header heights: 28px wide, 57px narrow. In the narrow reference the
right information cluster wraps and offscreen stages become edge lamps.
Native screenshots must use the same fixture and content width before any pixel
error is reported. Previous full-window screenshots cannot supply that proof.
