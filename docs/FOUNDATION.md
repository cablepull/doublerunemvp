# Foundation and deployment

## Control boundary

`rune build --freeze` -> approved plan + application receipt -> `rune ui` ->
verified artifact -> scripted input -> native Rune renderer -> observed tree + PNG ->
postconditions -> passed/rejected proof receipt.

The controller uses the original ruvnet `Store`, `Plan::check_fresh`, source digests,
compiler identity and offline Cargo adapter. A dedicated UI adapter extends the CLI;
the renderer remains the actual Anomaly Rune dependency. The controller does not link
the app model, and cannot bypass a button handler by mutating its state.

Every action requires a uniquely identified, visible, non-inert button with nonzero
bounds. Each observation must declare postconditions. Scenarios have a versioned schema,
deny unknown fields, are limited to 64 KiB and 16 actions, and contain no shell commands.
The child runs with a rendering-only environment allowlist and a per-observation timeout.
The renderer's PNG and tree are hashed. Source/compiler/binary freshness is checked before
the run and after the last observation. The artifact digest is checked before each launch.

This is a trusted-code development harness, not a hostile-code sandbox. It does not offer
filesystem containment, process-tree containment, screenshot tamper resistance, a secure
signing service or network enforcement. The pilot's receipt is unsigned. ruvnet's existing
signed optimization receipts are a separate facility, not a claim about UI proofs.

## Replay versus live control

The MVP deliberately uses prefix replay: the next observation replays earlier actions on
fresh fixture data, adding one action after inspecting the previous tree. This permits
step-by-step observation and fail-closed assertions without inventing a live IPC protocol.
Do not use replay for real email sends or other external side effects. A live Gmail client
will need persistent transport, idempotency keys, explicit action authorization and safe
secret storage. It must not replay a previously executed external write.

An LLM planner would propose this same restricted action schema. The controller would
still validate targets and postconditions, retain independent acceptance fixtures, and
apply budgets. This MVP does not call a model or assert that language planning works.

## Build choices

The root workspace includes the ruvnet toolchain and our app. Its default build product is
the app, avoiding selection of another CLI binary when the original adapter chooses one
workspace artifact. `cargo build --workspace` builds everything; `rune build` builds the
default application. Required Anomaly patches are applied at the root workspace.

The UI enables `tooling` explicitly in release builds. No upstream Anomaly source was
modified. The compiler and dependencies are pinned. Initial dependency acquisition is
online; the approved application build uses Cargo's locked offline mode.

## Deployment status

| Destination | State in this foundation |
| --- | --- |
| Linux x86-64 native UI and headless pilot | MVP proof target; Mesa software Vulkan can render in CI |
| macOS native UI | Upstream-supported UI; needs native build/test/signing job for this integration |
| Windows native UI | Upstream-supported UI; needs native MSVC build/test/installer job |
| Browser/WASM | Upstream experimental target; not wired into this native artifact pilot |
| iOS native | Upstream Metal embedding exists; host app/lifecycle/packaging remain separate work |
| Android native | No supported end-to-end native pipeline established here |
| ARM64 Linux | Cross-compilation is separate from native graphics and hardware validation |

The browser uses `wasm32-unknown-unknown`; ruvnet's existing WASI runner uses
`wasm32-wasip1`. They are not interchangeable. The optional WebGL feature in the current
UI manifest conflicts with the web guide's no-fallback statement; no WebGL compatibility
is claimed here. Release installers and complete web asset sets require their own manifest
and verification; a selected executable hash does not cover them.
