# ADR 0020: CSS Variable and Tailwind Theme Architecture

## Status
Accepted

## Context
As a highly customizable workspace, Life OS must offer users seamless transitions between multiple styling profiles, including **Light mode** (for daylight productivity), **Dark mode** (for low-strain night tracking), and **High Contrast mode** (complying with strict WCAG 2.1 accessibility specifications for visual clarity). Compiling multiple CSS sheets or importing distinct theme-specific component bundles increases build payload sizes and causes noticeable layout flickering during theme transitions.

## Decision
We select a theme design combining **Tailwind CSS classes** and **CSS custom properties (CSS variables)** mapped inside a global stylesheet `src/index.css`.
- **CSS Variables**: Core colors, background shades, borders, and margins are declared as CSS custom variables within root selection layers (e.g., `--background`, `--foreground`).
- **Tailwind Integration**: Tailwind is configured to map custom styles to these custom properties (e.g., `theme: { extend: { colors: { background: 'var(--background)' } } }`).
- **Trigger**: Switching themes simply requires adding or removing utility classes (`.dark`, `.high-contrast`) from the main HTML body, handled globally by a React `ThemeContext.Provider`.

## Rationale
- **Instantaneous Toggles**: Because CSS custom variables are updated at runtime directly by the webview's style engine, changing classes on the body immediately recalculates all child Tailwind classes without requiring React reconciliation sweeps or bundle reloads.
- **Minimal Asset Footprint**: All themes are packaged within a single, highly compressed styling asset, keeping Tauri compilation bundles under $10MB$.
- **Accessibility Safeguard**: Explicitly mapping `--border` and `--foreground` parameters in a `.high-contrast` class guarantees WCAG 2.1 AA compliant ratios ($>7:1$) on every system-designed layout.

## Consequences
- **Positive**: Zero rendering lag, instantaneous programmatic adjustments, easy customization, and compliance with the WCAG 2.1 accessibility criteria.
- **Negative**: Requires developers to strictly use variable-linked Tailwind utility classes (e.g., `bg-background` or `text-foreground`) instead of hardcoded hex values (`bg-white` or `text-slate-800`).
