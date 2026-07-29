# ADR 0019: Modular Routing via Hash Router for Tauri

## Status
Accepted

## Context
When packaging the Life OS frontend for desktop platforms using Tauri, client HTML, CSS, and JS assets are bundled and served directly from the physical local host system using local file pathways (using custom URI schemes such as `tauri://localhost/index.html` or `file://`). Utilizing standard HTML5 client-side path-based routers (`BrowserRouter`, matching routes like `tauri://localhost/dashboard/calendar`) causes application crashes on browser page refreshes because the native webview is forced to search for a physical asset directory at that path on disk, which does not exist.

## Decision
We select **React Router v6** configured to use **`HashRouter`** boundaries for all client-side page routing inside Life OS client applications.
- **Path Resolution**: Route paths are represented using hashes (e.g., `tauri://localhost/index.html#/dashboard/calendar`).
- **Guard Layers**: Private route guards are encapsulated inside a custom `<ProtectedRoute>` component, validating active session tokens in Zustand before resolving child viewports.

## Rationale
- **Zero Refresh Breakages**: Since a hash-based router handles route parameters inside the hash anchor (which is parsed entirely client-side by JS), browser page refreshes or native reloads always resolve back to the base `index.html` root file, preventing runtime resource-loading crashes.
- **Cross-Platform Portability**: Using hash-based paths makes the exact same build bundle fully portable across Tauri desktop apps, mobile Capacitor packages, and standard static web hosting directories without custom server-side routing rewrites.

## Consequences
- **Positive**: Complete cross-platform compatibility, seamless page refreshes inside native desktop/mobile wrappers, and simple guard implementation.
- **Negative**: Hashes look slightly less clean in browser address bars (e.g., `#/settings` vs `/settings`), which is irrelevant for standalone native desktop applications.
