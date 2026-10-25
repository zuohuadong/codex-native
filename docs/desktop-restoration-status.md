# Desktop restoration continuation — 2026-10-24

Status: **PARTIAL / ZCode 3.16.2 source alignment; matched visual acceptance remains incomplete**.
Parent: the earlier desktop implementation in this working tree.
Source: user's explicit “全部实施” continuation. Target: `codex-native`; current visual authority: ZCode 3.16.2 at `3c5b227649371507ddc4eb46ac5133afe6db1cab`. Earlier `volt-native` reverse-engineering and Codex captures remain historical implementation evidence.
No commit, push, deployment, or whole-product parity claim is included.

## Current acceptance audit

The full objective remains complete visual and interaction alignment in pure GPUI, with light default and dark/system choices. No pixel-parity PASS exists. The older Codex comparison percentages below cannot measure ZCode fidelity.

| Requirement | Current evidence | Remaining proof/work |
| --- | --- | --- |
| Reference authority | Pinned ZCode source and `DESIGN.md` | Matched ZCode/native capture pairs at identical state, size, scale and theme |
| Fonts, spacing, icons, colors | Native vector controls, source-derived tokens and dimensions | Rendered calibration across the whole interface; generic GPUI control internals remain a possible mismatch |
| Light/dark and responsive layout | Saved theme tests; completed workflow light wide/narrow and dark wide diagnostic PNGs | Fresh final-build captures for both themes, minimum window and dialogs |
| Conversation and multi-agent states | Inline workflow snapshots, persisted state, source-bound disclosures; parent tool dispatches bounded parallel read-only specialists with streamed node detail, deadline, cancellation and terminal snapshots | GUI running/failure/interrupt renders, multi-stage orchestration and edit attribution, dynamic rails and face motion |
| Long messages and tool output | Selectable native text, measured collapse, surface fade, fixed-height raw output | Actual wrapping, selection/copy, collapse and streamed follow/freeze interaction acceptance |
| Settings | Editable validated provider fields, native language/theme selectors and motion switch | Final screenshots, dropdown sizing/focus, remaining reference settings sections |
| Focus, shortcuts and accessibility | Native controls and key bindings | Full keyboard traversal, dialog focus return and Chinese IME GUI checks |
| Persistence | State, revision, queue and disclosure migration tests | GUI restart/recovery replay; live backend events |
| Automations and Skills | Local interval jobs and built-in skill instruction insertion | Full reference interaction breadth and external integrations |
| Completion gate | Desktop/core tests and local build | Whole-product visual and behavioral acceptance; not satisfied by compilation |

The latest independent screen inspection reports `Screen is locked`; it blocks GUI verification, not source work. Preserve that distinction. Current workflow details and capture provenance are maintained in `visual-parity/multi-agent-conversation.md` and `visual-parity/runs/zcode-3.16.2/conversation-20261024/README.md`.

## ZCode source alignment — 2026-10-24

User explicitly selected the open-source ZCode desktop as the primary implementation reference. The source was cloned and pinned to `3c5b227649371507ddc4eb46ac5133afe6db1cab` (`3.16.2`, Apache-2.0). Its UI dependencies installed and the desktop TypeScript host build completed successfully. The Electron runtime was not promoted to a product acceptance gate: the comparison target remains the native Rust/GPUI implementation.

- Extracted the UI token and layout manifest at [zcode-reference.json](visual-parity/zcode-reference.json) and documented the source boundary at [zcode-reference.md](visual-parity/zcode-reference.md).
- Applied ZCode light/dark surface, sidebar, text, border, selected-surface, and success colors to `pi-desktop`.
- Changed the native sidebar width to ZCode's measured 268px desktop value.
- Replaced the single settings form with a ZCode-shaped settings shell: 268px category navigation, 68px compact navigation, 48px header, 12px panel radius, grouped General / Appearance / Model providers / Shortcuts sections, and existing save/validation actions preserved.
- The post-change native settings capture was visually checked and compared against the current Codex 21477 settings frame. It remains `FAIL_PIXELS` at 3,126,049 changed pixels / 3,587,200 (87.1445%) because the products expose different settings information architecture and content. The result is a diagnostic, not a completion percentage.
- Native desktop tests (12) and build/check passed after the layout change. The ZCode host bundle build also passed.

## Historical Codex baseline migration — 2026-10-24 (superseded by ZCode)

User explicitly adopted the installed version after the reference-version discussion. At that stage, `docs/visual-parity/contract.json` selected build 21477; this is now a historical Codex contract, not a ZCode acceptance contract; `contract-8109.json` preserves the historical contract unchanged. This follow-up changes reference authority, not the scope of whole-product restoration.

- Bundle version and app.asar hash revalidated: `IDENTITY_MATCH`.
- Captured and visually inspected three actual reference settings frames: 1180×760, 600×620, and 480×600, all scale 2, light, zh-CN. A request for 480×480 was clamped to 480×600 by the reference app.
- Captured the native settings frame from an isolated state directory with provider keys removed from the test process environment.
- The valid 1180×760 settings comparison reports `FAIL_PIXELS`: 1,343,069 changed pixels / 3,587,200 (37.4406%). This is an initial full-frame diagnostic including different settings contents and window chrome, not a fidelity percentage or proof of matched focus/IME states.
- An earlier candidate capture was the empty-chat page despite sending Cmd+comma. It was marked REJECTED after inspection, and its comparison marked INVALIDATED. The comparator now requires image-bound visual-review evidence before calculating a result.
- `matrix-21477.json` tracks 144 target combinations: 3 reference rows captured and 1 candidate row captured. All other rows remain missing; no whole matrix PASS is claimed.
- 11 comparison tests, Python compilation, Swift type checking, and diff whitespace validation passed. No Rust behavior changed in this baseline migration.

See `docs/visual-parity/baseline-21477.md` for local screenshot/report links. The screen-lock limitation below describes the previous run, not this successful capture session.

## Implemented in this continuation

- Task-local queue, pause after failure/stop, explicit resume/removal, successful-run progression, restart-safe pending work, persisted error/token state.
- Read-only Git Review: unstaged/staged/untracked categories, file filtering, actual diffs, refresh and retryable loading errors. A bad untracked preview retains the file list. Unknown/non-repository state is not displayed as a clean tree.
- Conversation-bound command output drawer backed by the existing shell tool. This does not implement PTY stdin, interactive terminal programs, or terminal emulation.
- Review and output shortcuts, root focus restoration, narrowed-window layout, field labels, and constrained composer model control.
- Skill actions populate the actual built-in instructions in the composer.
- Provider streaming preserves UTF-8 across network chunks and rejects malformed JSON/schema, incomplete tool arguments, premature disconnects, stalled streams, and truncated/filtered completions. Partial rendered text is retained on error.
- The earlier implementation remains: vector assets, light/dark appearance, editable validated settings, history/drafts, local interval automations, stop/retry, tool details and reduced-motion streaming.

## Verification

- `cargo check -p pi-desktop --offline` and local build.
- Desktop tests cover state round trips, endpoint/automation validation, interrupted tool reconciliation, queue pause/resume, and actual Git fixtures (including paths with spaces, staged changes, broken symlinks, and non-repository errors).
- Core tests include nonzero shell exit and cancellation of descendants, plus local HTTP streaming contract tests for split Chinese/emoji UTF-8 and malformed/incomplete responses.
- Source formatting and `git diff --check`.
- Added `scripts/visual_parity/compare.py` with an identity-bound, lossless PNG comparison. It rejects a mismatched Codex build, viewport, scale, theme, locale, state, fixture, crop, or modified image; it emits JSON, overlay, and changed-pixel artifacts. `scripts/visual_parity/probe_reference.py` records installed bundle identity without silently replacing the locked reference.
- Native screenshots captured from the running local build, with no reference image substitution:
  - [Settings](evidence/desktop-2026-10-24/settings.png): command-comma opened the settings view.
  - [Wide Review](evidence/desktop-2026-10-24/review-wide.png): actual working-tree file list and diff, alongside missing-credential error state.
  - [Narrow Review](evidence/desktop-2026-10-24/review-narrow.png): window resized through macOS accessibility to 600 × 620; navigation collapsed and Review used the available single column.

These captures precede the last focus and parser corrections; they demonstrate the displayed states, not a complete final-build action replay.

## Unfinished acceptance and implementation boundaries

- At the final GUI check, `CGSessionCopyCurrentDictionary` reported `CGSSessionScreenIsLocked=1`. Window screenshots returned “could not create image from window”; synthetic input did not produce the command postcondition. That check is **blocked**, not passed. The test process was stopped; no attempt was made to unlock the system.
- An initial AX capture exposed only window chrome, not a complete semantic tree. Full accessibility labels/order, focus traversal, native Chinese IME composition, final-build settings/automation click-through, command execution, and restart recovery through the GUI remain unverified. Pure state tests are not substitutes for those checks.
- Exact build-8109 reference screenshots/actions remain unavailable. The historical Codex baseline used installed build 21477; ZCode is the current visual authority; dynamic capture and comparison are implemented, but no state is marked PASS until an independently captured matching frame exists. Original icon/font parameters, full transition timings, and authenticated backend postconditions remain separate acceptance evidence.
- The historical installed Codex baseline was probed on 2026-10-24: `26.1016.62342 / build 21477`, app.asar SHA256 `5b5cf52887cfd4de19cb5862155a8ef6cf58830c0a2fc592b72060cfcc3be869`. The previous 8109 contract is archived separately and is not mixed into current pixel comparisons.
- Automations are local interval jobs while the app is open; cloud scheduling, full recurrence semantics, approval/inbox workflows and external integrations are not implemented here.
- Review currently reads changes; staging, commit/revert and remote PR workflows are not implemented here. The command drawer is not a full terminal.
- Cloud/worktree execution, remote handoff, marketplace/MCP management, global Quick Chat, computer use, signing and notarization remain unimplemented in the current local build; this is not a claim that the full requested restoration is complete.

Resume the GUI acceptance on an unlocked macOS session with the matching reference action baseline. Do not promote any of the unverified rows merely because compilation or unit tests pass.


## Conversation messages and raw output follow-up — 2026-10-24

- User bubbles already matched the source geometry and surface; this round extracted the 624px breakpoint, 576px maximum, 32px horizontal inset and 120px + 1px overflow tolerance into pure helpers and added boundary tests. No visual values changed.
- Raw tool output now selects the terminal treatment for `bash`/`execute`/`shell` (14px monospace, 20px line height, 100px cap) and the fallback treatment for other results (12px, 200px cap). The nested output view owns a `ScrollHandle` and consumes wheel events only while it can scroll in that direction; at an edge or when short, the event propagates to the transcript. Unit tests cover the decision rule, but native wheel propagation and scrollbar behavior remain unverified.
- Assistant streaming fade is now limited to the current last assistant row instead of every historical assistant message. Completed messages remain Markdown-rendered and static under reduced motion. Empty assistant rows still have no placeholder text.
- A stopped run is now tracked separately from a failure: cancellation no longer overwrites the stop notice with a worker join error, the final status is “Stopped”, and the notice uses the neutral subtle color instead of the red failure color. Provider/tool errors keep the red notice. Pure tests cover bubble/output/streaming helpers; no GUI stop/error replay was possible.
- Verification: `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline` (46 tests), `cargo build -p pi-desktop --offline`, `cargo fmt --check -p pi-desktop`, `git diff --check` and untracked-file whitespace checks. No GUI launch or screenshot capture was performed. Whole-product visual acceptance remains PARTIAL.

## Native control palette follow-up

The application previously painted its own surfaces with Zai tokens while gpui-kit controls still read library defaults. The root renderer now projects the shared foreground, border, popup/list, primary-button, and neutral focus-border colors through `Theme::update`, including component tokens and Base projection, with an unchanged-palette guard. Settings selectors and form inputs use explicit Zai input surfaces. Desktop 24 tests, build and whitespace checks pass. The update does not prove focus traversal, dropdown placement or rendered color parity; those remain GUI acceptance work.

The settings switch now composes Base Switch, SwitchTrack and SwitchThumb with the reference 32x18 track, 16px thumb, 1px inset and 14px travel, using projected Zai colors. Position and track color use the reference 150ms easing; reduced motion disables the transition. A transparent child extends the pointer area without changing layout; native focus and switch semantics are retained. Fresh screenshot, pointer-target and keyboard verification remain pending: the macOS session was still locked during this source update. This does not advance the full visual-parity acceptance result beyond PARTIAL.

## Conversation workflow viewport follow-up — 2026-10-24

- Added native viewport-scoped horizontal scroll handles (chat and detail dialog separated), reference-derived fixed-point stage folding, up to three edge lamps with overflow count, clickable stage centering, and 40px edge fades. The geometry cases were checked by executing the actual pinned ZCode module in Chromium; see `visual-parity/runs/zcode-3.16.2/component-reference-20261024/geometry.json`.
- Split header status/name/right metadata into separate flex items and removed the redundant right chevron. The header is ordinary content: only staged cards expose whole-card button semantics, tab stop, Enter/Space and blank-area collapse. Card focus now uses a neutral light/dark-aware 2px `ring/40` band instead of a warning-colour border or background swap; child controls stop propagation. Corrected running/failure colors and the digest's 70% surface/border treatment.
- An independent read-only review found post-layout offset clamping, non-overflow wheel trapping, and arbitrary cache eviction risks. The implementation now reconciles width/offset/range after layout, allows surrounding vertical scroll when no horizontal range exists, and retains handles for extant workflows instead of evicting mounted cards by capacity. Lamp/segment widths now follow the 10px/6px reference spacing.
- These changes are implementation progress, not visual acceptance. The current native GUI is still locked. Wheel propagation, button hit targets, dialog focus, resize, masks, scrollbar drag and final-build pixel pairs remain unverified. Automatic running-stage camera, draggable timeline scrollbar, complex rails and child transcript navigation remain open.

## Workflow scrollbar and camera follow-up

- The workflow viewport now uses the installed GPUI base horizontal scrollbar with ZCode 2px idle / 4px hover-active geometry, a 24px minimum thumb, viewport track paging, and native mouse/touch drag handling. Hovering the timeline reveals the scrollbar.
- Running-stage centering waits for post-layout bounds and only re-anchors when the running stage changes. Manual wheel, ledge and scrollbar input cancels pending centering. Drag adapter state resets when overflow disappears or a new pointer sequence starts after a previous mount.
- Five focused viewport tests cover geometry, paging versus drag, camera measurement/manual input and shared drag cleanup. These are not native event-dispatch tests.
- Headless GPUI event testing was attempted, but published test-support dependency resolution failed: the kit route conflicts on cocoa 0.26.0/0.26.1, and the engine-only route requests proptest's unavailable attr-macro feature. The added test dependencies and unexecutable test harness were removed; normal offline builds remain supported.
- Native pointer/touch replay, screenshot pairs and exact motion remain pending. Current camera/page movements are immediate; scrollbar visibility/animation timing follows the installed base theme rather than the reference's exact 800ms hold. Whole-product status remains PARTIAL.

## Native launch verification — 2026-10-24

- Launching the newly built conversation fixture exposed a real `hover style already set` panic. GPUI component Button registers its own hover styling during render; application-level `.hover` on that same button registered it twice. Workflow pills and automation cards now supply their normal/hover/active palette through `ButtonCustomVariant`, keeping the component's interaction handling.
- The rebuilt native process opened window 23785 with the isolated completed fixture. This verifies the conversation startup path past the previous panic, not the automation page or pointer behavior. Capture then failed because the screen locked again; no fresh PNG or matched pixel comparison was produced. The 18:09 diagnostic image belongs to the older process.
- Full visual and interaction acceptance remains PARTIAL, including both themes, running/error/stopped cards, narrow layouts, dialogs and automation hover/focus.

## Conversation composer source-alignment follow-up — 2026-10-24

- Reconciled the native composer with `ConversationComposer.tsx`, `ChatPromptEditor.tsx`, `LexicalChatInput.tsx`, `V4ComposerToolbar.tsx`, `ModelConfigSelect.tsx`, `useComposerToolbarFit.ts`, `button.tsx` and the pinned `styles.css`. Implementation remains in `crates/pi-desktop/src/workbench.rs`; no runtime, agent-core, authentication or other page workflow was changed.
- Draft width remains 672px (`max-w-2xl`). Existing conversations now use the reference content column instead of the previous fixed 760px cap: 100% below 864px, `width - 96px` capped at 896px from 864px, then `width - 384px` capped at 1152px from 1280px. The added unit test covers 480/864/1180/1440px.
- Shell/dock: 16px side padding, existing-conversation 16px bottom padding, 12px inner padding and section gap, 16px radius, alpha border with hover/focus activation. The draft card keeps the 6px workspace bar and gains a subtle shadow. The separate footer hint was removed; the editor is 14px/20px with 40px minimum and 160px auto-grow ceiling.
- Toolbar: 28px rounded-8 plus and send/stop targets; model trigger now includes the 16px package icon, truncated label and chevron with reference insets; trailing controls use a 6px gap. Stop uses the secondary variant. Busy mode keeps both the explicit queue-plus control and stop visible beside the existing queue preview.
- Narrow compression: usage text becomes `12.3k tok` / `1.2m tok` below 680px, and the model trigger cap drops from 240px to 160px and then 116px below 540px. Tests cover compact token labels. This is a source-derived responsive rule, not evidence of rendered overflow behavior.
- Enter-to-send/queue, Shift+Enter newline and native IME composition ownership are unchanged. No attachment, permission or backend-less toolbar action was added. The model popup opens above the trigger, centered on it, matching the composer’s `contentSide="top"` / `contentAlign="center"`.
- Verification this round: `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline` (42 tests), `cargo check`, `cargo build`, `cargo fmt --check -p pi-desktop`, plus whitespace checks. GUI launch, screenshot capture, focus/IME replay and 480px rendered acceptance were **not** performed; the macOS session remained unavailable for GUI verification. Whole-product status remains PARTIAL, not pixel-passed.


## Git review pane and command drawer alignment — 2026-10-24

- Review remains the existing read-only `git diff`/`ls-files` implementation. The wide contract stays a 380px side pane at >=1180px; below 1180px Review takes the single content column and the chat/composer are not rendered beside it. Closing via the pane action or `Dismiss` restores the conversation.
- Added fixed header actions, Unstaged/Staged/Untracked tabs, a 32px search field, a bounded file list and a bordered diff surface with a monospace selected-path header. File-list and diff scrolling now have independent native handles, and their wheel events stop propagation only while that nested section can consume the direction. Loading, error, empty-category, no-match and ready states are mapped by a pure helper instead of overlapping conditionals.
- The command drawer keeps the existing real shell-backed run path but now derives its height from a pure helper: 30% of the viewport capped at 210px. The header/close and run row stay fixed while output scrolls independently. Bash results render as selectable 14px/20px monospace blocks, with per-result error color/border and explicit empty/running states. It remains a command-output drawer, not a PTY or terminal emulator; staging, commit/revert and remote PR actions are still not added.
- Tests added for the 1180px review breakpoint, 30%/210px drawer height, review state precedence, and empty/running/ready drawer mapping. `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline` passed 50 tests, followed by `cargo build -p pi-desktop --offline`, `cargo fmt --check -p pi-desktop`, `git diff --check` and an untracked-file trailing-whitespace scan. No GUI launch or screenshot capture was performed. Rendered hierarchy, narrow composition, wheel propagation and visual parity remain unverified; whole-product status remains PARTIAL.

## Global shell sidebar/header source alignment — 2026-10-24

- Rechecked the pinned ZCode `project-sidebar/index.tsx`, `ProjectSidebarNavigation.tsx`, `ProjectSidebarListHeader.tsx`, `WorkspaceSidebarFooter.tsx`, `DesktopWindowFrame.tsx`, `button.tsx` and `styles.css`. The existing 268px sidebar, sub-820px hide breakpoint, 40px header, 28px controls and 16px vector icons were already correct; they are now named constants and covered by pure boundary tests rather than changed by guesswork.
- Sidebar hierarchy now follows the shell: 48px top clearance, px-2/py-3 primary navigation with 4px gaps, New task then Skills then Automations, 32px rows at 6px radius, a bordered list region with 12px top padding and the sole vertical scroll, 31px history rows, and a fixed Settings footer behind a divider. The list header, workspace row, search and History label are constrained and truncate within the sidebar. Existing navigation behavior is preserved.
- The footer intentionally keeps only the existing Settings entry. No remote-control, upgrade, account, avatar or additional navigation capability was introduced.
- Header controls remain 28px with 16px glyphs. Compact navigation keeps all five narrow-window entries plus Review/Command visible at 480px; the title truncates in the remaining width. The redundant Running label is only rendered when the measured minimum control cluster has room.
- Hover/selected semantics use separate Zai surfaces: 5% hover and the stronger selected token (5% light / 10% dark). Ghost-button hover is projected through the component secondary tokens instead of registering application-level duplicate hover styling. Selected navigation now relies on the component's selected state, with `toggled` exposed for accessibility.
- Pure tests cover the 820px sidebar breakpoint and chat-width subtraction, compact header breakpoint, 172px five-action cluster and its 480px minimum fit. No GUI launch, screenshot capture, focus traversal or pixel comparison was performed in this round; rendered sidebar/footer hierarchy and narrow-header overflow remain pending GUI acceptance. Whole-product status remains PARTIAL.
