---
# 归档：v1 不实现亮色主题，仅作参考。Canonical 见 keyvault/DESIGN.md
name: Secure Utility Light
colors:
  surface: '#f7f9fb'
  surface-dim: '#d8dadc'
  surface-bright: '#f7f9fb'
  surface-container-lowest: '#ffffff'
  surface-container-low: '#f2f4f6'
  surface-container: '#eceef0'
  surface-container-high: '#e6e8ea'
  surface-container-highest: '#e0e3e5'
  on-surface: '#191c1e'
  on-surface-variant: '#434655'
  inverse-surface: '#2d3133'
  inverse-on-surface: '#eff1f3'
  outline: '#737686'
  outline-variant: '#c3c6d7'
  surface-tint: '#0053db'
  primary: '#004ac6'
  on-primary: '#ffffff'
  primary-container: '#2563eb'
  on-primary-container: '#eeefff'
  inverse-primary: '#b4c5ff'
  secondary: '#505f76'
  on-secondary: '#ffffff'
  secondary-container: '#d0e1fb'
  on-secondary-container: '#54647a'
  tertiary: '#943700'
  on-tertiary: '#ffffff'
  tertiary-container: '#bc4800'
  on-tertiary-container: '#ffede6'
  error: '#ba1a1a'
  on-error: '#ffffff'
  error-container: '#ffdad6'
  on-error-container: '#93000a'
  primary-fixed: '#dbe1ff'
  primary-fixed-dim: '#b4c5ff'
  on-primary-fixed: '#00174b'
  on-primary-fixed-variant: '#003ea8'
  secondary-fixed: '#d3e4fe'
  secondary-fixed-dim: '#b7c8e1'
  on-secondary-fixed: '#0b1c30'
  on-secondary-fixed-variant: '#38485d'
  tertiary-fixed: '#ffdbcd'
  tertiary-fixed-dim: '#ffb596'
  on-tertiary-fixed: '#360f00'
  on-tertiary-fixed-variant: '#7d2d00'
  background: '#f7f9fb'
  on-background: '#191c1e'
  surface-variant: '#e0e3e5'
typography:
  headline-lg:
    fontFamily: Inter
    fontSize: 30px
    fontWeight: '600'
    lineHeight: 38px
    letterSpacing: -0.02em
  headline-md:
    fontFamily: Inter
    fontSize: 24px
    fontWeight: '600'
    lineHeight: 32px
    letterSpacing: -0.01em
  body-lg:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  body-md:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  label-md:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: '500'
    lineHeight: 16px
  code-sm:
    fontFamily: JetBrains Mono
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 18px
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  base: 8px
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 32px
  gutter: 16px
  margin: 24px
---

## Brand & Style

The design system is engineered for a high-security environment where clarity, precision, and efficiency are paramount. The personality is professional and understated, evoking the feeling of a well-organized physical vault. It draws heavily from **Corporate Modern** and **Minimalist** influences, prioritizing information density without sacrificing legibility.

The target audience consists of developers and security engineers who require a "tool-first" interface. The aesthetic is inspired by high-end engineering software, utilizing a "light-wash" palette that reduces eye strain while maintaining a crisp, authoritative presence. The emotional response should be one of calm confidence—users should feel that their data is structured, safe, and easily accessible.

## Colors

This color palette focuses on a "Paper & Ink" philosophy. The base layer is a clean, bright white or extremely light gray to establish a neutral canvas. 

- **Primary:** A focused, professional blue (#2563EB) reserved for action-oriented elements and critical feedback loops.
- **Backgrounds:** The main application background uses #F8FAFC to prevent the harshness of pure white, while nested containers use #F1F5F9 to create subtle logical separation.
- **Typography:** Contrast is maintained through a deep graphite hierarchy. Primary text uses #0F172A for maximum readability, while metadata and labels use #334155.
- **Borders:** A consistent #E2E8F0 is used for hair-line strokes (1px) to define boundaries without adding visual weight.

## Typography

The typography strategy employs a dual-font approach to distinguish between navigation/interface elements and technical data.

- **Inter (Interface Sans):** Used for all UI controls, headers, and body text. It provides a modern, neutral foundation that handles varied weights exceptionally well.
- **JetBrains Mono (Technical Mono):** Specifically used for keys, tokens, code snippets, and labels. This monospaced font ensures that every character (such as '0' vs 'O' or '1' vs 'l') is distinct, which is critical for security operations.

Hierarchy is established through weight shifts (SemiBold for headers) rather than just size, maintaining a compact footprint suitable for data-heavy dashboards.

## Layout & Spacing

The layout follows a strict **Fixed Grid** model for desktop to ensure data tables and vault views remain predictable and structured. 

- **Grid:** A 12-column system with 16px gutters and 24px outer margins.
- **Rhythm:** An 8px base unit governs all padding and margin decisions, ensuring vertical and horizontal rhythm.
- **Responsive Behavior:** 
    - **Desktop:** Fixed maximum content width of 1440px.
    - **Tablet:** Fluid width with reduced margins (16px).
    - **Mobile:** Single column layout; typography scales down (e.g., Headline-LG becomes 24px) and margins shrink to 12px.

## Elevation & Depth

In this design system, depth is communicated through "Layered Flatness." Instead of heavy shadows or dramatic gradients, hierarchy is achieved through:

1.  **Stroke Definition:** All interactive containers (cards, modals) use a 1px solid border (#E2E8F0).
2.  **Subtle Shadows:** Level 1 elevation (Cards, Dropdowns) uses a `shadow-sm` (0 1px 2px 0 rgba(0, 0, 0, 0.05)). Level 2 (Modals, Context Menus) uses a slightly more pronounced but diffused shadow to separate from the background.
3.  **Tonal Offsets:** Background layers move from #F8FAFC (Base) to #FFFFFF (Component Surface) to #F1F5F9 (Input Fields/In-set areas).

## Shapes

The shape language is "Soft-Mechanical." By using **Level 1 (Soft)** roundedness, the UI avoids the aggressive sharpness of pure brutalism while remaining more serious and "engineered" than consumer-grade pill-shaped designs.

- **Small Components (Buttons, Inputs):** 0.25rem (4px) corner radius.
- **Medium Components (Cards, Modals):** 0.5rem (8px) corner radius.
- **Focus States:** 2px solid primary blue offset by a 2px white gap to maintain the "fine-line" aesthetic.

## Components

- **Buttons:** Primary buttons are solid Blue (#2563EB) with white text. Secondary buttons use a white background with #E2E8F0 border and #334155 text. Focus on 1px borders and compact padding.
- **Input Fields:** Use a subtle inset look. Background: #F1F5F9; Border: #E2E8F0. On focus, the border transitions to Primary Blue with a subtle glow.
- **Chips/Badges:** Use a "Monospace Label" style. Small text (JetBrains Mono), light gray background (#F1F5F9), and no borders.
- **Cards:** Pure white (#FFFFFF) background with a 1px #E2E8F0 border and the `shadow-sm` defined in the Elevation section.
- **Vault Lists:** Table rows should have a hover state of #F8FAFC and a subtle 1px bottom divider. Key strings should always use the `code-sm` typography style.
- **Checkboxes/Radios:** Small, precise 4px radius squares for checkboxes. Use Primary Blue only for the "Checked" state.