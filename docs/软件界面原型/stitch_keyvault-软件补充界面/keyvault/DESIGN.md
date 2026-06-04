---
name: KeyVault
colors:
  surface: '#0b141c'
  surface-dim: '#0b141c'
  surface-bright: '#313a43'
  surface-container-lowest: '#060f16'
  surface-container-low: '#141c24'
  surface-container: '#182028'
  surface-container-high: '#222b33'
  surface-container-highest: '#2d363e'
  on-surface: '#dae3ee'
  on-surface-variant: '#c0c7d4'
  inverse-surface: '#dae3ee'
  inverse-on-surface: '#29313a'
  outline: '#8b919d'
  outline-variant: '#414752'
  surface-tint: '#a2c9ff'
  primary: '#a2c9ff'
  on-primary: '#00315c'
  primary-container: '#58a6ff'
  on-primary-container: '#003a6b'
  inverse-primary: '#0060aa'
  secondary: '#67df70'
  on-secondary: '#00390d'
  secondary-container: '#27a640'
  on-secondary-container: '#00320a'
  tertiary: '#ffba42'
  on-tertiary: '#432c00'
  tertiary-container: '#da9600'
  on-tertiary-container: '#4f3400'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#d3e4ff'
  primary-fixed-dim: '#a2c9ff'
  on-primary-fixed: '#001c38'
  on-primary-fixed-variant: '#004882'
  secondary-fixed: '#83fc89'
  secondary-fixed-dim: '#67df70'
  on-secondary-fixed: '#002105'
  on-secondary-fixed-variant: '#005317'
  tertiary-fixed: '#ffddaf'
  tertiary-fixed-dim: '#ffba42'
  on-tertiary-fixed: '#281800'
  on-tertiary-fixed-variant: '#614000'
  background: '#0b141c'
  on-background: '#dae3ee'
  surface-variant: '#2d363e'
typography:
  headline-lg:
    fontFamily: Inter
    fontSize: 24px
    fontWeight: '600'
    lineHeight: 32px
    letterSpacing: -0.02em
  headline-md:
    fontFamily: Inter
    fontSize: 18px
    fontWeight: '600'
    lineHeight: 24px
    letterSpacing: -0.01em
  body-md:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  body-sm:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 16px
  code-md:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 20px
  label-caps:
    fontFamily: Inter
    fontSize: 11px
    fontWeight: '700'
    lineHeight: 16px
    letterSpacing: 0.05em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  base: 4px
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 40px
  sidebar-width: 260px
  detail-panel-width: 400px
---

## Brand & Style
The design system is engineered for a local-first password manager, prioritizing a sense of absolute security, technical precision, and high-velocity utility. The aesthetic is rooted in **Modern Minimalism** with a **Technical** edge, drawing inspiration from developer tools where information density is a feature, not a flaw.

The target audience consists of power users and security-conscious individuals who value reliability over ornamentation. The UI should evoke a "vault-like" emotional response—solid, impenetrable, and quiet. This is achieved through a monochromatic base, precise line work, and purposeful use of monospaced typography for sensitive data.

## Colors
The palette is strictly dark-mode, utilizing a layered "Obsidian" grayscale to establish hierarchy. 

- **Primary (#58A6FF):** "Security Blue" is used for primary actions, focus states, and active selection indicators.
- **Success (#3FB950):** Used for password strength indicators and successful copy/sync operations.
- **Backgrounds:** The foundation is `#0D1117`, with `#161B22` used for elevated surfaces like sidebars and cards.
- **Borders:** `#30363D` provides the primary structural definition, replacing heavy shadows to maintain a flat, technical aesthetic.

## Typography
The system employs a dual-font strategy:
1. **Inter:** Used for all interface elements, navigation, and labels. It provides a neutral, highly legible foundation that feels professional and contemporary.
2. **JetBrains Mono:** Reserved exclusively for passwords, recovery keys, and hashes. The monospaced nature ensures character clarity (e.g., distinguishing '1', 'l', and 'I'), which is critical for security contexts.

**Scale:** We prioritize smaller font sizes (12px-14px) to facilitate high information density, ensuring users can view large lists of credentials without excessive scrolling.

## Layout & Spacing
The layout follows a **Fixed-Fluid-Fixed** three-pane architecture:
1. **Navigation Sidebar (Fixed):** 260px width. Houses categories, tags, and vaults.
2. **List View (Fluid):** Expands to fill the center, optimized for rapid scanning.
3. **Detail Panel (Fixed/Contextual):** 400px width. Displays full entry details on the right.

We use a **4px base unit** for spacing. Gutters and margins are kept tight (16px) to maximize screen real estate. On mobile, the layout collapses into a single-pane view with a persistent bottom navigation bar for quick access to search and the generator.

## Elevation & Depth
In this design system, depth is communicated through **Tonal Layering** and **Low-Contrast Outlines** rather than physical shadows. 

- **Level 0 (Base):** `#0D1117` for the main application background.
- **Level 1 (Surface):** `#161B22` for sidebars and floating panels.
- **Level 2 (Overlay):** `#1C2128` for command palettes and modals.

Every interactive container must have a `1px` solid border using the `#30363D` token. This "hairline" border style creates a sharp, technical feel. Soft, diffused shadows (black, 40% opacity, 8px blur) are used sparingly only for top-level modals (like the Command Palette) to separate them from the workspace.

## Shapes
We utilize a **Soft (0.25rem)** roundedness approach. This slight rounding prevents the UI from feeling "sharp" or aggressive while maintaining a compact, efficient footprint. 

- **Standard Elements:** 4px (inputs, buttons, cards).
- **Large Elements:** 8px (modals, dropdown menus).
- **Search Bars:** 6px to differentiate from standard input fields.

## Components
### Buttons
- **Primary:** Background `#58A6FF`, text `#0D1117` (Dark-on-Light contrast for visibility).
- **Secondary:** Transparent background with `#30363D` border; text `#C9D1D9`.
- **Ghost:** No border/background until hover; used for utility actions in lists.

### Secure Inputs
Fields containing sensitive data must use the **JetBrains Mono** font. Include a persistent "visibility toggle" icon on the right. Focus states should be indicated by a 1px solid border of `#58A6FF` and a subtle blue outer glow (2px).

### Strength Meters
Horizontal progress bars for password strength. Use a 4-step segment system:
- Weak: Red
- Fair: Orange
- Good: Yellow
- Strong: `#3FB950` (Success Green)

### Lists & Tables
Credential lists should use a condensed row height (40px-48px). Use alternating background tints or a 1px bottom border to separate entries. Hover states should use a subtle highlight of `#21262D`.

### Command Palette
A centralized search overlay triggered by `Cmd+K`. It should feature a translucent background blur (backdrop-filter: blur(12px)) over a `#1C2128` surface to create a sense of focus and priority.