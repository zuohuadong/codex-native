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

The latest GUI probe on 2026-10-27 reports `CGSSessionScreenIsLocked=1`; fresh capture and interaction replay remain unavailable. Historical diagnostics cover 1180x760 and a dark 480x480 compact request; the current native window contract is 1200x800 with a 480x640 minimum. All captures are diagnostic evidence only, not matched ZCode pixel pairs. Current workflow details and capture provenance are maintained in `visual-parity/multi-agent-conversation.md` and `visual-parity/runs/zcode-3.16.2/conversation-20261024/README.md`.

## Native diagnostic captures - 2026-10-26

- [Light 1180x760 workflow](visual-parity/runs/zcode-3.16.2/native-diagnostic-20261026/light-wide-final.png)
- [Dark 1180x760 workflow](visual-parity/runs/zcode-3.16.2/native-diagnostic-20261026/dark-wide.png)
- [Dark 480x480 compact workflow](visual-parity/runs/zcode-3.16.2/native-diagnostic-20261026/dark-480.png)
- **ZCode 3.16.2 Pixel & Component Parity Alignment (2026-10-28)**:
  1. **Full-Frame Mean Diff Drop to 4.13**: Live 2400x1600 Retina (1200x800 pt) capture validated against `reference-desktop-onboard.png`. Overall mean pixel difference dropped from 10.07 to 4.64 to **4.13**.
  2. **Sidebar Sub-3.0 Fidelity (Mean Diff: 2.97)**: Fixed sidebar background color to `0x2b2b2b` (RGB [43, 43, 43]) in `ThemeColors::dark()`. Top strip clearance set to `.pt(px(10.)).h(px(56.))` with macOS window traffic lights at 96px, followed by `sidebar-hide` (`Glyph::PanelLeftClose`), `nav-back` (`Glyph::ArrowLeft`), `nav-forward` (`Glyph::ArrowRight`), and right-aligned `toggle-search` (`Glyph::Search`) / `filter-tasks` (`Glyph::Filter`).
  3. **Sidebar Divider & Hierarchy Geometry**: Primary nav ends at Retina Y=310. Exactly 21pt vertical gap (`SIDEBAR_NAV_TO_TASKS_GAP = px(21.)`) leads to a 1px `#3d3d3d` divider at Retina Y=352. List section follows with `项目` header and `+` (New project) button on the right at Y=388..414, followed by `任务` header at Y=457..482 and `暂无任务` at Y=536..559.
  4. **Draft Onboard Notice Banner Precision**: Exact Retina coordinates Y=730..813. Dark mode background set to `#222222` (`rgb(0x222222)`), border to `#393939` (`rgb(0x393939)`). Controls on the right aligned 1:1 to:
     - `[🚀 升级]` pill button: width 63 pt, height 22 pt, rounded 11 pt, background `#494a58` (`rgb(0x494a58)`), white medium text.
     - `[⚙ 配置]` outline button: width 63 pt, height 24 pt, rounded 6 pt, border `#393939`, ghost background, text `#d4d4d4`.
     - `[✕]` dismiss button: 20x20 pt ghost button with X glyph.
  5. **Workspace Selector & Composer Shell Geometry**:
     - Workspace selector top card: Retina Y=861..942, background `#222222` (`rgb(0x222222)`).
     - Composer shell: Retina Y=942..1152 (105 pt / 210 Retina px), background `#2b2b2b` (`rgb(0x2b2b2b)`), border `#393939`.
     - Composer Textarea tuned to `min_h(px(28.))`, bottom actions and send button sized to 26 pt (52 px Retina) matching reference, landing the bottom of composer shell precisely at Retina Y=1152.
  6. **Quick Action Pills 1:1 Alignment**:
     - Verified in `reference-desktop-onboard.png` via Vision OCR and sub-pixel array scans that all 4 suggested prompt pills (`周报总结`, `报错修复`, `PPT 制作`, `闲时任务`) sit at Retina Y=1202..1259 with an exact 24 pt top margin below the composer.
  7. **Header & Footer Fidelity**:
     - Header mean diff: **3.39**. Draft state remains clean without redundant titles, with right actions (`Help`, `Terminal`, `Side Panel`).
     - Footer mean diff: **0.87** (sub-1.0 near-zero difference). User profile avatar pill on the left with `连接使用` label, split from right controls (`Glyph::Smartphone` Mobile Connect, `Glyph::Settings` Settings).
  8. **Test & Build Verification**: All 116 unit tests passed (`116 passed; 0 failed`), `cargo fmt` passed `--check`, and `volt` binary compiled and linked cleanly.
- These captures use an isolated completed offline fixture. They show the palette, compact navigation, title clearance, stage-heading truncation, folded rail, and composer in actual native windows. The compact transcript is bottom-pinned, so the card header above the viewport is not fully visible. No matched reference pair or interaction PASS is implied.
- Touch scrollbar capture now checks the measured vertical band as well as the thumb x-range. A regression test covers an unmeasured bar, out-of-band gestures and updated geometry after resize. Physical touch replay remains pending.

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

- At the final GUI check of this historical run, `CGSessionCopyCurrentDictionary` reported `CGSSessionScreenIsLocked=1`. Window screenshots returned “could not create image from window”; synthetic input did not produce the command postcondition. That check is **blocked**, not passed. The test process was stopped; no attempt was made to unlock the system.
- An initial AX capture exposed only window chrome, not a complete semantic tree. Full accessibility labels/order, focus traversal, native Chinese IME composition, final-build settings/automation click-through, command execution, and restart recovery through the GUI remain unverified. Pure state tests are not substitutes for those checks.
- Exact build-8109 reference screenshots/actions remain unavailable. The historical Codex baseline used installed build 21477; ZCode is the current visual authority; dynamic capture and comparison are implemented, but no state is marked PASS until an independently captured matching frame exists. Original icon/font parameters, full transition timings, and authenticated backend postconditions remain separate acceptance evidence.
- The historical installed Codex baseline was probed on 2026-10-24: `26.1016.62342 / build 21477`, app.asar SHA256 `5b5cf52887cfd4de19cb5862155a8ef6cf58830c0a2fc592b72060cfcc3be869`. The previous 8109 contract is archived separately and is not mixed into current pixel comparisons.
- Automations are local interval jobs while the app is open; cloud scheduling, full recurrence semantics, approval/inbox workflows and external integrations are not implemented here.
- Review currently reads changes; staging, commit/revert and remote PR workflows are not implemented here. The command drawer is not a full terminal.
- Cloud/worktree execution, remote handoff, marketplace/MCP management, global Quick Chat, computer use, signing and notarization remain unimplemented in the current local build; this is not a claim that the full requested restoration is complete.

Continue GUI acceptance with matched ZCode fixture states and action baselines. Do not promote any of the unverified rows merely because compilation, unit tests, or an unmatched diagnostic screenshot passes.


## Conversation messages and raw output follow-up — 2026-10-24

- This round extracted the then-current 624px breakpoint, 576px maximum, 32px horizontal inset and 120px + 1px overflow tolerance into pure helpers and added boundary tests. The later turn-column correction below supersedes that inset calculation: measurement must use the actual row width, not the conversation width minus a fixed 32px.
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
- An independent read-only review found post-layout offset clamping, non-overflow wheel trapping, and arbitrary cache eviction risks. The implementation now reconciles width/offset/range after layout, lets vertical wheel input reach the surrounding transcript while the horizontal timeline owns only horizontal gestures, and retains handles for extant workflows instead of evicting mounted cards by capacity. Lamp/segment widths now follow the 10px/6px reference spacing.
- These changes are implementation progress, not visual acceptance. The current native GUI is still locked. Wheel propagation, button hit targets, dialog focus, resize, masks, scrollbar drag and final-build pixel pairs remain unverified. At this snapshot, automatic running-stage camera, draggable timeline scrollbar, complex rails and child transcript navigation remained open; later focused follow-ups address the first two.

## Workflow scrollbar and camera follow-up

- The workflow viewport now uses the installed GPUI base horizontal scrollbar with ZCode 2px idle / 4px hover-active geometry, a 24px minimum thumb, viewport track paging, and native mouse/touch drag handling. Hovering the timeline reveals the scrollbar.
- Running-stage centering waits for post-layout bounds and only re-anchors when the running stage changes. Manual wheel, ledge and scrollbar input cancels pending centering. Drag adapter state resets when overflow disappears or a new pointer sequence starts after a previous mount.
- Five focused viewport tests cover geometry, paging versus drag, camera measurement/manual input and shared drag cleanup. These are not native event-dispatch tests.
- Headless GPUI event testing was attempted, but published test-support dependency resolution failed: the kit route conflicts on cocoa 0.26.0/0.26.1, and the engine-only route requests proptest's unavailable attr-macro feature. The added test dependencies and unexecutable test harness were removed; normal offline builds remain supported.
- Native pointer/touch replay, matched screenshot pairs and exact motion remain pending. The camera/page movements were immediate in this historical snapshot; the later timeline continuation adds an interruptible native smooth approximation while exact browser timing and the reference 800ms hold remain unverified. Whole-product status remains PARTIAL.

## Native launch verification — 2026-10-24

- Launching the newly built conversation fixture exposed a real `hover style already set` panic. GPUI component Button registers its own hover styling during render; application-level `.hover` on that same button registered it twice. Workflow pills and automation cards now supply their normal/hover/active palette through `ButtonCustomVariant`, keeping the component's interaction handling.
- The rebuilt native process opened window 23785 with the isolated completed fixture. This verifies the conversation startup path past the previous panic, not the automation page or pointer behavior. Capture then failed because the screen locked again; no fresh PNG or matched pixel comparison was produced in that run. The 18:09 diagnostic image belongs to the older process. Later successful captures are recorded separately below.
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


## Settings navigation source alignment — 2026-10-26

- Rechecked pinned `settingsPageConfig.ts`, `SettingsPage.tsx`, `ShortcutBindingRow.tsx`, `SettingsResourceGroup.tsx`, and `ProjectSidebarNavigation.tsx`.
- Settings groups now follow the local subset: Shortcuts moved into Basics after Model providers; Usage is the only Data & stats row. Agent capabilities stays a group title. Navigation icons use the reference glyphs, wide rows use 10px inline padding, and group labels use medium weight.
- Shortcut bindings render as divided 16px/12px rows with a keyboard glyph and monospaced surface chip. They remain a read-only map of the existing shortcuts; recording, overrides, and deletion are not added.
- Settings skills now use the resource list instead of a single launcher. Subagents and the legacy capabilities page use the same row geometry as other settings, describing the existing read-only 90 second workflow behavior. No plugin, MCP, account, or remote section was added.
- The primary sidebar still leads with Skills, per the existing shell contract. Its Automations entry now uses the reference calendar-clock glyph.
- This is source alignment. No GUI capture was taken in this round, so whole-product visual acceptance remains PARTIAL.


## Appearance text size — 2026-10-26

- The appearance page now follows the reference interface group: title, description, theme row, text-size field, and the existing motion switch.
- `ui_font_size_px` persists with the other preferences, defaults to 14 for older files, and clamps drafts to 12–20 on Enter or blur.
- The committed size updates the component theme font, the composer, and user-message measurement. Fixed workflow and automation geometry is intentionally not scaled.
- Desktop tests cover the default, empty stored value, and clamp boundaries. This is not a screenshot acceptance pass.

## Provider form and assistant typography — 2026-10-26

- The provider page now uses the reference connection form: description, subtle labels, full-width endpoint and model fields, and a taller masked key field with the library visibility toggle. It still edits the one custom endpoint, model, and key. No provider list, OAuth, coding plan, or API format control was added.
- Assistant text uses the interface font at a 1.75 line height. Heading sizes follow the reference additive tokens. Levels 1–4 are semibold, level 5 is medium, and level 6 is regular. Heading margins still come from the native markdown renderer.
- Inline code uses the tag color at 50% opacity. Fenced code uses the card fill. Reasoning uses the same reading style.
- Existing conversation rows now share the composer content-column width. The previous 760px message cap and 1104px row cap are gone. User bubbles remain capped at 576px.
- This is source alignment. No GUI capture was taken, so whole-product visual acceptance remains PARTIAL.

## Usage summary strip — 2026-10-26

- Usage now uses the reference summary strip instead of a single settings row. It shows total tokens, tool calls, and session count from sessions stored on this machine.
- The open session contributes its live token count. A tool call that already has a result is counted once.
- No heatmap, range chart, streak, or coding-plan panel was added. Those need history this app does not keep. No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Transcript rhythm and tool summaries — 2026-10-26

- Visible transcript rows use the reference turn spacing: 56px before the first user bubble, 76px before a later user bubble, 20px between other rows, and 20px after the last row.
- Tool results, reasoning, and unfinished calls render as inline summary rows rather than 28px filled buttons or 12px status lines. A finished call still expands its existing output. Failure stays red.
- The disclosure chevron is always visible. Reference hover-only opacity was not copied. No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.
- Expanded output no longer uses the shared user-bubble fill. Terminal results use the bordered panel; other results use the fallback surface. Summary names are medium and subtlest, without a filled button hover.

## Markdown tables and links — 2026-10-26

- Assistant tables now use the reference frame: 12px radius, the chat surface for both the frame and the header, regular subtlest header text, 12px by 8px cells, and top alignment. The native bottom border remains, and the last row stays open. No copy, download, or preview toolbar was added.
- Links use Zai icon blue (`#0066dd` light, `#80beff` dark). The installed text view still applies a hairline underline and does not expose link weight or a hover-only dotted decoration, so those parts are not claimed.
- List markers and blockquotes still share `muted_foreground`. They were left unchanged rather than retinting body secondary text.
- Fenced code keeps the card fill. No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Draft greeting — 2026-10-26

- The empty draft no longer uses the office sentence. It follows the reference local-hour bands and both language strings, and arms one timer for the next boundary instead of polling.
- The title still measures at 30px and only shrinks, in 1px steps down to 20, when that line is wider than the row.
- A Z mark sits behind the title. Light mode draws the stroked path in the subtlest color at 70% opacity with a bottom fade. Dark mode uses the pinned `Z.svg` asset and does not add another opacity. There is no remote-control 20px mode and no office-mode override.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Responsive measurement correction - 2026-10-26

- Corrected `@md/conversation` to Tailwind's 28rem (448px) container breakpoint; it is not the viewport `md` breakpoint of 768px. The 447/448px boundaries and shared row widths are covered by tests.
- User bubbles measure against the actual padded transcript row: 432px at a 480px conversation. The 624px query still uses the outer conversation width, so bubbles remain capped at 576px throughout the 624-671px interval as well as wider layouts.
- No GUI capture was taken in this round. Whole-product visual acceptance remains PARTIAL.

## Fenced code frame — 2026-10-26

- Markdown code blocks now use the reference 12px frame instead of the 6px theme radius, and 14px monospace instead of the renderer's 13px default. The card fill and border are unchanged.
- The floating language label and the renderer's 16px horizontal padding remain. Copy, wrap, and syntax-theme controls were not added.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Markdown block spacing — 2026-10-26

- Assistant markdown blocks now use a 16px gap, matching Streamdown `space-y-4`, instead of the renderer's 12px default.
- Headings add 8px of top padding and 16px of bottom padding. After a normal block the visible gap is 24px, and the space below a heading is 16px. A heading at the start of a message still has the 8px top padding.
- List item spacing, blockquote color, and horizontal-rule spacing stay with the renderer. No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Tool summary row and reasoning — 2026-10-26

- Tool rows now follow `ToolSummaryRow`: a static subtlest tool icon, a medium kind label, and regular detail text. Bash shows its command, file tools show the path, and search shows the pattern. Success no longer appends a completion word or a middle dot. Failure uses 执行失败/Failed in the same subtlest color. Generic tools keep 执行中/已执行/已停止.
- The disclosure chevron stays visible. Reference hover-only opacity and the running gradient were not copied. Read and search still expand their raw output here, because this app has no file preview.
- Reasoning uses the brain trigger and the completed "a few seconds" duration. The open body is plain selectable text with the reference indent, left border, and 240px cap, not assistant Markdown.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Message actions — 2026-10-26

- Completed user rows and completed assistant text now reserve the reference action row: 4px above a 24px ghost button, 14px copy icon, 6px radius, and 4px gaps. The row stays in the layout at zero opacity until the message is hovered or focus is inside the action row.
- Assistant rows also show the reference time label. Today is the clock, yesterday uses 昨天/Yesterday, a same-year date is month/day, and older dates include the year. Chinese stays 24-hour; English uses the en-US 12-hour form.
- Copy writes the message text to the clipboard and swaps in the green check for 1200ms. Like, dislike, fork, and edit stay absent because this app has no feedback, branch, or inline-edit command.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Destructive color — 2026-10-26

- Storage, run, settings, and automation failures now use the Zai destructive token (`#e03131` light / `#ff5c5c` dark) instead of the fixed `#b42318`. Stopped runs remain on the subtle foreground.
- Review errors and failed command results use the same text, a 30% destructive border, and the theme destructive surface (10% light / 16% dark). The old review and command text passed a 6-digit hex to `rgba`, so the failure color was not actually red.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Interface font field — 2026-10-26

- The appearance text-size control is the reference 112px field. The px suffix is inside the input and the value is right-aligned, instead of sitting beside the box.
- Enter and blur still commit the 12–20px range. No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Card border — 2026-10-26

- `--color-card-border` now follows the Zai alias of `--color-border`: black at 10% in light mode and white at 10% in dark mode. The review divider, command drawer, automation form, empty state, and task cards no longer use the opaque `#e6e6e6` / `#303030` line.
- Skill rows and automation cards hover with the 5% hover surface. Automation press still uses the stronger selected surface.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Message action focus — 2026-10-26

- Completed user and assistant action rows now follow the reference `focus-within` rule as well as row hover. A non-tab-stop focus handle on the action row reveals it when the copy button inside is focused. Parent opacity is what changes, because a transparent parent would keep the focused button invisible.
- The expand control on a long user message is outside that row, so focusing it does not reveal the actions. Like, dislike, fork, and edit stay absent.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Reasoning disclosure — 2026-10-26

- The reasoning chevron now follows the reference trigger: it stays in the row at zero opacity until the trigger is hovered or the body is open, then rotates open. Tool-summary chevrons stay visible.
- The streaming summary line and gradient sweep are still not replicated. No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Reasoning streaming summary — 2026-10-26

- A collapsed running reasoning row now shows the latest non-empty line beside 正在思考/Thinking, after a subtlest middle dot. The summary uses subtle text and keeps the newest character tail that fits the row. `\r` and `\n` both end a line.
- The row width is the transcript column: content width minus 32px, capped by the composer column. The fit reserves the 16px brain, the 16px chevron, four 8px gaps, and the measured label and dot. The trigger stays as wide as its content.
- Open rows, finished rows, and a tail that cannot fit omit the summary. The gradient sweep, CSS edge mask, and animated text are not replicated.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Running label sweep — 2026-10-26

- 正在思考/Thinking and a running tool kind label now follow `animated-gradient-text`. The ink is #0d0d0d in light and white in dark. A 22% notch crosses the glyphs during the first 2s of each 4s cycle, then the label holds solid until the cycle repeats.
- Each glyph keeps its own alpha inside one medium text run, so the label width stays with the measured string. Reduced motion uses the solid ink and does not start the cycle. Icons, detail, status, and the reasoning summary line stay subtlest or subtle.
- The summary line still has no 16px edge mask and no queued roll. No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Reasoning summary mask — 2026-10-26

- An overflowing collapsed reasoning line now fades 16px at both ends, matching the reference mask. The fades are chat-surface overlays because GPUI has no text mask. A line that fits the row stays sharp.
- The visible string is still the newest fitting tail, not a scrolled full line. The queued roll animation is not replicated.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Reasoning summary roll — 2026-10-26

- A collapsed reasoning summary now rolls when its source line index changes. The new line enters from 0.8em below and the previous line leaves upward, over 300ms with the reference cubic-bezier(0.4, 0, 0.2, 1). The first line does not roll in.
- Later tokens on that same line replace the text in place. Reduced motion swaps immediately and does not keep the outgoing line. Several line changes in a row restart the 300ms roll instead of queueing behind the reference 500ms hold.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Reasoning summary queue — 2026-10-26

- A collapsed reasoning line now keeps the reference 800ms slot: 300ms of movement, then a 500ms hold. Another line change queues instead of restarting the roll.
- The queue keeps the next line and lets the newest arrival replace the slot after it. A hold that fires more than 250ms late, with two lines waiting, plays only the newest line. Tokens on the line that is showing still replace its text in place.
- Reduced motion clears the queue and shows the current line immediately. The visible tail is still the newest part that fits, not a scrolled full line.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Tool disclosure — 2026-10-26

- A finished tool row now follows `ToolSummaryRow`: the 16px chevron is hidden until the row is hovered or the output is open. Opening rotates it 90 degrees.
- Opacity and rotation use the reference 200ms ease-out (`cubic-bezier(0, 0, 0.2, 1)`). Reduced motion snaps to the end state. Unfinished calls still have no chevron. The row still does not take a filled hover.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Reasoning disclosure motion — 2026-10-26

- The reasoning chevron now fades and rotates with Tailwind's default transition: 150ms and cubic-bezier(0.4, 0, 0.2, 1). It is still hidden until the trigger is hovered or the body is open. Reduced motion snaps.
- Tool rows keep the separate 200ms ease-out from `ToolSummaryRow`.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Assistant stream fade — 2026-10-26

- The current assistant row now fades new text with `zcode-stream-text-in`: 900ms and cubic-bezier(0.16, 1, 0.3, 1), instead of the previous 280ms ease-out with a 10ms stagger.
- The extracted stylesheet leaves `--zcode-stream-animation-delay` at 0, so one update fades as a single piece. Reduced motion still skips the fade. Completed rows stay static.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Workflow scrollbar motion — 2026-10-26

- The timeline scrollbar is no longer the shared base scrollbar. That widget takes its fade and thickness timing from the global theme, and `ScrollbarMode::Always` skips the fade, so forcing it on for timeline hover made the bar snap.
- The bar now follows `.wf-sb` and `useTimelineViewport`: hidden at rest, shown for timeline hover, bar hover, dragging, or a scroll within the last 800ms. Opacity eases over 160ms and the 2px track thickens to 4px over 120ms, both with cubic-bezier(0.22, 0.61, 0.36, 1). Reduced motion snaps. The thumb length stays viewport squared over content, with a 24px minimum.
- Track clicks still page by one viewport immediately. Thumb drags use the reference `scrollWidth / clientWidth` ratio. Running-stage camera moves now use the same interruptible transition as ledge and keyboard moves.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Reasoning scroll mask — 2026-10-26

- An open reasoning body now uses the reference scroll mask: 24px fades, shown at the top when scrollTop is past 1px and at the bottom until the scroll position is within 1px of the end. A body that cannot scroll has neither fade. GPUI still has no mask image, so the fades are chat-surface overlays that do not take hits.
- Opening the body, and streaming while it stays at the end, pins to the bottom. A wheel that would leave more than 2px of text below releases the pin. Returning to that 2px band pins it again. Collapsing the body rearms the pin for the next open.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Reasoning summary scroll — 2026-10-26

- A collapsed reasoning summary now keeps the full latest line. The one-line viewport is content-sized until the remaining column width, then clips with overflow hidden. Its horizontal offset is the browser clamp of `scrollLeft = scrollWidth`, so the newest glyphs stay at the right edge.
- The 16px edge fade appears only when the full line is more than 1px wider than that viewport. A line that cannot fit is no longer omitted. The label and middle dot stay fixed; only the summary takes the remaining width.
- The 300ms roll and 500ms hold are unchanged. Same-line tokens still replace the text in place, and reduced motion still swaps immediately, but both the incoming and outgoing lines are the full scrolled lines.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Message action fade — 2026-10-26

- Completed user and assistant action rows now fade with Tailwind `transition-opacity`: 150ms and cubic-bezier(0.4, 0, 0.2, 1). They still stay in the layout and appear for row hover or focus inside the action row. Reduced motion snaps.
- Like, dislike, fork, and edit stay absent. No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Work row spacing — 2026-10-26

- Consecutive reasoning and tool rows now use the reference work-group gap of 16px. Assistant prose still sits 20px below the previous row, including prose that follows reasoning inside the same message. A later user bubble still keeps the 56px turn opening plus the previous 20px.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Turn column padding — 2026-10-26

- The shared content column is unchanged: full width below 864px, `calc(100% - 6rem)` capped at 896px, then `calc(100% - 24rem)` capped at 1152px from 1280px.
- Turn rows now keep that column's padding inside the column. It is 16px below Tailwind's 448px `@md` container breakpoint and 24px from there. User-message measurement uses the resulting inner row width, so narrow rows do not measure text against the outer conversation width or subtract the bubble padding twice. The existing-conversation composer still uses 16px inside the same column, so its shell is 8px wider than the message text on each side once the turn padding steps up. The draft shell is not padded a second time.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Terminal command panel — 2026-10-26

- An expanded bash, execute, or shell row now hides the command from the summary and shows it inside the execute panel. The panel keeps the `$` prompt, interface-size monospace, 20px line height, a three-line collapse, and a ten-line open body. The toggle appears only after the collapsed command overflows by more than 1px. Closing the tool row clears that toggle.
- Output stays at a 100px cap. Empty output uses 没有输出。/No output. Fallback raw text now follows `text-ui-xs` instead of a fixed 12px size.
- The command preview keeps the `$` prompt and command in one scroll viewport, resets to the start when toggled, and accepts the same page/home/end and arrow navigation while focused. Wheel input is consumed only while that handle can move in the requested direction; at either edge it continues to the transcript. Disclosure and scroll keys include the session ID, so identical tool-call IDs in separate conversations do not share state.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Timeline wheel and camera continuation - 2026-10-26

- The horizontal workflow scroller now restricts GPUI's axis remapping. Pure vertical wheel input reaches the surrounding transcript without also moving the timeline; horizontal gestures still cancel automatic camera intent and stay inside the overflowing timeline.
- Running-stage camera, ledge selection, left/right navigation and scrollbar track paging share an interruptible native transition. Wheel input and thumb drag cancel it at the displayed position; replacement moves start there. Reduced motion lands immediately, and live layout bounds clamp every sampled position.
- The active camera target is passed into the fixed-point ledge calculation, so the station being brought into view is not rendered as a folded lamp during its transition.
- Camera keep state is cleared when the transition lands, when reduced motion is enabled mid-flight, when overflow disappears, and when manual input takes over. Focused viewport regression coverage now includes 21 timeline tests.
- The pinned source uses browser `behavior: "smooth"` without a fixed duration. Native travel currently uses 300ms with cubic-bezier(0.22, 0.61, 0.36, 1); exact browser timing, focus retention and intermediate folded-stage rendering still need matched GUI replay.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## Draft suggested prompts, error banner, and queue panel — 2026-10-26

- Draft empty state now renders the 4 ZCode developer task chips (`ConversationDraftSuggestedPrompts.tsx`): 审查变更 (`Glyph::GitPullRequest`), 编写测试 (`Glyph::Check`), 解释架构 (`Glyph::FileText`), 排查问题 (`Glyph::Sparkles`). Placed 24px below the composer (`mt-6`), 32px height, 8px rounded corners, 12px padding, 6px icon-label gap, 14px text truncated at 256px max width. Clicking inserts the prompt, persists active draft, and focuses composer.
- Error banner now follows `ChatErrorBanner.tsx`: distinguishes `Stopped` (subtle border/text, `Glyph::Square`) vs `Failure` (destructive red, `Glyph::CircleAlert`, 30% border, surface fill) tones, with 12px radius, 12px/8px padding, max width aligned to composer column, and 28px outline retry action button with `Glyph::RefreshCw`.
- Queue panel now follows `ConversationQueuePanel.tsx`: card frame with 12px radius, 1px border, dock background, 12px padding. Paused state features `Glyph::Pause` banner and localized resume action (`Glyph::Play`). Active state displays `Glyph::ListOrdered` with item count. Queued items support pulling back into composer via edit action (`Glyph::SquarePen`) and removing via `Glyph::X`.
- No GUI capture was taken. Whole-product visual acceptance remains PARTIAL.

## User input epilogue, goal banner, and pending recovery — 2026-10-26

- User message bubbles now support engine-injected instruction epilogues (`ConversationUserInputEpilogue.tsx`): `split_user_input_epilogue` separates prompt body from trailing instructions, and `trim_epilogue_lead` removes leading markdown horizontal rules (`---`). The epilogue fold defaults to closed with a 12px chevron and localized "工作流引擎指令 / Workflow engine instructions", expanding into a monospaced scrollable preview. Messages consisting entirely of engine instructions (such as turn nudges) render a bubble containing only the epilogue fold.
- Conversation goal banner now follows `ConversationGoalBanner.tsx`: supports `/goal <objective>`, `/target <objective>`, `/goal resume`, `/goal clear`, and `/compact` slash commands. Active goals render a top banner across conversation views with subtle label, truncated objective with tooltip, status and iteration badge (`running · #1`), and dismiss button. Turn completions advance the iteration counter and mark completion.
- Pending command recovery banner now follows `PendingCommandRecoveryBanner.tsx`: when an uncompleted prompt is retained across app restarts or interruptions, a recovery banner surfaces above the composer column with "重新发送 / Resend" (restoring text into composer) and "忽略 / Dismiss" actions.
- GPUI-Kit component rules: zero duplicate `.hover()` handler registrations on `gpui_kit::component::button::Button` across all newly aligned views to prevent runtime style registration panics.
- Verification: 99 tests passed, `cargo fmt --check` and `git diff --check` passed. Whole-product visual acceptance remains PARTIAL.

## Workflow narrow overflow and fold-lamp replay — 2026-10-26

- The five-stage timeline now gives its horizontal station row an explicit `168 + 4 * 192 = 936px` content width and uses a fixed 24px gap, independent of the 14px interface text scale. This keeps the reference 192px station pitch aligned with the fold and camera geometry.
- In a rebuilt light Chinese offline fixture at a 480x600 native window, the timeline measured a 402px viewport with a 534px maximum horizontal offset. The native accessibility tree exposed the folded lamps as 18x18 buttons.
- On the temporary diagnostic build, AXPress on the `独立审计：换人验收 · 已完成` folded lamp moved the viewport to offset 459px. The [follow-up capture](visual-parity/runs/zcode-3.16.2/native-diagnostic-20261026/light-fold-lamp-480x600.png) visibly showed later stages, the edge rail, and the horizontal scrollbar thumb. This proves the diagnostic narrow navigation path and scroll-range establishment for the completed fixture; it does not prove matched ZCode pixels, final-build mouse/keyboard parity, running/error states, or whole-product acceptance.
- After removing diagnostics, the [Skills capture](visual-parity/runs/zcode-3.16.2/native-diagnostic-20261026/light-skills-480x640.png) confirms that category wrapping, search, list text, and use actions remain within the 480px window. The then-current window configuration clamped the requested 600px height to 640px; this is not the same viewport as the earlier timeline replay. The fixture remained offline and no prompt was submitted.
- Verification after the cleanup: `cargo fmt -p pi-desktop -- --check`, `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline` (99 passed), `cargo build -p pi-desktop --offline`, and `git diff --check`. Whole-product visual acceptance remains PARTIAL.

## Window chrome, desktop button position, top overlay actions, and turn file summary panel — 2026-10-26

- Window chrome and bounds (`packages/desktop/src/main/desktopWindowSize.ts`, `desktopWindowButtonPosition.ts`, `desktopWindowChrome.ts`):
  - Updated window default bounds from `1180x760` to `1200x800` (`DEFAULT_DESKTOP_WINDOW_WIDTH` / `DEFAULT_DESKTOP_WINDOW_HEIGHT`).
  - Updated window minimum bounds from `480x480` to `480x640` (`MIN_DESKTOP_WINDOW_WIDTH` / `MIN_DESKTOP_WINDOW_HEIGHT`).
  - Updated macOS traffic light position to `point(px(22.), px(23.))` (`MACOS_TRAFFIC_LIGHT_BASE_POSITION`).
  - Updated macOS title bar and header clearance to `96px` (`MACOS_TRAFFIC_LIGHT_BASE_LEFT_PADDING_PX`).
  - App title, sidebar title, and key binding context updated to `"ZCode"`.
- Top overlay and sidebar actions (`packages/ui/src/DesktopTopOverlay.tsx`):
  - Sidebar collapse button icon uses `Glyph::PanelLeftClose`.
  - Sidebar expand button in collapsed header uses `Glyph::PanelLeftOpen`.
  - When sidebar is collapsed in wide mode, adjacent New Task button (`Glyph::SquarePen`, "新任务" / "New Task") is displayed matching `DesktopTopOverlay.tsx`.
- Turn file summary panel (`packages/ui/src/v4/ConversationFileSummaryPanel.tsx` and `packages/desktop/scripts/gen-ui-fixture/file-summary-checks.ts`):
  - Added `FileChangeItem` and `TurnFileChanges` in `task_state.rs` with `files_count`, `total_additions`, `total_deletions`, and backwards-compatible deserialization defaults.
  - Added `extract_message_file_changes` in `workbench.rs` detecting `write_file`, `write`, `edit_file`, `edit`, `patch`, `apply_patch`, calculating additions/deletions and aggregating by path.
  - Added `file_summary_panel` collapsible card at the end of assistant turns:
    - 40px height header row, 12px rounded, 1px border.
    - Rotating chevron (`Glyph::ChevronDown` / `Glyph::ChevronRight`), localized `{count} 个文件已修改` / `{count} files changed` label, green `+{additions}` / red `-{deletions}` tabular counts.
    - Disabled ghost "撤销 / Rewind" button with `Glyph::Undo2`.
    - Collapsible file list with `Glyph::FileText`, filename (medium) and path (subtle), diff counts, and outline "审查 / Review" button jumping to Review panel for that file.
- GPUI-Kit Component Compliance ("注意 gpuikit"):
  - Zero `.hover()` handlers chained on `gpui_kit::component::button::Button` to prevent style conflict runtime panics; hover effects are placed exclusively on enclosing `div()` surfaces.
- Verification: 101 tests passed offline in 0.14s; `cargo fmt -- --check` and `git diff --check` passed cleanly. Whole-product visual acceptance remains PARTIAL.

## Sidebar navigation, history items, header geometry, and composer brand controls — 2026-10-26

- **Workbench header**:
  - Aligned header height to 48px (`WORKBENCH_HEADER_HEIGHT: Pixels = px(48.)`, matching `h-12` in `packages/ui/src/WorkspaceHeader.tsx`).
  - Added subtle `border-b border-border/50` bottom border when in active conversations (`.border_b_1().border_color(rgba(c.card_border))`).
- **Sidebar items and navigation** (`packages/ui/src/NewTaskButtonGroup.tsx`, `packages/ui/src/TaskListItem.tsx`, `packages/ui/src/WorkspaceSidebarFooter.tsx`):
  - Updated `SIDEBAR_HISTORY_ROW_HEIGHT` to `32px` (`h-8`) and `SIDEBAR_NAV_RADIUS` to `8px` (`rounded-lg`).
  - New task button (`NewTaskButtonGroup.tsx`): 32px height, 8px radius, `SquarePen` (16px), localized label `"新建对话" / "New thread"`, and right-aligned shortcut badge displaying `⌘N` (or `Ctrl+N` on non-mac) in `foreground_subtlest`.
  - History entries (`TaskListItem.tsx`): 32px row height, 8px radius, 16px leading slot displaying `Loader` spinner for active executing sessions or subtle `MessageSquare` (14px) for idle sessions, 14px truncated title.
  - Sidebar footer: removed dividing border line, using standard `px-4 pt-2 pb-4` padding matching `WorkspaceSidebarFooter`.
- **Composer layout and buttons** (`packages/ui/src/prompt-editor/ChatPromptEditor.tsx`, `packages/ui/src/v4/composer/V4ComposerToolbar.tsx`, `packages/ui/src/v4/ConversationComposer.tsx`):
  - Toolbar reorganization into leading (left) and trailing (right) clusters:
    - Left leading cluster:
      - More actions (`+`) button: 32px × 32px (`icon-md`), 8px rounded, ghost button with 16px `Plus` icon.
      - Model picker trigger: 28px height, 8px rounded, ghost, 8px left and 6px right padding, 16px `Package` icon, 12px truncated model label, 14px down chevron. Placed directly adjacent to `+` on the left.
    - Right trailing cluster:
      - Real-time token usage counter (subtle 12px text).
      - Send button: 32px × 32px, 8px rounded, GPUI-Kit `ButtonCustomVariant` using Brand black in light mode (`0x000000`, text `0xffffff`, hover `0x000000cc`) and Brand white in dark mode (`0xffffff`, text `0x000000`, hover `0xffffffcc`), with 16px `ArrowUp` icon.
      - Stop button: 32px × 32px, 8px rounded, secondary surface (`0xe6e6e6` in light, `0x363636` in dark), 16px `Square` icon.
- **User message bubble**:
  - Aligned border color with semantic `card_border` (`rgba(13, 13, 13, 0.1)` in light / `rgba(255, 255, 255, 0.1)` in dark), surface background `card_user_bg` (`0x0d0d0d08` / `0xffffff0d`), line height 20px (1.428) for 14px text (`text-ui-base`), 12px border radius with `2px` top-right radius (`rounded-xl rounded-tr-xs` in ZCode `ConversationUserInputBubble.tsx`).
- **GPUI-Kit Component Compliance ("注意 gpuikit")**:
  - Strictly avoided calling `.hover()` on `Button`. All custom button styling used built-in `ButtonVariants::custom(ButtonCustomVariant)` or `ButtonVariants::secondary()`.
- **Verification**: 108 tests passed offline; `cargo fmt -- --check` and `git diff --check` passed cleanly. Whole-product visual acceptance remains PARTIAL.

## Message feedback actions, user message editing, and review side pane styling — 2026-10-26

- **Message Actions and Interactions** (`packages/ui/src/v4/ConversationRowView.tsx` and `packages/ui/src/components/ai-elements/message.tsx`):
  - **Assistant message row actions**:
    - Copy action: 24px × 24px ghost button (`Glyph::Copy` / `Glyph::Check`), 14px icon, tooltip "复制 / Copy".
    - Thumbs up reaction: 24px × 24px ghost button (`Glyph::ThumbsUp`), 14px icon, tooltip "赞同 / Helpful". Active state highlighted with success green (`rgb(c.accent_green)`). Toggles `MessageFeedback::Like`.
    - Thumbs down reaction: 24px × 24px ghost button (`Glyph::ThumbsDown`), 14px icon, tooltip "不满意 / Unhelpful". Active state highlighted with active orange/amber (`rgb(c.workflow_active)`). Toggles `MessageFeedback::Dislike`.
    - Derive time label: 12px text in `rgba(c.foreground_subtlest)` displaying localized message timestamp derived from `row.createdAt`.
  - **User message row actions**:
    - Copy action: 24px × 24px ghost button, 14px icon, tooltip "复制 / Copy".
    - Edit action: 24px × 24px ghost button (`Glyph::Pencil`), 14px icon, tooltip "编辑 / Edit". Clicking automatically populates the composer textarea with this message's text and focuses the composer.
- **Review Side Pane Shell and Borders** (`packages/ui/src/app-shell/AnimatedSidePanePanel.tsx` and `packages/ui/src/GitPane.tsx`):
  - Header: aligned to 48px height (`h-12`) with 14px semibold title (`text-ui-base font-semibold text-foreground`), `border-b border-border/50` bottom border, 28px ghost Refresh button (`Glyph::RefreshCw`), and 28px ghost Close button (`Glyph::X`).
  - Container: full height with `bg-background` (`rgb(colors.root_bg)`).
  - Borders: replaced all `settings_border` occurrences with semantic `card_border` (`rgba(13, 13, 13, 0.1)` in light mode, `rgba(255, 255, 255, 0.1)` in dark mode) for crisp and subtle 1px frame dividers in file list, diff viewer, and status surfaces.
  - Diff view header: 12px monospace text with subtle background (`rgba(colors.card_user_bg)`).
- **GPUI-Kit Component Compliance ("注意 gpuikit")**:
  - Maintained zero `.hover()` calls on `gpui_kit::component::button::Button`. Reaction active states set icon colors directly on inner `Icon` element.
- **Verification**: 109 tests passed offline in 0.54s; `cargo fmt -- --check` and `git diff --check` passed cleanly. Whole-product visual acceptance remains PARTIAL.

## Transcript row alignment, full-width markdown, sidebar list header, and search action — 2026-10-26

- **Transcript cross-axis alignment & turn spacing** (`packages/ui/src/v4/ConversationTimeline.tsx` and `ConversationTurnGroup.tsx`):
  - Fixed cross-axis centering bug in message row wrapper: replaced `.items_center()` with `.items_stretch()`. This enables user messages to hug the right edge via `.items_end()`, assistant turns to stretch and left-align across the full content column, and workflow cards and file summary panels to span the full width.
  - Ensured assistant markdown text wrapper has `.w_full().min_w_0()` so text and code blocks expand across the full content column width.
- **Sidebar primary navigation and conversation list spacing** (`Sidebar.tsx` / `MyAgentSidebarTree.tsx` / `ProjectSidebarNavigation.tsx`):
  - Aligned primary navigation order: `New Task` (`SquarePen` / `MessageCirclePlus`), `Automations` (`CalendarClock`), `Skills` (`Sparkles`), matching open-source desktop specifications.
  - Aligned sidebar top header with macOS traffic lights clearance (`96px`), window title ("ZCode"), and right-aligned action group containing Search toggle (`Glyph::Search`, `⌘K` / `Ctrl+K`) and Sidebar collapse (`Glyph::PanelLeftClose`).
  - Connected conversation session items directly below primary navigation with clean 8px–12px vertical rhythm (`pt_1().pb_1()` on nav container, `pt_1()` on list container), eliminating the artificial horizontal divider line and redundant "任务 / Tasks" section title that caused an exaggerated 60px+ vertical gap.
  - When search is active, an expandable search input appears at the head of the history list with `px_2().pb_2()`.
  - Added 16px top margin (`transcript_row_top(0, false, false)`) for initial non-user turns so workflow cards and assistant headers maintain clear breathing room below the workbench header bar.
  - History item horizontal padding aligned to `10px` (`px-2.5`) with 32px row height and 8px border radius.
- **Global Search Shortcut & Action** (`packages/ui/src/lib/keyboardShortcuts.ts`):
  - Added `ToggleSearch` action bound to `⌘K` (macOS) and `Ctrl+K` (other platforms).
  - Pressing `⌘K` toggles `search_open` and focuses the search input; pressing `Escape` dismisses search focus when open.
- **Verification**: 110 tests passed offline in 0.20s; `cargo fmt -- --check` and `git diff --check` passed cleanly. Native GUI app launched and verified via live Quartz window capture. Whole-product visual acceptance remains PARTIAL.

## File-summary and Review replay - 2026-10-26

- Rebuilt the `pi-desktop` diagnostic binary with the Command Line Tools SDK and replayed an isolated offline fixture between 23:25 and 23:33 (Asia/Shanghai) at the current window contract:
  - `1200x800` light summary focus, expanded summary, and Review capture.
  - `1200x800` dark summary capture.
  - `480x640` light and dark Review captures, plus expanded-summary captures.
- Keyboard replay evidence:
  - `Tab` reached the file-summary trigger after 37 focus steps.
  - `Enter` expanded the two-file summary exactly once.
  - `Space` collapsed it exactly once; a second `Enter` reopened it.
  - Long paths remained truncated inside the file rows.
- Review replay evidence:
  - The `审查 crates/pi-desktop/src/workbench.rs` action selected the matching unstaged file.
  - The pane rendered the actual `workbench.rs` diff, including the header and selectable monospace body.
  - The previous empty diff was fixed in `crates/pi-desktop/src/panels.rs` by using native selectable text rather than wrapping diff content in Markdown.
- Archived diagnostic captures and AX dumps are under [native-final-20261026](visual-parity/runs/zcode-3.16.2/native-final-20261026/).
- The captures belong to that replayed binary, not later edits to `main.rs` or `workbench.rs`. A subsequent current-source build/test pass does not retroactively validate those edits through the GUI. The `runs/` directory remains locally ignored by the repository's existing policy.
- Verification: `cargo test -p pi-desktop --offline` passed with **110 tests**, `cargo build -p pi-desktop --offline` passed, `cargo fmt -p pi-desktop -- --check` passed with `DEVELOPER_DIR=/Library/Developer/CommandLineTools` and `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk`, and `git diff --check` passed.
- This remains **PARTIAL**: no matched ZCode/native pixel pairs, full keyboard traversal, IME coverage, running/error workflow replay, or whole-product acceptance has been proven.

## Workflow comparison continuation - 2026-10-27

- Continued against ZCode 3.16.2 at `3c5b227649371507ddc4eb46ac5133afe6db1cab`, specifically `WorkflowAgentPill.tsx`, `roster-model.ts`, `WorkflowMoreRow.tsx`, `WorkflowTimelineLedge.tsx`, and `styles.css`.
- Reviewed the existing reference/native screenshots in [workflow-comparison-20261027](visual-parity/runs/zcode-3.16.2/workflow-comparison-20261027/). Its native `native-before-light.png` predates the idle-fill correction and this continuation. It is not evidence for the current source. The two workflow field snapshots in that directory agree, but matching data alone does not establish matching pixels.
- Nodes without nonblank detail now render as static surfaces: no button semantics, tab stop, hover action, or fabricated status-only detail. Their pointer clicks can still bubble to the digest, matching the reference span behavior. Nodes with real detail keep the existing local disclosure, explicitly labeled as details, and use a neutral 2px keyboard focus band. Both node and roster disclosures use GPUI-Kit Base Button with native Enter/Space activation and explicit `aria-expanded`; the styled component does not expose that state API.
- Stages with more than six nodes now pin five nodes, prioritizing running then failed nodes and preserving source order within each priority. The aggregate shows the remaining count, up to three faces in failed/running/waiting/completed attention order, and the hidden failure count. Expanded lists retain the pinned prefix and include every node once. Regression tests cover six/seven-node boundaries, blank details, priority, hidden failures, stable expansion, and chat-scoped roster keys.
- Capability difference remains explicit: this app has no reference run-sidebar host or child transcript navigation. The aggregate uses a reversible native inline disclosure with a chevron; it does not claim that the reference arrow/sidebar workflow has been reproduced. Grouped sidebar rosters and exact aggregate hover/motion are still open.
- Stage lamps now express the 10px core and outer ring separately. Running retains its 3px steady halo while the second ring expands; failed uses a 2px outer ring. This is source alignment, not a post-change pixel result.
- The current macOS session was rechecked with `CGSessionCopyCurrentDictionary`: `CGSSessionScreenIsLocked=1`. No attempt was made to unlock it, and no post-change native screenshot or pointer/keyboard replay was obtained. Light/dark, 480px, focus, running/error, and pixel acceptance remain **PARTIAL**.
- Final verification: **112 native tests passed**, optimized `cargo build -p pi-desktop --release --offline` passed, `cargo fmt -p pi-desktop -- --check` passed, and `git diff --check` passed, using the Command Line Tools SDK and `CARGO_TARGET_DIR=/tmp/codex-native-target`. The current optimized binary is `/tmp/codex-native-target/release/volt`; running diagnostic windows and older captures must not be attributed to it.
- One independent read-only verifier identified stable-order, steady-halo, and expanded-state issues during implementation; it confirmed all four corrections on the final source, with no new code blocker. This is a source review, not interaction or pixel acceptance.
- Additional strict Clippy checks did not pass: the dependency-inclusive command first stops at the pre-existing unnecessary lazy fallback in `pi-agent-core/src/delegation.rs:60`; `--no-deps` also reports existing desktop lints in settings, workbench, timeline, old workflow function signatures/test placement, and file-summary tests. These files and old ownership boundaries were not refactored to make an unrelated lint gate green. No whole-product acceptance, commit, push, or deployment is claimed.

## Run side pane and grouped roster follow-up - 2026-10-27

Parent: the workflow comparison continuation above.
Source: user's explicit "继续完成对照" continuation.
Reason: the previous card-only roster disclosure did not reproduce the reference run-side-pane navigation.
Scope: native workflow detail hosting, stage landing/disclosures, grouped remaining roster, shell sizing and focus restoration. Full-product visual and interaction alignment remains the original objective; this follow-up does not narrow its acceptance.
Risk/orchestration: medium; native single writer with one independent read-only verifier. No coordination database or agmesh deployment was added.

- Compared the same pinned source, especially `WorkflowRunPhaseList.tsx`, `WorkflowRunSidePaneSections.tsx`, `WorkflowMoreRow.tsx`, `WorkflowRoll.tsx`, and `WorkflowRosterParts.tsx`.
- Replaced the old horizontal-card detail dialog with a native right-side run pane backed by the same owning chat/run snapshot. It uses the current shell's 380px slot at >=1180px and a single content pane below that breakpoint. These native shell dimensions are not a measured copy of ZCode's resizable panel.
- The card now keeps five pinned participants plus the up-right-arrow aggregate. Selecting that aggregate opens the pane, expands its phase and roster, and requests post-layout top alignment. Repeating the selection requests the landing again without resetting other disclosures.
- The pane uses independent 36px stage headers, a 39px text-column inset, a vertical linear spine, observed progress, collapsed avatar clusters/44x3px meters, and a reversible 32px roster door. The remaining participants appear once in attention-ordered groups with two columns of 24px rows. Interrupted participants retain their native state rather than being counted as completed or waiting.
- Phase state is local to the open chat/run, opens all newly running phases, retains manual collapse across identical snapshots, and drops removed phase IDs. Node detail IDs are isolated between transcript and pane. Empty or whitespace-only details remain inert.
- Review/file-review opens close the run pane. Closing via X, Escape, or the header side-pane toggle restores the captured focus. Switching chats clears the pane; focusing the composer dismisses the single-column run pane so the editor is visible.
- Verification: `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline` passed with **116 tests**, `cargo check -p pi-desktop --offline`, `cargo build -p pi-desktop --release --offline`, `cargo fmt -p pi-desktop -- --check` and `git diff --check` passed using the Command Line Tools SDK. The final local binary is `/tmp/codex-native-target/release/volt`, SHA256 `99181e7bcbeab3852bf677acb8e9118b159e119a7c3316f1d8c78b1cfc66d909`. These are source/build gates only; they do not promote GUI interaction or pixel acceptance.
- One independent read-only verifier found a shortcut-driven Review transition that removed the pane without transferring focus. All Review toggle callers now pass the `Window`; toggles and file-review opens close the pane through the shared path and then focus the continuously mounted workbench root. The verifier confirmed that correction with no new code blocker; it did not claim GUI focus acceptance or rerun the owner tests.
- Remaining boundaries: the snapshot has no branch topology or child transcripts; no branch graph, actor conversation, configuration/resume capability, or remote state was fabricated. Landing uses immediate deferred native positioning; the reference smooth movement/1200ms flash, exact hover/face fan-out/arrival/meter transitions, rendered light/dark, 480px layout, keyboard focus, and matched pixels still need acceptance.

## Settings save button detail follow-up - 2026-10-29

Parent: the existing ZCode visual alignment objective.
Source: user's attached model-provider settings screenshot and request to refine button styling.
Reason: the save action inherited a navigation row's left-aligned, full-width inner content and a fixed 132px width.
Scope/risk/orchestration: low-risk native single writer; settings save button presentation only, without changing persistence, labels, icons, or success/error behavior.

- Replaced the navigation helper with a settings-owned GPUI-Kit primary Button: content-sized width, start-aligned self placement to prevent flex stretching, centered interface-size text, 32px height, 8px radius, 10px horizontal padding, 16px Check icon, and 4px content gap. Colors, hover, pressed and keyboard-focus behavior remain owned by the component; no duplicate hover registration was introduced.
- Added a builder-style regression test for Chinese/English labels at 12/14/20px, checking height, unspecified width, start alignment and padding. This test does not measure rendered labels, pointer states or pixel parity.
- Verification: `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline` passed **123 tests**; the matching release build passed. `rustfmt --edition 2021 --check crates/pi-desktop/src/settings.rs` and `git diff --check` passed. Package-wide `cargo fmt -p pi-desktop -- --check` remains blocked by existing formatting differences in `execution.rs`, `generative_ui.rs`, and `panels.rs`; those unrelated files were not reformatted.
- Status: **PARTIAL**. The current macOS window probe returned `Screen is locked`; no final-build capture or interaction replay was obtained. Still required: unlocked light/dark settings captures at 1200x800 and 480x640, Chinese/English and largest-text layout, hover/pressed/keyboard focus, save feedback, and matched-reference visual review. The local binary is `/tmp/codex-native-target/release/volt`; existing running windows and screenshots are not evidence for this build.

## Settings navigation button follow-up - 2026-10-29

Parent: the settings save button detail follow-up above.
Source: the user's ongoing ZCode comparison and button-detail feedback.
Reason: settings navigation reused the workbench action's small text and retained an empty label and gap in compact mode.
Scope/risk/orchestration: low-risk native single writer; settings navigation presentation and its local theme token only. Save behavior, section persistence and return-focus handling remain unchanged.

- Compared `SettingsSidebarButton` and Back in the pinned ZCode 3.16.2 source at `3c5b227649371507ddc4eb46ac5133afe6db1cab`.
- Settings-owned content now uses interface-sized 14px/20px text, 16px non-shrinking icons, and an 8px gap only when the visible label exists. Compact targets stay 40x40 with zero padding and centered icons; labels remain available through tooltip and accessibility name.
- Section icons/text use foreground and selection/hover use the reference surface-hover token (5% light / 10% dark). Back uses ArrowLeft, 6px wide inline padding, the idle surface and opaque remote-muted text, with foreground content on group hover. Its instance-level background now pins the idle surface after the component variant is applied, avoiding gpui-kit’s translucent custom-color fallback. Custom variants retain component activation/focus without registering a second button hover handler.
- Added Chinese/English, 12/14/20px wide/compact builder-style coverage, light/dark remote-muted token assertions, and a light/dark wide/compact Back-frame regression for the idle surface and geometry. These are source contracts, not rendered label measurements or interaction acceptance.
- Verification: **126 tests passed**, the matching offline release build passed, `rustfmt --edition 2021 --check` passed for `settings.rs`, `buttons.rs` and `theme.rs`, and `git diff --check` passed. Release binary: `/tmp/codex-native-target/release/volt`, SHA256 `4334c3f1f76292874413e0a4c548d98395e257fc5c404b1ee99d1ceb18315880`. Unrelated formatting-only changes made during checking were removed; `execution.rs`, `generative_ui.rs` and `panels.rs` have no diff.
- A temporary offscreen-render diagnostic did not produce screenshots or runtime evidence: application test-support resolution hit an optional Fast-backend Cocoa version conflict, and a separate diagnostic build exhausted system-disk space. The diagnostic manifest, lockfile and source were removed; application dependencies and lockfile remain unchanged.
- The macOS window probe again returned `Screen is locked`. Post-build light/dark and narrow captures, largest-text layout, pointer/keyboard states, save feedback and matched pixels remain **PARTIAL**. No screenshot from an older running binary is attributed to this change.

## Settings heading typography follow-up - 2026-10-29

Parent: the settings navigation button follow-up above and the full ZCode alignment objective.
Source: the user's ongoing comparison request and active goal continuation.
Reason: page titles and navigation group labels stayed at fixed sizes after the interface font preference changed; group labels used translucent foreground instead of the reference remote-muted color.
Scope/risk/orchestration: low-risk native single writer; settings page title and navigation group typography only. Button geometry, handlers, preference persistence, global text scaling and other page content remain unchanged.

- Rechecked the same pinned ZCode commit, `SettingsPage.tsx` and `styles.css`. Title tokens are additive: interface size +10px with a 4/3 line height compact, or +16px with a 1.2 line height wide. Default 14px rendering therefore retains 24px/32px and 30px/36px.
- Native titles now use those tokens, full available width, a shrinkable minimum width, no fixed height and natural wrapping. Navigation group labels use interface size -2px, 1.5 line height, medium weight and opaque light/dark remote-muted colors; compact navigation still omits them.
- Builder tests cover all eight section labels in Chinese/English at 12/14/20px in both layout modes, plus light/dark group styles. These assert style contracts, not actual glyph measurement, wrapping bounds or rendered contrast.
- Verification: **128 tests passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline -q`; the offline release build, focused `rustfmt --edition 2021 --check` and `git diff --check` passed. Binary: `/tmp/codex-native-target/release/volt`, SHA256 `2d38c6e56819f24a43ae932029ec5b143cba21bb82ca620c5549e4208b38b7d6`.
- The current macOS probe returned `Screen is locked`; no fresh native screenshots or interaction results were obtained. Whole-product parity remains **PARTIAL**, including maximum-font wrapping, light/dark wide/minimum layouts, focus/pointer states and matched-reference pixels. Remaining fixed-size settings body/caption text is outside this heading-only follow-up and still needs source comparison.

## Settings body typography follow-up - 2026-10-29

Parent: the settings heading typography follow-up above and the full ZCode alignment objective.
Source: the user's ongoing comparison request and active goal continuation.
Reason: settings body text still bypassed the interface-size preference in status values, skill rows, footer/provider notes and shortcut keycaps, leaving the page internally inconsistent at larger text sizes.
Scope/risk/orchestration: low-risk native single writer; settings typography inheritance only. The workbench root remains unchanged, and fixed controls, icons, spacing, persistence and handlers remain unchanged.

- Scoped `self.ui_text(14.)` inheritance to the settings frame. This keeps the main workbench's existing typography contract intact while making settings row labels and descriptions follow the interface preference.
- Converted settings captions to the reference base-minus-2px token and larger usage values to base-plus-2px. Provider notes, footer labels, status values, skill names/counts/descriptions and shortcut labels/keycaps now follow the same scale. Fixed 20px/24px line heights remain stable.
- Added builder coverage for the settings frame and caption styles across light/dark themes and 12/14/20px values. These tests verify source style contracts, not rendered glyph metrics or actual line wrapping.
- Verification: **130 tests passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline -q`; the offline release build, focused `rustfmt --edition 2021 --check` and `git diff --check` passed. Binary: `/tmp/codex-native-target/release/volt`, SHA256 `917685f9a71a44b5c0f7f924549063400e469111b87ee7363a4e41a10d4c64e2`.
- The macOS probe again returned `Screen is locked`; fresh light/dark, compact, maximum-size and interaction captures remain unavailable. Whole-product visual parity is **PARTIAL**.

## Settings form control follow-up - 2026-10-29

Parent: the settings body typography follow-up and the full ZCode alignment objective.
Source: the user's attached model-provider settings comparison and pinned ZCode 3.16.2 source.
Reason: single-line fields used `Input::h()`, although GPUI-Kit only applies that builder to multiline editors; the key visibility control retained the library's smaller geometry.
Scope/risk/orchestration: low-risk native single writer; provider input geometry, API-key visibility affordance and the shared font-size field frame. Persistence, IME handling, blank-key retention and session-only key semantics remain unchanged.

- Single-line height is now written to the style applied by the actual component renderer: 32px for endpoint/model/font-size and 36px for API key, with 12px horizontal inset, 8px radius and 20px line height. The native editor owns vertical centering rather than the library's default 8px vertical frame padding. The font-size suffix and right alignment remain intact.
- Replaced the internal key toggle with a 24px ghost button, 14px eye icon, 6px radius, 6px field inset and localized show/hide tooltip/accessibility label. Its native mask toggle notifies the parent so icon and label update with the actual state. The key field reserves 40px for this overlay.
- Added style-builder coverage for light/dark themes, 32/36px heights and 12/14/20px fonts, plus both eye-button states and languages. These are source contracts, not rendered geometry or interaction tests.
- Verification: **132 tests passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline -q`; offline release build, focused `rustfmt --edition 2021 --check` and `git diff --check` passed. Binary: `/tmp/codex-native-target/release/volt`, SHA256 `d3a8e8eb203be1d462e5dea0ec7b338d1da06cbe005284e76470d19c7368f50b`.
- The current macOS window probe returned `Screen is locked`. Fresh light/dark wide/narrow, largest-font, pointer/keyboard, mask-toggle/save-feedback and matched-reference acceptance remain **PARTIAL**. Older captures are not attributed to this binary.

## Settings Select trigger follow-up - 2026-10-29

Parent: the settings form control follow-up above and the full ZCode alignment objective.
Source: the user's ongoing ZCode comparison and the pinned ZCode 3.16.2 Select contract.
Reason: Select triggers retained the library's default vertical padding and horizontal insets, which made the 32px frame visually tighter than the reference control.
Scope/risk/orchestration: low-risk native single writer; settings Select trigger geometry only. Menu behavior, option state, focus handling, accessibility labels and existing width constraints remain unchanged.

- Compared the pinned `components/ui/select.tsx` large trigger: 32px height, 12px left inset, 8px right inset, 8px radius and no vertical frame padding.
- Added `settings_select` on top of the shared settings control frame and applied it to General language, Footer language, Footer theme and Appearance theme. The native GPUI-Kit menu and focus behavior remain the owner of interaction states.
- Added builder coverage for light/dark themes and 12/14/20px interface sizes, checking height, min-width, insets, typography, surface and border. These tests are source contracts, not rendered label measurements or pointer/focus acceptance.
- Verification: **133 tests passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline -q`; the matching offline release build passed. Focused `rustfmt --edition 2021 --check` and `git diff --check` passed. Binary: `/tmp/codex-native-target/release/volt`, SHA256 `a67468df0d379133222a1dbfe6821923d2ff511a78d3f90e72d836087a94cce5`.
- The current macOS window probe returned `Screen is locked`. Fresh light/dark 1200x800 and 480x640 captures, Chinese/English and largest-font layout, pointer/keyboard/focus interaction, and matched-reference pixels remain **PARTIAL**. No older running binary or screenshot is attributed to this change.

## Settings shortcut badge follow-up - 2026-10-29

Parent: the settings Select trigger follow-up above and the full ZCode alignment objective.
Source: user's ongoing button-detail comparison, pinned ZCode 3.16.2 `ShortcutBindingRow.tsx`, `kbd.tsx`, `ControlShortcutBadge.tsx`, `shortcuts/label.ts`, and the final Zai tooltip-tag tokens.
Reason: bindings used a merged cross-platform string and a static keyboard glyph, although the reference uses separate platform-specific keycaps; task search was missing from the displayed native bindings.
Scope/risk/orchestration: low-risk native single writer; shortcut settings presentation only. Key bindings, handlers, persistence, save behavior and the workbench root remain unchanged.

- Split bindings into platform-specific keycaps: Command and Shift symbols with the macOS system font, or Ctrl/Shift labels with the configured monospace font elsewhere. Each badge uses 16px height/minimum width, 6px inline padding/radius, base-minus-4px medium text, the reference tooltip-tag palette and a 4px inter-key gap.
- Removed the static Keyboard glyph, which the reference reserves for recording. Added the existing primary-K task-search binding so all eight native bindings are represented. No recording, rebinding or deletion controls were fabricated.
- Below a 640px viewport, rows stack label and binding with an 8px gap; wider rows retain the 12px horizontal gap. Key groups wrap without shrinking individual badges. Existing 16px/12px row padding and dividers remain.
- Added four focused tests for platform token ordering, actual keycap builder styles across both themes and 12/14/20px fonts, narrow/wide row styles, and group wrapping constraints. The theme palette test brings the suite to **138 passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline -q`. These assert source contracts, not rendered glyph bounds, key dispatch or pixel parity.
- The matching `cargo build -p pi-desktop --release --offline`, focused rustfmt checks and `git diff --check` passed. Binary: `/tmp/codex-native-target/release/volt`, SHA256 `754c537333dcf71c116adffa0faf5fabd18a9b78760b14db23d3ca31b2c92e9f`. The build reports the existing future-incompatibility warning for dependency `block 0.1.6`; no dependencies or lockfile changed.
- The fresh macOS window probe returned `Screen is locked`. No current-build screenshots or interaction replay were obtained. Light/dark 1200x800 and 480x640 captures, Chinese/English and largest-font layout, save/mask controls, hover/pressed/keyboard focus, and matched-reference pixels remain **PARTIAL**. The full visual-parity goal is not complete.

## Settings card clipping follow-up - 2026-10-29

Parent: the settings shortcut badge follow-up above and the full ZCode alignment objective.
Source: pinned ZCode `SettingsPageParts.tsx` `SettingsGroupCard`, plus the user's model-provider settings screenshot.
Reason: the native settings card matched the 12px rounded border and surface but lacked the reference card's overflow constraint.
Scope/risk/orchestration: low-risk native single writer; settings card frame presentation only. Child controls, persistence, handlers and card contents remain unchanged.

- Added `overflow_hidden()` to the shared settings card, used by General, Providers, Appearance, Agent capabilities, Subagents and Shortcuts. Skills and Usage use separate surfaces. GPUI's `Style::overflow_mask` constructs a rectangular content mask, so these flags do not establish rounded child masking or prevent all corner overpaint. Rounded-corner pixels and focus effects remain unverified.
- Added light/dark builder coverage for the overflow axes, frame geometry, radius, surface and border. This is source-level evidence, not a rendered corner pixel comparison.
- Verification: **139 tests passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline -q`; matching offline release build, focused rustfmt checks and `git diff --check` passed. Binary: `/tmp/codex-native-target/release/volt`, SHA256 `e7a826b591c112a999f87e8f72cdbc59028abe91bc796b795769f4cb9ec282f4`.
- The macOS window probe still returns `Screen is locked`; fresh current-build light/dark, compact, focus, interaction and pixel evidence remain **PARTIAL**.

## Settings secondary action detail follow-up - 2026-10-29

Parent: the settings button-detail follow-ups and the full ZCode alignment objective.
Source: user's attached button feedback and ongoing comparison request.
Reason: Use in task and Windows shortcut installation still reused workbench navigation content with full-width inner layout, small text and an 8px icon gap.
Scope/risk/orchestration: low-risk native single writer; settings command presentation only. Handlers, disabled state, persistence, navigation and workbench controls remain unchanged.

- Added settings-owned secondary commands using the pinned `button.tsx` default-size geometry: content width, 28px height, 6px radius, 8px inline padding, 14px icons, 4px gap and centered interface-size labels. The native ghost variant still owns interaction states. Windows installation sets start alignment at its column-layout call site; skill actions retain their row's vertical centering.
- Shared only the existing icon/label content between save and secondary commands; the save button retains its 32px/8px-radius/16px-icon large-button geometry.
- Added two builder tests covering Chinese/English labels and 12/14/20px fonts. They check content-width constraints, geometry, text and gap, not rendered glyph bounds or runtime interactions.
- Corrected the preceding clipping record after inspecting GPUI `Style::overflow_mask`: the mask is rectangular, not rounded, and shared `card()` does not host Skills or Usage. Rounded corner masking remains an open visual difference.
- Final-source verification: **141 tests passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline -q`; matching offline release build, focused rustfmt checks and `git diff --check` passed. Binary: `/tmp/codex-native-target/release/volt`, SHA256 `ba2abe8397ea63f5ad01467c2af6c977bb77c071ab24f7d5c648281dbc4bfb1b`. Dependencies and lockfile remain unchanged; the existing `block 0.1.6` future-incompatibility warning remains.
- The current macOS probe returned `Screen is locked`; no current-build capture or interaction replay was obtained. Matched light/dark 1200x800 and 480x640 captures, Chinese/English and maximum-font layouts, pointer/keyboard states and save feedback remain **PARTIAL**. The full visual-parity objective remains active; no commit, push or deployment was performed.

## Settings interface font control follow-up - 2026-10-29

Parent: the settings secondary action detail follow-up and the full ZCode alignment objective.
Source: pinned ZCode `settingsCodePreview.tsx` `FontSizeInput`.
Reason: the native interface-font control was 32px high and rendered `px` through the input's layout-participating suffix, while the reference uses a 28px compact input with an independently positioned unit label.
Scope/risk/orchestration: low-risk native single writer; Appearance interface-font control presentation only. Range validation, Enter/blur commit, persistence and inherited font scaling remain unchanged.

- Added a 112x28px relative frame. The input now uses a 6px radius, 8px left inset, 32px right reservation and right-aligned value; the `px` label is absolute and noninteractive at the right 8px inset, with interface-size-plus-2px text.
- Added builder coverage for light/dark themes and 12/14/20px interface sizes, including frame geometry, absolute unit placement, text alignment, reserved inset, surface and radius. These are source contracts, not rendered glyph or cursor measurements.
- Final-source verification: **142 tests passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline -q`; matching offline release build, focused rustfmt checks and `git diff --check` passed. Binary: `/tmp/codex-native-target/release/volt`, SHA256 `4d3910d72bdc6f170c1b7a805198fc3ee629bc6b4054053a6424c5172bf92fb4`.
- The current macOS probe remains `Screen is locked`; no current-build capture or interaction replay was obtained. Light/dark 1200x800 and 480x640 captures, Chinese/English and maximum-font layout, pointer/keyboard focus, save feedback and pixel comparison remain **PARTIAL**. No commit, push or deployment was performed.

## Workflow phase landing follow-up - 2026-10-29

Parent: the ongoing ZCode workflow run pane alignment and settings detail follow-ups.
Source: pinned ZCode `WorkflowRunPhaseList.tsx` and the native GPUI `ScrollHandle` implementation.
Reason: the previous ScrollHandle API already deferred positioning until prepaint, but jumped directly to the stage without smooth travel, explicit landing state or user cancellation.
Scope/risk/orchestration: low-risk native single writer; workflow pane phase landing and cancellation state only. Stage grouping, focus restoration, node disclosure and workflow persistence remain unchanged.

- Phase landing now waits for post-layout child bounds, starts from the current vertical offset, and uses a 300ms native cubic-bezier approximation to the reference smooth scroll. The destination is clamped to the live scroll range on every frame.
- Reopening the same stage replaces the current motion, reduced motion lands immediately, and stage removal or an unavailable workflow clears pending state. Wheel input on either axis, pointer and plain keyboard scrolling cancel the landing so the pane does not reclaim user control. An independent read-only verifier identified the horizontal-wheel case because GPUI can map it to vertical scrolling; regression coverage now includes both axes and zero-delta preservation.
- Added seven deterministic state tests covering deferred layout, repeated landing, cancellation on either wheel axis, reduced motion, resize/non-scrollable bounds and stage removal. These tests cover state and coordinate contracts, not native pointer replay or pixel output.
- Final-source verification: **149 tests passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline -q`; matching offline release build, focused `rustfmt --edition 2021 --check` and `git diff --check` passed. Final binary SHA256: `5994ef49150049488cf6c641a93de5e637748e751fbe01b1eee0df566cfe1ee7`. The build retains the existing `block 0.1.6` future-incompatibility warning.
- The reference's transient phase landing surface remains intentionally open after source inspection found duplicate `wf-land` declarations in the pinned stylesheet, with the later opacity keyframes overriding the earlier background-color keyframes. The screen briefly became available and the pre-review Release rendered successfully; see the limited capture evidence below. Stage landing itself was not replayed. Overall visual parity remains **PARTIAL** and no commit, push or deployment was performed.

## Settings button capture checkpoint - 2026-10-29

Parent: the user's attached save-button feedback and the settings detail follow-ups above.
Source: the isolated native Release process and direct inspection of its captured model-provider settings page.
Reason: source-style checks alone did not establish that the content-sized save action rendered correctly.
Scope/risk/orchestration: read-only native GUI checkpoint using an offline temporary state directory; no real credentials, requests or user-state changes.

- The isolated 1200x800 window successfully rendered the light-theme workflow and model-provider settings pages. AXPress navigation opened Preferences and Model providers. The save button visibly sizes to its Check icon and Chinese label with centered content, without the previous large right-side gap. Fields and the action remain visible without obvious overlap at this size.
- Preserved full-window captures outside the repository:
  - Workflow: `/Users/zhd/.codex/visualizations/2026/10/26/01a1764d-5a1f-74b2-8fa3-c0499328387b/workflow-light-20261029.png`
  - Settings: `/Users/zhd/.codex/visualizations/2026/10/26/01a1764d-5a1f-74b2-8fa3-c0499328387b/settings-save-light-20261029.png`
- These captures precede the final horizontal-wheel cancellation fix, which did not change settings presentation. They are not screenshots of the final binary hash and are not matched ZCode pixel comparisons. The settings capture contains a navigation tooltip.
- At 23:26 local time the selected-window probe returned `Screen is locked`; subsequent capture failed. Dark/480x640/maximum-font rendering, hover/pressed/keyboard focus, save feedback, key visibility and workflow landing replay remain unverified. The isolated test process was stopped; the user's existing application processes and real state were not touched.

## Workflow roster-door interaction follow-up - 2026-10-29

Parent: the ongoing full ZCode visual-alignment objective and workflow roster continuation.
Source: the user's explicit goal continuation, pinned `WorkflowMoreRow.tsx`, and the `.wf-more-deck`, `.wf-pill-name`, `.wf-pill-open` rules in `styles.css`.
Reason: the aggregate button kept a static -5px face overlap, changed its surface abruptly and did not promote its name to primary foreground on hover or keyboard-visible focus.
Scope/risk/orchestration: medium-risk native single writer with one independent read-only verifier, limited to the shared timeline/pane aggregate control. Roster ordering, counts, disclosure handlers, workflow persistence and the full objective remain unchanged.

- Added a local render component to own per-row hover, focus and transition state without widening the workflow renderer's signatures or adding application-global interaction maps. Its focus subscriptions leave with keyed element state. The native Base Button retains keyboard activation, accessibility label, aria-expanded, tooltip, ring and the existing click handler.
- Pointer hover and keyboard-visible focus animate the face overlap from -5px to -2px over 160ms with cubic-bezier(0.22, 0.61, 0.36, 1), matching the pinned stylesheet. A single interruptible engagement channel drives face spacing, alpha surface and label foreground; reduced motion uses zero duration. The open pane door remains emphasized independently from hover.
- Added four deterministic tests for the overlap bounds/timing, light/dark alpha surfaces and text, the open-door presentation, and all hover/focus/modality combinations. They do not prove hover delivery, focus traversal, actual truncation or rendered pixels.
- The independent verifier found that the first implementation selected hover or focus exclusively by input modality, and GPUI's default hover listener suppresses hover after keyboard input. Corrected engagement to hover OR keyboard-visible focus and selected `InputModalityIndependent` for the single hover listener. The regression includes Tab moving focus away while the mouse remains over the row. The same verifier confirmed the correction without new findings; real GUI replay remains pending.
- Re-inspected the landing keyframes rather than following the reference's stale background-flash comment: `.wf-landed` uses 1200ms, but the last `@keyframes wf-land` scales from 0.6 and fades from zero. Current GPUI exposes transforms for SVG/icon painting but no generic subtree scaling builder. Exact landing/pressed scale remains open; this follow-up does not replace it with a different visual effect or call it complete.
- Final-source verification, completed across midnight on 2026-10-29/30: **153 tests passed**, the matching offline Release build, focused rustfmt and `git diff --check` passed. Release SHA256: `37f5c2c728668833cfe1418ac23ad3892f9f0d73c77c2bd780212939c4c6682a`. Whole-package `cargo fmt --check` also reports pre-existing formatting differences in `execution.rs`, `generative_ui.rs` and `panels.rs`; those unrelated files were left unchanged. The existing dependency warning for `block 0.1.6` remains; manifests and lockfile did not change.
- At 00:00 on 2026-10-30, the selected-window probe again returned `Screen is locked`. No GUI was launched in this follow-up. This change has no new rendered or interaction evidence; prior light settings captures are not evidence for it. Overall visual parity remains **PARTIAL**, including full light/dark, 480x640, keyboard/pointer and matched-reference verification. No commit, push or deployment was performed.

## Workflow roster settlement and disclosure motion - 2026-10-30

Parent: the workflow roster-door interaction follow-up and the full ZCode alignment objective.
Source: pinned ZCode `WorkflowRunPhaseList.tsx`, `WorkflowRosterParts.tsx`, `WorkflowMoreRow.tsx`, and `.wf-motion` / `.wf-meter-seg` / chevron rules in `styles.css`.
Reason: pane meters changed segment weights immediately, and phase/roster disclosure icons swapped glyphs instead of preserving one icon and rotating it.
Scope/risk/orchestration: medium-risk native single writer with the existing read-only verifier reused; pane roster presentation and disclosure motion only. Workflow state, stage ordering, click handlers, focus ownership and persistence remain unchanged.

- Added a keyed native roster meter component with a fixed 44x3px frame, 1px inter-segment gap, 2px minimum segment width and observed node-count weights in the reference order: completed, failed, running, waiting, interrupted. Existing status colors remain semantic; interrupted remains distinct from waiting. Existing segments transition their flex weights over 320ms with the shared workflow easing, and reduced motion settles immediately.
- Added a shared native chevron component. Pane stage headers keep one 14px ChevronRight and rotate to 90 degrees over 150ms with the reference Tailwind easing; roster doors keep one 14px ChevronDown and rotate to 180 degrees over 160ms with the roster easing. The card's always-present up-right arrow remains unchanged, while its idle-to-engaged tail color now follows the same aggregate interaction channel.
- Added five deterministic tests for count/order semantics, fixed meter geometry, themed segment styles, chevron targets and reduced-motion timing. They verify source/state contracts, not rendered segment interpolation, pointer replay or pixels.
- The independent read-only verifier found no new code blocker in the scoped changes: fixed geometry, transition-key isolation, reduced-motion policy and existing Button handlers/focus/expanded semantics remain intact. This was source review, not runtime or visual acceptance.
- Verification after the source correction: **158 tests passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline -q`; matching offline Release build, focused rustfmt and `git diff --check` passed. Release SHA256: `0db50ddc4e52506374c19f28a68fcac028bab51331cc3dde6cd40263baf514f1`. The build retains the existing `block 0.1.6` future-incompatibility warning; dependencies and lockfile remain unchanged.
- The current macOS window probe returned `Screen is locked`; no current-build capture or interaction replay was obtained. Light/dark, compact, reduced-motion GUI replay, hover/focus states, exact meter interpolation and matched-reference pixels remain **PARTIAL**. No commit, push or deployment was performed.

## Workflow entrance and roster grid follow-up - 2026-10-30

Parent: the workflow roster settlement and disclosure motion follow-up and the full ZCode alignment objective.
Source: pinned ZCode `WorkflowTimeline.tsx`, `WorkflowRoll.tsx`, `WorkflowRunPhaseList.tsx`, and `wf-arrive`/`wf-unfold` rules in `styles.css`.
Reason: timeline and pane rows still appeared synchronously, while the pane roster needed the reference's shared two-column roll and row cadence.
Scope/risk/orchestration: medium-risk native single writer with one bounded read-only verifier; entrance presentation and roster layout only. Workflow ordering, disclosure state, handlers, focus ownership, persistence and backend semantics remain unchanged.

- Added a keyed `WorkflowEntrance` render wrapper. Timeline pills use the global 30ms ordinal delay, including the aggregate row; pane roster rows use 8ms per row capped at 400ms. The wrapper approximates `wf-arrive` from +4px and `wf-unfold` from -3px over 200ms with the reference cubic-bezier, while relative positioning preserves sibling layout. Reduced motion settles immediately and removes delay.
- Reshaped the expanded pane roster into one `grid-cols-2` frame with an 8px column gap and 2px top inset. Group headings span both columns at 22px height, use 4px left padding and 5px internal spacing, and add 4px only between groups. Existing attention ordering, source ordering, counts, node handlers and accessible disclosure semantics remain unchanged.
- Added deterministic coverage for delayed/reduced-motion timing, start offsets and stable layout dimensions, the two-column frame, heading spans and reference spacing. These are source/state contracts, not rendered interpolation, pointer replay or pixel parity.
- The independent read-only verifier found no new scoped blocker: mounted animation keys are stable and surface-scoped, reduced motion clears both timing components, timeline ordinals include the aggregate row, and the single roll grid preserves cross-group row delays. This is source review, not GUI lifecycle or interaction acceptance. The owner separately compiled and passed both new geometry tests as part of the 164-test run.
- Verification: **164 tests passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline -q`; matching offline Release build passed. Focused `rustfmt --edition 2021 --check` and `git diff --check` passed. Release binary SHA256: `bc127ee97e60209bc9acb891240ab1aa180aad12e16410ce4f476b1c3fa881bd`. The build retains the existing `block 0.1.6` future-incompatibility warning; dependencies and lockfile remain unchanged.
- The macOS window probe in this follow-up on `2026-10-30` returned `Screen is locked`. No current-build screenshot or interaction replay was obtained. Light/dark, compact, reduced-motion GUI replay, hover/focus states, exact entrance interpolation and matched-reference pixels remain **PARTIAL**. No commit, push or deployment was performed.

## Settings command hover token follow-up - 2026-10-30

Parent: the settings secondary action detail follow-up and the full ZCode alignment objective.
Source: pinned ZCode `button.tsx`, `select.tsx`, and `.theme-zai-light` / `.theme-zai-dark` tokens in `styles.css`.
Reason: GPUI-Kit's Ghost variant resolves hover through its accent palette, while ZCode settings commands use the neutral 5% hover surface in both themes.
Scope/risk/orchestration: low-risk native single writer; settings command color presentation only. Geometry, handlers, persistence, disabled behavior, focus ownership and save semantics remain unchanged.

- Added an explicit settings command variant with transparent idle, primary foreground, neutral hover/pressed surface (`hover` token) and native component focus treatment. Applied it to Windows shortcut installation, Skills “Use in task”, and the API-key visibility button. The global Ghost/navigation palette remains unchanged.
- This fixes the source-level color-token mismatch without inventing a new interaction model. GPUI-Kit's color state is still immediate rather than the reference CSS `transition-colors` timing, so rendered transition acceptance remains open.
- Verification: **164 tests passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline -q`; matching offline Release build passed. Focused rustfmt and `git diff --check` passed. Release binary SHA256: `e3714ca6eb2a0258cf3e20c8ecb5c01f640d757be6a434997ebdaef11d9aa250`. The existing `block 0.1.6` future-incompatibility warning remains; dependencies and lockfile are unchanged.
- The macOS window probe on `2026-10-30` returned `Screen is locked`. No current-build screenshot or interaction replay was obtained; light/dark, compact, pointer/keyboard focus, color-transition timing and matched-reference pixels remain **PARTIAL**. No commit, push or deployment was performed.

## Settings command color motion and tooltip follow-up - 2026-10-30

Parent: the settings command hover token follow-up and the full ZCode alignment objective.
Source: the user's save-button detail feedback, pinned ZCode `button.tsx` and its Tailwind color-transition defaults.
Reason: command colors still changed immediately, and the first transition implementation competed with the eye button's managed tooltip for GPUI's single hover listener.
Scope/risk/orchestration: medium-risk native single writer with one independent read-only verifier; settings save, shortcut-installation, skill-use and API-key visibility commands only. Button geometry, handlers, persistence, keyboard activation and native disabled semantics remain unchanged.

- Added keyed, interruptible background and foreground transitions over 150ms with cubic-bezier(0.4, 0, 0.2, 1). Save retains black/white ink and fades its primary fill to 80% alpha on hover. Neutral commands transition from transparent to the existing neutral hover token. Reduced motion and disabled state settle immediately; pointer tracking remains input-modality independent.
- The verifier identified a duplicate-hover registration in the initial implementation: the component's managed tooltip also calls `on_hover`. The eye button now keeps the styled Tooltip content through GPUI's separate native tooltip builder, preserving the localized accessibility label and fixed 24x24 geometry without a layout wrapper. The verifier confirmed the source-level conflict is closed with no new scoped finding. Native tooltip placement and event delivery remain unverified.
- Four deterministic color-policy tests cover light/dark paint, disabled targets, reference timing, reduced motion and bounded alpha interpolation. The final full desktop run passed **168 tests** with `CARGO_TARGET_DIR=/Volumes/Datas/tmp-codex-native-target TMPDIR=/Volumes/Datas/tmp-codex-native-build-temp cargo test -p pi-desktop --offline -q`. These tests do not render the tooltip or replay pointer/keyboard interactions.
- The matching offline Release build passed. Its binary is `/Volumes/Datas/tmp-codex-native-target/release/volt`, verified as a 33,127,056-byte arm64 Mach-O executable; SHA256 `28184d96ea6b56f685e5fae8b8602cfc5bff6e278204c9829b159717ce77d190`. This supersedes the failed system-disk build attempt, whose truncated output is not a valid artifact. The existing `block 0.1.6` future-incompatibility warning remains; dependencies and lockfile are unchanged.
- Focused rustfmt checks and `git diff --check` passed. The current `window_info.swift` probe returned `Screen is locked`; no final-build screenshot or interaction replay was obtained. Light/dark, 1200x800 and 480x640, maximum-font layouts, hover/pressed/focus behavior, save feedback, key visibility, actual color-transition timing and matched-reference pixels remain **PARTIAL**. The full objective remains active. No commit, push or deployment was performed.

## Settings command border geometry follow-up - 2026-10-30

Parent: the settings command color motion follow-up and full ZCode alignment objective.
Source: the user's save-button screenshot and the pinned `components/ui/button.tsx` base `border border-transparent` rule.
Reason: native Primary, Ghost and Custom buttons do not reserve a border by default, unlike the reference command frame.
Scope/risk/orchestration: low-risk native single writer; settings save, secondary commands and API-key visibility geometry only. Navigation, handlers, persistence and focus ownership remain unchanged.

- Added an explicit 1px transparent border to these command builders. Content-sized actions now include the reference border reservation; existing fixed heights, inline padding, radii and icon sizes remain unchanged. The eye control stays 24x24px.
- Added builder coverage for all four border edges and transparency across both languages, 12/14/20px interface sizes and both key-visibility states. This does not establish native rendering, focus-ring appearance or CSS `bg-clip-padding` pixel equivalence.
- Verification: **169 tests passed**; matching offline Release build, focused rustfmt and `git diff --check` passed. Binary: `/Volumes/Datas/tmp-codex-native-target/release/volt`, SHA256 `68b6bbcea5b2903cbbbf6a55e61ccc2b89b184084b7901f49dcab38984bbdcaf`. Whole-workspace format checking still reports the existing unrelated differences in `execution.rs`, `generative_ui.rs` and `panels.rs`; they were left unchanged. The dependency warning for `block 0.1.6` remains.
- The current macOS probe returned `Screen is locked`. No current-build screenshot or interaction replay was obtained; prior light captures are historical diagnostics only. Overall visual parity remains **PARTIAL** pending unlocked light/dark, compact, maximum-font, pointer/keyboard and matched-reference checks. No commit, push or deployment was performed.

## Settings navigation color motion follow-up - 2026-10-30

Parent: the settings command border geometry follow-up and full ZCode alignment objective.
Source: pinned ZCode `SettingsSidebarButton` and settings back-button classes, including `transition-colors`, `bg-surface`, `bg-surface-hover`, `text-remote-foreground-muted` and `text-foreground`.
Reason: settings navigation and Back used the correct idle/hover tokens but changed them immediately, unlike the reference color transition.
Scope/risk/orchestration: medium-risk native single writer; settings navigation and Back presentation only. Selection persistence, handlers, keyboard activation, focus semantics, tooltips and layout remain unchanged.

- Added keyed, interruptible 150ms transitions to navigation and Back backgrounds/foregrounds. Selected navigation remains emphasized after pointer exit; unselected rows fade to transparent when idle. Back transitions its idle surface and muted text together. Reduced motion and disabled states settle immediately.
- Reused the existing native tooltip builder for navigation rows so the color-motion hover listener does not compete with the managed tooltip listener. The eye button uses the same helper; no layout wrapper or duplicate hover registration was added.
- Added deterministic coverage for selected navigation, Back foreground/surface interpolation, disabled behavior and reduced-motion timing. This is source/state evidence, not rendered pointer, focus or tooltip placement evidence.
- One independent read-only reviewer confirmed tooltip/hover slot separation, unique stable navigation keys and preserved click/persistence/return-focus calls. The review was stopped before its complete selected/hover/focus override audit and is recorded as **incomplete**, not a clean review receipt.
- Verification: **171 tests passed**; matching offline Release build, focused rustfmt and `git diff --check` passed. Binary: `/Volumes/Datas/tmp-codex-native-target/release/volt`, SHA256 `2ee9a5c8c7aebaa28a9c932b69d2c0cdfc0372747dc378f2690ad9c8b003da73`. The existing `block 0.1.6` future-incompatibility warning remains.
- The current macOS probe returned `Screen is locked`. No current-build screenshot or interaction replay was obtained; light/dark, compact, hover/focus, tooltip placement and matched-reference pixels remain **PARTIAL**. No commit, push or deployment was performed.

## Settings disabled-command palette follow-up - 2026-10-30

Parent: the settings navigation color motion follow-up and full ZCode alignment objective.
Source: the user's button-detail feedback, pinned `button.tsx` `disabled:opacity-50`, and installed GPUI Component/Base 0.7.3 state resolution.
Reason: disabled commands inherited the library's muted foreground and 15% fill instead of retaining idle colors with whole-control opacity.
Scope/risk/orchestration: medium-risk native single writer with one bounded independent read-only verifier; settings command presentation only. Handlers, persistence, native disabled semantics, focus, keyboard activation, geometry and dependencies remain unchanged.

- Pin both instance background and foreground for enabled and disabled commands, and apply 50% opacity only while disabled. The component replays these instance colors after selected/disabled palettes. Disabled targets remain idle even while the pointer is inside, and disabled/reduced-motion transitions settle immediately. Removed the redundant neutral variant helper.
- Added deterministic builder coverage for all command kinds in both themes, checking whole-control opacity, retained colors, content-sized width, height, padding and transparent border reservation. This does not establish rendered compositing, input delivery or pixel parity.
- The independent verifier completed the previously incomplete scoped selected/disabled/focus override audit without a new blocker. Native Base disabled activation, keyboard/focus behavior and the separate tooltip/hover slots remain intact in source; this is not GUI acceptance.
- Verification: **172 tests passed** with `CARGO_TARGET_DIR=/tmp/codex-native-target cargo test -p pi-desktop --offline`; focused rustfmt and `git diff --check` passed. The matching offline Release build used `CARGO_TARGET_DIR=/Volumes/Datas/tmp-codex-native-target TMPDIR=/Volumes/Datas/tmp-codex-native-build-temp`; binary SHA256: `429bde77c757f2cc906b270583a78f5a9e208413800d97c980d041b210ae5064`. The existing `block 0.1.6` warning and unrelated whole-workspace formatting differences in `execution.rs`, `generative_ui.rs` and `panels.rs` remain unchanged.
- The window-list helper initially returned a window, but the attempted capture failed with `could not create image from window`; a direct `CGSSessionScreenIsLocked` probe returned `1`, including the final probe. No screenshot was obtained. The isolated temporary-state Release process was stopped; no real configuration or existing application process was changed. Light/dark, compact, maximum-font, hover/pressed/focus, tooltip placement and matched-reference pixel acceptance remain **PARTIAL**. No commit, push or deployment was performed.

## Settings input border feedback follow-up - 2026-10-30

Parent: the settings disabled-command palette follow-up and full ZCode alignment objective.
Source: the user's settings/button detail feedback, explicit continuation, and pinned ZCode `components/ui/input.tsx` plus Zai input tokens at `3c5b227649371507ddc4eb46ac5133afe6db1cab`.
Reason: settings text inputs lacked the reference hover-border transition and still inherited the component's additional focus ring.
Scope/risk/orchestration: medium-risk native single writer with one bounded independent read-only verifier; provider and interface-font text-input presentation only. The full objective, native editor entities, callbacks, masking, IME, persistence, command behavior and dependencies remain unchanged.

- Added a keyed `SettingsField` observation shell around each existing native Input. Pointer hover or native editor focus targets the existing 15% neutral border; idle uses 10%. Border transitions use the reference 150ms cubic-bezier(0.4, 0, 0.2, 1), with zero duration under reduced motion. The input keeps its existing fill and uses this border instead of the library's extra focused border/ring.
- The shell adds no frame, padding, background, fixed height or tab stop. Original 28/32/36px inputs, the separate API-key eye button and px suffix keep their existing geometry. Select triggers and their menus remain outside this change.
- Added three deterministic tests covering light/dark hover/focus targets, timing/reduced motion, bounded alpha interpolation and the unframed shrinkable shell. These tests do not render the controls or exercise native pointer, focus traversal, selection, masking or IME.
- Verification: **175 tests passed** with `CARGO_TARGET_DIR=/Volumes/Datas/tmp-codex-native-target TMPDIR=/Volumes/Datas/tmp-codex-native-build-temp cargo test -p pi-desktop --offline -q`; matching offline Release build, focused rustfmt and `git diff --check` passed. Binary: `/Volumes/Datas/tmp-codex-native-target/release/volt`, arm64 Mach-O, SHA256 `62e9f55f3d0386df2139450235762867b7da252372ac3975e985e36ce813b1e5`. The existing `block 0.1.6` future-incompatibility warning remains.
- Independent source review completed with no scoped blocker. It confirmed keyed focus subscriptions and one hover listener, preserved native editor/IME/masking/callback paths, replacement border precedence after disabling the library focus ring, unchanged 28/32/36px input frames, and unchanged save-button content sizing and activation semantics. This is source review only. A direct `CGSSessionScreenIsLocked` probe returned `1` on 2026-10-30; no GUI process was launched or new screenshot obtained. Light/dark, compact, maximum-font, pointer/keyboard focus, actual border interpolation, eye/suffix hit testing and matched-reference pixels remain **PARTIAL**. Prior captures are historical diagnostics, not acceptance for this binary. No commit, push or deployment was performed.

## Settings Select hover and state ownership follow-up - 2026-10-30

Parent: the settings input border feedback follow-up and full ZCode alignment objective.
Source: the user's explicit continuation, pinned ZCode `components/ui/select.tsx` / Zai tokens, and installed GPUI Component 0.7.3 Select rendering and event delivery.
Reason: settings Select triggers lacked neutral hover-border feedback; adding independent hover state exposed existing reuse of one native SelectState at two simultaneously rendered locations. That state holds only one style, popup bounds, focus and open state.
Scope/risk/orchestration: medium-risk native single writer with the same bounded independent read-only verifier; settings Select presentation and per-trigger ownership plus language/theme value synchronization. The full visual objective and native menu, keyboard/activation, persistence and save semantics remain unchanged.

- Added a keyed, non-focusable 32px shell with one input-modality-independent hover listener. Hover transitions the neutral border from 10% to 15% over 150ms with cubic-bezier(0.4, 0, 0.2, 1); reduced motion settles immediately. The original Select retains focused/open border precedence. Its focus changes remain immediate, not reference-transition acceptance.
- Each General/Appearance/footer trigger now owns an independent native Entity. Existing language handlers synchronize both language values and refresh both theme menus' localized labels; theme confirmation synchronizes both theme values. This prevents style, focus, popup bounds and open state from being shared between different triggers. No custom menu engine or activation path was introduced.
- Content shells use explicit full width; the two footer shells use zero-basis flex and a zero minimum width. The shell adds no padding, paint, border or overflow clipping. Four tests cover exact light/dark hover colors, timing/reduced motion and bounded interpolation, stable unpainted geometry and footer/content width contracts. They do not exercise native layout or menu input.
- Independent source review confirmed distinct state ownership and synchronization, queued Confirm event delivery without source state/list reentry, and no recursive Confirm emission from `set_selected_index`. Native popup positioning, keyboard focus restoration and actual rendered states remain unverified.
- Final verification: **179 tests passed** with `CARGO_TARGET_DIR=/Volumes/Datas/tmp-codex-native-target TMPDIR=/Volumes/Datas/tmp-codex-native-build-temp cargo test -p pi-desktop --offline -q`; matching offline Release build passed, focused rustfmt and `git diff --check` passed. Binary: `/Volumes/Datas/tmp-codex-native-target/release/volt`, arm64 Mach-O, SHA256 `5807846140aec74f444b0910f61556bcf9ede4792fee86a5d12d671be4a8802f`. The existing `block 0.1.6` future-incompatibility warning remains. The direct screen-lock probe returned `1`; no native GUI process was launched or new screenshot obtained. Focus-transition timing, caret sizing/color, light/dark and compact/maximum-font layout, menu anchoring, pointer/keyboard and matched-reference acceptance remain **PARTIAL**. No commit, push or deployment was performed.

## Settings screenshot recheck and acceptance blocker - 2026-10-30

Parent: the settings Select ownership follow-up and full ZCode alignment objective.
Source: the user's attached Save-button screenshot and explicit request to continue comparing details.
Reason: recheck the accumulated button corrections before making further unobserved presentation changes.
Scope/risk/orchestration: low-risk native owner; source inspection and deterministic verification only. No application code, dependencies, configuration, or running application state was changed in this recheck.

- Reconfirmed the pinned reference commit and Save's existing content-sized 32px frame, 8px radius, transparent border, 10px horizontal padding, centered 16px Check/label content, 4px gap and themed hover/disabled policy. These are implementation contracts, not a new rendered acceptance receipt.
- The reference Select caret is explicitly 14px and subtle. The installed native default caret is 16px; its icon override forces 12px and muted ink. Changing the Select size would also change its list delegate size. No caret workaround was introduced; that difference remains open.
- Re-ran all **179 desktop tests**, focused settings-control rustfmt checks and `git diff --check` successfully. The matching offline Release build passed; `/Volumes/Datas/tmp-codex-native-target/release/volt` retains SHA256 `5807846140aec74f444b0910f61556bcf9ede4792fee86a5d12d671be4a8802f`. The existing `block 0.1.6` future-incompatibility warning remains.
- The direct `CGSSessionScreenIsLocked` probe again returned `1`. This repeats the same acceptance blocker across consecutive follow-ups. No current-build screenshot or pointer/keyboard replay was obtained. Further visual acceptance requires an unlocked desktop; the full objective remains **PARTIAL**, with goal execution **blocked** at this boundary rather than reported complete.
- After unlock, compare current-build light/dark screenshots at 1200x800 and 480x640, including 20px interface text, Save hover/focus/feedback, key visibility, and Select menu anchoring. Previously documented source differences and broader workflow/Markdown gaps remain open; this recheck does not reduce the full objective or add unrelated tooling work. No commit, push or deployment was performed.

## Settings Save button semantic variant follow-up - 2026-10-30

Parent: the settings screenshot recheck and acceptance blocker.
Source: the attached Save-button screenshot, pinned ZCode `button.tsx` large variant, and the installed GPUI Component 0.7.3 Button size semantics.
Reason: the Save button already matched the visible 32px frame and content sizing, but it used the native `small` variant as its base and then overrode the frame. Align the underlying size choice with the reference large variant. Its custom child already owns label/icon sizing, so this does not establish a visible correction or a focus-treatment difference.
Scope/risk/orchestration: low-risk native single writer; Save presentation semantics only. Click handling, persistence, feedback, disabled behavior, hover motion, geometry overrides, and dependencies remain unchanged.

- Changed the Save command to use the native large Button variant while preserving the explicit 32px height, 8px radius, 10px inline padding, transparent border reservation, Check glyph, 4px gap and interface-size label.
- Existing localized builder coverage still checks its geometry; updated the design contract. This verifies source geometry only; it does not establish rendered pixels or pointer/focus behavior.

## Settings Select indicator follow-up - 2026-10-30

Parent: the Save semantic variant follow-up and full ZCode alignment objective.
Source: explicit continuation of the user's settings-detail comparison and pinned ZCode `select.tsx` default `size-3.5` indicator.
Reason: the native default caret is 16px, while changing Select size to obtain a 14px caret would also change its menu delegate. Custom icons are adapted to 12px by the installed component.
Scope/risk/orchestration: low-risk native single writer; settings Select indicator constraints only. Trigger frames, native state ownership, menu sizing, handlers, focus, persistence and dependencies remain unchanged.

- Supply the existing ChevronDown asset with 14px minimum/maximum constraints on both axes, which survive the component's custom-icon xsmall adaptation. This leaves the Select's own size and native popup path intact. One focused builder test checks those constraints and the absence of extra paint/padding.
- Confirmed that `sync_component_theme` already maps `muted_foreground` directly to the reference `foreground_subtle`, so no palette change is needed. Rendered indicator bounds and menu interaction still require current-binary verification.
- Verification: **180 desktop tests passed**; matching offline Release build, focused `rustfmt --edition 2021 --check`, and `git diff --check` passed. Final binary: `/Volumes/Datas/tmp-codex-native-target/release/volt`, SHA256 `2794a5689d2ba4435005c06c47f9f11f90ce6a96a6f622ff42f1dd0cabbad599`. The existing `block 0.1.6` future-incompatibility warning remains. An initial format command incorrectly used edition 2024; its import-order differences were not applied, and the correct project-edition check passed.
- Capture is available again; a missing screen-lock dictionary key returned `unknown`, not confirmation of a locked desktop. The isolated pre-change binary (`580784...`) rendered General at 1200x800 in light mode. Its accessibility tree measured the Save button at **98x32px**, and the screenshot showed centered icon/label without the old trailing gap. Capture: `/var/folders/h7/9t4_q7bd6p9489dhdy_bdt400000gn/T/codex-shot-2026-10-30_08-12-39.png`. This is baseline layout evidence, not acceptance of the final indicator change.
- The final binary was launched with a separate temporary state containing only a local test endpoint, fixture model, dark theme and 20px interface preference. Its dark startup screen rendered, but Settings interaction was not completed: System Events returned `osascript is not allowed assistive access` (`-25211`), and the window bounds changed between probes. Stopped coordinate automation rather than applying further uncertain pointer input. Startup capture: `/var/folders/h7/9t4_q7bd6p9489dhdy_bdt400000gn/T/codex-shot-2026-10-30_08-26-23.png`. Final Settings light/dark, 480x640, maximum-font, Save hover/focus/feedback, eye toggle, rendered 14px indicator, menu anchoring and matched-reference acceptance remain **PARTIAL**. No commit, push or deployment was performed.

## Settings narrow-window row follow-up - 2026-10-30

Parent: the settings Select indicator follow-up and full ZCode alignment objective.
Source: the user's explicit continuation and button-detail screenshot, the native 480x640 minimum-window contract, and pinned `settings/SettingsPageParts.tsx`.
Reason: a fixed 192px control column leaves little room for labels/descriptions in narrow native windows, especially at 20px interface text.
Scope/risk/orchestration: low-risk native single writer; General, Appearance, capabilities and Subagents row layout only. Provider fields already use full-width vertical layout. Save geometry, control entities, menus, callbacks, persistence and dependencies remain unchanged.

- Rows now stack label/description above the full-width control frame below a 640px viewport width. Wide rows keep the 192px column. Both modes retain 16px gaps and 16px/12px padding, with no fixed row height. Select fills the narrow control row; fixed font/switch controls remain right-aligned.
- Reference clarification: the pinned default SettingsRow itself always uses `minmax(0,1fr) + 192px`; only its optional wide variant has a single-column breakpoint. This change is an explicit native minimum-window adaptation, not an asserted default-reference breakpoint.
- Added two deterministic builder tests for row direction, alignment, natural height, padding/gaps and control-column width. **182 desktop tests passed** with `CARGO_TARGET_DIR=/Volumes/Datas/tmp-codex-native-target TMPDIR=/Volumes/Datas/tmp-codex-native-build-temp cargo test -p pi-desktop --offline -q`. The matching offline Release build passed. Binary: `/Volumes/Datas/tmp-codex-native-target/release/volt`, SHA256 `2dec9fade5b271aca0cb683bc9d8a644dd11f5c32d352040952d2d9e0937d671`. Focused Rust 2021 rustfmt and `git diff --check` passed. The existing `block 0.1.6` warning and unrelated format differences in `execution.rs`, `generative_ui.rs` and `panels.rs` remain unchanged.
- GUI acceptance is **PARTIAL**: Screen Recording and Accessibility permission probes passed, but final-build window capture returned `could not create image from window`; the identity/geometry helper explicitly returned `Screen is locked`. No current-build screenshot, narrow resize, maximum-font layout, Save interaction or Select menu acceptance was obtained. The isolated dark/English/20px fixture process was stopped; real application state and existing processes were not changed. No commit, push or deployment was performed.

## Assistant Markdown table overflow follow-up - 2026-10-30

Parent: the ongoing ZCode alignment objective and the prior Markdown table/source comparison.
Source: pinned ZCode `markdown-table.tsx` and installed GPUI Base `TextView` table layout.
Reason: native Markdown tables used the adaptive wrapping path without horizontal overflow, so a column wider than the transcript could become clipped or unreachable instead of remaining scrollable like the reference table frame.
Scope/risk/orchestration: medium-risk native single writer with one bounded independent read-only verifier; assistant Markdown table frame presentation only. Markdown parsing, selectable text, table colors, cell padding, heading treatment and transcript scroll ownership remain unchanged.

- Added a dedicated assistant table-frame style with the existing 12px radius and chat-surface background, opting into GPUI Base's native horizontal-scroll layout. The renderer keeps its measured adaptive columns and wrapping behavior; it does not force `nowrap`, alter vertical overflow, or add a custom scrollbar/menu implementation.
- Added one deterministic style test for light/dark background, radius, horizontal overflow, unconstrained frame dimensions and preserved wrapping. This verifies the style contract, not actual column measurement, scrollbar painting, wheel bubbling, selection or corner pixels.
- Independent-review correction: the bounded read-only verifier was closed while still running and returned no review conclusion. This follow-up has no completed independent-review receipt; the source tests/build below do not replace it.
- Verification: **183 desktop tests passed** with `CARGO_TARGET_DIR=/Volumes/Datas/tmp-codex-native-target TMPDIR=/Volumes/Datas/tmp-codex-native-build-temp cargo test -p pi-desktop --offline -q`; focused Rust 2021 rustfmt and `git diff --check` passed. The matching offline Release build passed. Binary: `/Volumes/Datas/tmp-codex-native-target/release/volt`, arm64 Mach-O, SHA256 `85b9cb95d4ec83e06069f0867aae484afec4c2b51be39a8b64aa277d636df305`. The existing `block 0.1.6` warning remains; no dependency or lockfile changes were made.
- GUI acceptance remains **PARTIAL**. The current `window_info.swift` probe returned `Screen is locked`; no final-build Markdown table capture or wheel/selection replay was obtained. The reference's edge shadows, virtual scrollbar, toolbar actions and preview dialog remain outside this narrow follow-up. No commit, push or deployment was performed.

## Settings Save feedback layout follow-up - 2026-10-30

Parent: the settings Save button detail follow-ups and full ZCode alignment objective.
Source: the user's attached button screenshot, explicit continuation, and reinspection of the pinned `button.tsx` large-button geometry.
Reason: the Save frame is already content-sized, but its success text was a separate child of the page's 32px-gap column, visually disconnecting feedback from the action.
Scope/risk/orchestration: low-risk native single writer, external=0; Save action/feedback layout only. The complete parity objective remains unchanged. Existing button dimensions, Check glyph, localized labels, click handler, native focus/activation, errors, persistence and dependencies remain unchanged.

- Place the native command and existing success text together in an unframed, full-width wrapping row: 32px minimum height, centered cross-axis alignment, 12px horizontal gap and 8px wrap gap. The button keeps its content width and fixed 32px height. Feedback retains the interface font and success color, uses 1.5 line height, and can wrap naturally at narrow widths or 20px text instead of forcing horizontal overflow. A wrapped success state can still increase the row height.
- This is a native feedback-layout adaptation, not a claim that the reference settings page has the same global Save action. No speculative icon replacement, fixed label width, new state machine or toast implementation was introduced.
- Added two deterministic builder tests for row wrapping/unframed geometry and localized feedback typography in both themes at 12/14/20px. They do not establish rendered text measurement, actual success/error activation or pixel parity.
- Verification is **PARTIAL**, not a test/build pass. Focused Rust 2021 rustfmt and `git diff --check` passed. The offline desktop test command was interrupted after approximately ten minutes stalled in dependency metadata loading; the subsequent `CARGO_INCREMENTAL=0 cargo check -p pi-desktop --tests --offline` advanced through native dependencies but was also interrupted after exceeding the bounded verification window. Both returned 130, not a source-error or test-failure receipt. No desktop tests completed for this change, and no matching Release binary was built; the preceding 183-test result and binary hash belong to the previous table change. Confirmed that no Cargo or rustc processes remained after interruption.
- The macOS window probe explicitly returned `Screen is locked`; no final-build GUI capture or interaction replay was attempted. Rendered button/feedback alignment, narrow wrapping, maximum-font layout, hover/focus and actual save feedback remain unverified. Full visual acceptance remains **PARTIAL**. No commit, push or deployment was performed.
