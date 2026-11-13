# Upstream provenance

| Component | Source revision | Treatment |
| --- | --- | --- |
| ruvnet Rune | `ced361f0249a69accce4c5133027ccdb6299420d` | MIT-licensed toolchain crates copied into root workspace; CLI extended with `ui` |
| Anomaly Rune | `6ba947e580f2d864bb333d913ba843284e57f427` | Unmodified pinned git submodule |

Sources: https://github.com/ruvnet/rune and https://github.com/anomalyco/rune

The root LICENSE is the ruvnet upstream MIT license. Anomaly's license remains in its
submodule. New adapter/application code is MIT-licensed. Upstream tests remain with the
vendored crates; the focused MVP suite avoids claiming that optional Node, cross-target,
namespace-dependent and optimization studies were all executed.

Integration modifications: root workspace/default members and Anomaly dependency patches;
`rune-toolchain` adds the `rune-ui` dependency, subcommand registration and command module;
new `rune-toolchain-ui` and `rune-inbox` crates; proof fixtures, scripts and CI. No upstream
performance, isolation or signing claims are extended to the new pilot by implication.
