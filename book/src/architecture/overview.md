# Architecture overview

Pixora Studio separates the document engine from presentation and transport. The desktop UI, CLI, JSON control channel, and MCP server reach editing behavior through the command registry rather than maintaining separate implementations.

```text
desktop UI       CLI       JSON control       MCP
    |             |              |              |
    +-------------+--------------+--------------+
                          |
                   command registry
                          |
                    engine Session
                          |
       document model + operations + compositors
                          |
             PSD / raster / .pcraft I/O
```

The enforced dependency direction runs from format-independent foundation crates through the document and operation layers to composition, I/O, the engine, and finally UI/automation. `cargo xtask layers` checks this rule.

The complete design narrative and dependency diagram remain in [`docs/architecture.md`](https://github.com/storytold/photocraft/blob/main/docs/architecture.md). That document contains historical design material as well as implemented architecture; confirm behavior in current source when security or compatibility depends on it.

## Core properties

- `pixora-doc` holds the pure-data document model.
- `pixora-engine::Session` owns open documents, active state, history, and command dispatch.
- `pixora-compose` is the CPU reference compositor; `pixora-gpu` accelerates supported interactive paths.
- `pixora-io` maps between documents and PSD or flat formats.
- `pixora-format` reads and writes the native `.pcraft` bundle.
- `pixora-ui-egui` is a thin shell over engine commands.
- `pixora-automation` exposes the headless engine and the live-GUI bridge.

See [Crates and dependency layers](crates.md) for the complete workspace map.
