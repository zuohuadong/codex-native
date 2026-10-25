# Build 21477 first dynamic baseline

Status: **PARTIAL**. Baseline migration authorized on 2026-10-24. Installed identity is `26.1016.62342 / 21477`, verified against `contract.json`; no claim is made that this is the newest public release.

| Frame | Measured points | Scale | Evidence |
| --- | --- | --- | --- |
| Reference settings, light, zh-CN | 1180 × 760 | 2 | [PNG](runs/21477/settings-light-zh-cn-wide/capture.png) |
| Reference settings, light, zh-CN | 600 × 620 | 2 | [PNG](runs/21477/settings-light-zh-cn-request600/capture.png) |
| Reference settings, light, zh-CN | 480 × 600 | 2 | [PNG](runs/21477/settings-light-zh-cn-request480/capture.png) |
| Native settings, light, zh-CN | 1180 × 760 | 2 | [PNG](runs/21477/volt-settings-wide-active/capture.png) |

[Interactive comparison](runs/21477/compare-settings-wide-verified/index.html) · [JSON metrics](runs/21477/compare-settings-wide-verified/report.json) · [Capture matrix](matrix-21477.json)

The comparison reports **FAIL_PIXELS**, with 1,343,069 differing pixels out of 3,587,200 (37.4406%). This is a full-frame diagnostic: it includes different settings content, layout and window chrome. It is not “62.56% restored”. Focus/IME parity and interaction semantics need separate evidence.

Observed structural differences: the reference has a global navigation rail, a settings-category column, and a grouped general-settings pane. Native currently retains the conversation sidebar and a single preferences form. Width 480 is accepted by the reference, but height 480 is clamped to 600; the category sidebar remains visible and general settings become very narrow. These observations update capture targets without silently changing native functionality.

The initial native screenshot did not enter settings. Its sidecar is REJECTED and its report INVALIDATED. After activating the owned test window and repeating the shortcut, the settings page was visually confirmed and compared. No result is inferred solely from sending a shortcut.

The 144-row matrix is a capture plan, not a set of 144 proven reference states. Missing states, dark theme, English locale, paired narrow native captures, backend transitions, and action replays remain outstanding. State names inherited from the earlier plan still need mapping to observed 21477 pages.

Images and interactive reports remain in Git-ignored `runs/` because live application windows may contain local account or workspace information. Their hashes and measured geometry are in capture sidecars. Links require this local checkout. No provider request was sent; the native capture process used isolated state and was stopped afterward.
