# ZCode implementation reference

ZCode is the primary open source implementation reference for this continuation. The pinned checkout is `zai-org/ZCode` commit `3c5b227649371507ddc4eb46ac5133afe6db1cab`, package version `3.16.2`, licensed under Apache-2.0. The extracted manifest is [zcode-reference.json](zcode-reference.json).

The source is used for component structure, state transitions, responsive behavior, settings grouping, composer controls, focus states, and token vocabulary. Its React/Electron runtime is not copied into the Rust/GPUI application. Codex build 21477 remains historical behavior evidence only. ZCode is the sole current visual authority. The native app must remain pure GPUI.

Observed ZCode contract used in the native implementation:

- UI base size `14px`; settings desktop navigation `268px`; compact navigation `68px`.
- Settings header `48px`; regular controls `32px`; common settings radius `12px`.
- Light surface `#f8f8f8`, sidebar `#f0f0f0`, panel `#ffffff`, foreground `var(--color-neutral-800)` (approximately `#262626` in sRGB).
- Dark surface `#161616`, sidebar `#161616`, panel `#202020`, card/input `#2b2b2b`, foreground `var(--color-neutral-300)` (approximately `#d4d4d4` in sRGB).
- Selection is a low contrast neutral surface; focus uses the semantic input border token.

The native changes are intentionally limited to equivalent tokens and layout behavior. Any future copied asset or code must retain the Apache-2.0 notice and be reviewed against the project boundary.

Reference startup succeeded after installing the complete root workspace and running `ZCODE_ENV=test node scripts/build-desktop-agent-cli.mjs`. The prior desktop-only dependency filter omitted CLI workspace dependencies. A genuine dark General settings screenshot was observed, but it is not yet a matched, identity-bound visual acceptance pair.
