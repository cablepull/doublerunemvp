# Double Rune MVP

An Anomaly Rune inbox UI **built and piloted by ruvnet Rune**. Both compile in one Cargo
workspace and share one lockfile. The ruvnet CLI has a new `rune ui` command that observes
the actual element tree, checks a target, sends a UI tap, and verifies the resulting tree.
It never calls the inbox's state functions directly.

## Run the proof (Linux)

```sh
git clone --recurse-submodules https://github.com/cablepull/doublerunemvp.git
cd doublerunemvp
git checkout foundation/ui-pilot-mvp
git submodule update --init
# Debian/Ubuntu build and software-rendering prerequisites:
sudo apt-get install -y build-essential pkg-config libxkbcommon-dev libwayland-dev mesa-vulkan-drivers
./scripts/prove.sh
```

Rustup installs the pinned compiler from `rust-toolchain.toml`. The first fetch is online;
the controller's approved application build is locked and offline. No model API key,
Google account, or paid service is needed. This is a **mock inbox**, not a Gmail connection.

```sh
# Open the actual native application:
./target/rune/release/rune-inbox
# Or let ruvnet Rune pilot it headlessly:
./target/rune/release/rune ui --scenario fixtures/archive.json
```

The proof produces screenshots, element trees, replay scripts, process logs, and hashed
receipts under `.rune/ui/`. `.rune/proof-summary.json` describes all acceptance checks.

## What is proven

1. One workspace compiles the real UI dependency and the extended real ruvnet toolchain.
2. `rune build --freeze` produces an offline build receipt for `rune-inbox`.
3. `rune ui` checks source, lockfile, compiler and executable against that approved build.
4. Actual input selects message 2, marks it read and archives it; unrelated messages survive.
5. A second scenario resets the fixtures through a UI button.
6. Wrong postconditions, nonexistent controls, modified binaries and stale source are rejected.

The pilot is deterministic and offline. Each step launches the approved application and
replays the accumulated input prefix, then inspects a fresh GPU-rendered screenshot and
element tree. It is **not a persistent live session, a natural-language planner, or an LLM
optimization demonstration**. Those can use this action/observation boundary later.

## Foundation

- `crates/rune-inbox`: real Anomaly Rune UI, mock data and UI-handler tests.
- `crates/rune-toolchain-ui`: bounded controller adapter and evidence schema.
- `crates/rune-toolchain/src/commands/ui.rs`: `rune ui` integration into ruvnet's CLI.
- Other `rune-toolchain-*` crates: vendored ruvnet source, preserving its MIT license.
- `vendor/anomaly-rune`: Anomaly Rune pinned as a git submodule, including required patches.
- `fixtures/`: explicit actions and independent per-step postconditions.
- `scripts/prove.sh`: reproducible build, positive and negative acceptance proof.
- [Executed proof](docs/proof/README.md), [architecture and deployment](docs/FOUNDATION.md), [upstream provenance](docs/UPSTREAMS.md).

The new UI adapter does not change ruvnet's statistical optimization evaluator or weaken
its performance gates. UI correctness has its own acceptance command. No performance
improvement is claimed by this MVP.

Both upstreams are evolving. Keep the submodule revision, Rust version and Cargo.lock
pinned. A source build is evidence for the machine that ran it, not proof of every target.
