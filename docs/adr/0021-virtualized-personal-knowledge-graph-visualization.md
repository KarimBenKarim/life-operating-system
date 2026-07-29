# ADR 0021: Virtualized Personal Knowledge Graph Visualization

## Status
Accepted

## Context
One of Life OS's core tenets is "Everything Connects" — modeling a user's life as a deeply connected personal knowledge network. At scale over decades, a user's vault can grow to contain over $5,000$ notes, daily journals, and task files, each connected by numerous backlink relationships.
Renders using standard SVG-based force-directed layouts or unvirtualized scrolling grids suffer from severe rendering bottlenecks:
1. **SVG DOM Bloat**: Attempting to render thousands of active nodes and edges as distinct SVG path elements results in DOM overhead, freezing client browsers and webviews.
2. **List Rendering Lag**: Standard vertical list scrolling of thousands of detailed ledger transactions or biometric logs causes paint bottlenecks.

## Decision
We select a two-pronged high-performance rendering strategy for complex visual elements:
1. **WebGL for Graph Rendering**: The force-directed personal knowledge graph visualization component must utilize **WebGL canvas rendering** (via engines like `3d-force-graph` or custom HTML5 WebGL layers) rather than standard SVG.
2. **List Virtualization (Windowing)**: All large chronological streams, transaction ledgers, biometric logs, and audit logs must use **virtualized rendering** (using libraries like `react-window` or `react-virtualized`). Only elements in the visible viewport (plus a small buffer) are drawn in the DOM, with absolute absolute layout positions mimicking the total list height.

## Rationale
- **60 FPS Performance**: WebGL moves graph calculation vectors to the host machine's physical GPU, allowing Life OS to render thousands of linked document nodes smoothly at 60 FPS.
- **Constant Memory Footprint**: List virtualization keeps the total number of DOM elements rendered on the screen constant (e.g., restricted to around 50 elements) regardless of whether the log database contains 100 or 100,000 entries, saving precious memory cycles.

## Consequences
- **Positive**: Sub-millisecond interface responsiveness, buttery-smooth graph traversal animations, and total stability for massive decades-long life logging.
- **Negative**: Adds component setup overhead and requires developers to pass explicit cell heights/widths to virtualized list templates.
