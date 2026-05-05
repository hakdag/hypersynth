---
name: HyperSynth
colors:
  surface: '#11121e'
  surface-dim: '#11121e'
  surface-bright: '#383846'
  surface-container-lowest: '#0c0d19'
  surface-container-low: '#1a1b27'
  surface-container: '#1e1f2b'
  surface-container-high: '#282936'
  surface-container-highest: '#333441'
  on-surface: '#e2e1f2'
  on-surface-variant: '#c7c4d8'
  inverse-surface: '#e2e1f2'
  inverse-on-surface: '#2f2f3c'
  outline: '#908fa1'
  outline-variant: '#464555'
  surface-tint: '#c1c1ff'
  primary: '#c1c1ff'
  on-primary: '#1600a8'
  primary-container: '#5e5df6'
  on-primary-container: '#fbf7ff'
  inverse-primary: '#4946e1'
  secondary: '#c8bfff'
  on-secondary: '#2f2274'
  secondary-container: '#463a8b'
  on-secondary-container: '#b6abff'
  tertiary: '#bec5e5'
  on-tertiary: '#282f48'
  tertiary-container: '#6a718e'
  on-tertiary-container: '#f9f8ff'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#e2dfff'
  primary-fixed-dim: '#c1c1ff'
  on-primary-fixed: '#0a006b'
  on-primary-fixed-variant: '#2e26ca'
  secondary-fixed: '#e5deff'
  secondary-fixed-dim: '#c8bfff'
  on-secondary-fixed: '#19045f'
  on-secondary-fixed-variant: '#463a8b'
  tertiary-fixed: '#dbe1ff'
  tertiary-fixed-dim: '#bec5e5'
  on-tertiary-fixed: '#131a32'
  on-tertiary-fixed-variant: '#3e4660'
  background: '#11121e'
  on-background: '#e2e1f2'
  surface-variant: '#333441'
typography:
  display:
    fontFamily: Manrope
    fontSize: 48px
    fontWeight: '700'
    lineHeight: '1.1'
    letterSpacing: -0.02em
  h1:
    fontFamily: Manrope
    fontSize: 32px
    fontWeight: '600'
    lineHeight: '1.2'
    letterSpacing: -0.01em
  h2:
    fontFamily: Manrope
    fontSize: 24px
    fontWeight: '600'
    lineHeight: '1.3'
  body-lg:
    fontFamily: Manrope
    fontSize: 18px
    fontWeight: '400'
    lineHeight: '1.6'
  body-md:
    fontFamily: Manrope
    fontSize: 16px
    fontWeight: '400'
    lineHeight: '1.5'
  body-sm:
    fontFamily: Manrope
    fontSize: 14px
    fontWeight: '400'
    lineHeight: '1.5'
  label-caps:
    fontFamily: Manrope
    fontSize: 12px
    fontWeight: '600'
    lineHeight: '1.2'
    letterSpacing: 0.05em
  button:
    fontFamily: Manrope
    fontSize: 14px
    fontWeight: '600'
    lineHeight: '1'
rounded:
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.75rem
  lg: 1rem
  xl: 1.5rem
  full: 9999px
spacing:
  unit: 4px
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 48px
  panel-margin: 20px
  container-padding: 32px
---

## Brand & Style

This design system is built for focus and high-performance project management. It prioritizes a "calm-tech" philosophy, using a dark, immersive environment to reduce cognitive load and eye strain during long working sessions.

The aesthetic blends **Minimalism** with **Modern Corporate** sensibilities, infused with a subtle "Synthwave" undertone through its deep violet and indigo accents. It utilizes a "Stage and Shell" layout: the shell (sidebar and background) is deeply anchored in dark tones, while the workspace (content area) is treated as an elevated, floating canvas. The emotional response is one of controlled power, precision, and executive clarity.

## Colors

The palette is rooted in a refined cool slate base, moving through a sophisticated range of violets and indigos that provide structural depth and high-tech clarity.

- **Primary:** Vivid Indigo (#5e5df6) is the signature "energy" color. It is used for high-intent actions, progress indicators, and active states, providing a punchy, modern contrast against the dark backdrop.
- **Secondary:** Deep Violet (#372b7c) serves as the primary functional accent, used for secondary actions and structural elements that require a serious, stable presence.
- **Tertiary:** Lavender Mist (#c3caea) is used for low-contrast backgrounds, disabled states with high legibility, or subtle highlights.
- **Surface Tiers:** The neutral foundation uses a Muted Slate (#6a6a79) to derive its surface tiers. Floating content panels utilize tiered dark surfaces to create a subtle but distinct visual separation.

## Typography

This design system utilizes **Manrope** for its modern, geometric quality and open counters, which ensure high legibility in dark mode. The typeface gives the system a progressive, tech-forward character.

Hierarchy is established through weight and color rather than drastic size changes. Headings use semi-bold weights with tighter letter-spacing to feel grounded and authoritative. Labels and metadata use the Tertiary Lavender Mist or Secondary Deep Violet in muted opacities to recede, while primary content remains white or off-white. All-caps labels with slight tracking are reserved for section headers in the sidebar and small utility tags to maintain a systematic feel.

## Layout & Spacing

The layout follows a **Fluid Canvas** model within a fixed sidebar shell. 

1. **The Shell:** A fixed-width dark sidebar (260px) integrated into the left edge.
2. **The Panel:** The main content area is a "Floating Panel" that sits 20px away from the top, bottom, and right edges of the browser window. This panel has a permanent margin from the sidebar to emphasize its "floated" nature.
3. **Internal Spacing:** Inside the floating panel, generous padding (32px) and a consistent 8px grid ensure that complex project data (Gantt charts, Kanban boards) has room to breathe. Components should be spaced with a preference for `lg` (24px) gaps to maintain the minimal, airy feel.

## Elevation & Depth

Depth is achieved through **Tonal Layering** and **Ambient Shadows** rather than traditional heavy gradients.

- **Level 0 (Floor):** Muted Slate (#6a6a79) - The base foundation that informs the darkest background tones of the shell and sidebar.
- **Level 1 (The Panel):** A slightly lifted surface that creates a primary work area. It features a subtle 1px border using Secondary Deep Violet (#372b7c) at low opacity and a soft, wide-spread shadow (0px 10px 30px rgba(0,0,0,0.5)).
- **Level 2 (Interactive):** Surfaces within the panel use a mix of tonal variations and Deep Violet (#372b7c) for cards and input fields.
- **Level 3 (Popovers/Modals):** Surfaces utilizing Tertiary Lavender Mist (#c3caea) for high-contrast visibility or distinct backdrop blurs (12px) to focus the user's attention.

## Shapes

The shape language is sophisticated and approachable. The main content panel uses a significant corner radius (16px) to reinforce the "floating" metaphor. Internal components like cards, buttons, and input fields follow a standard 8px-12px radius. This softens the technical nature of a project management tool, making it feel more like a productivity-focused "environment" rather than a rigid database.

## Components

- **Buttons:** Primary buttons are solid Indigo (#5e5df6) with white text. Secondary buttons utilize Deep Violet (#372b7c) or "Ghost" styles—Deep Violet borders with transparent backgrounds, becoming solid on hover.
- **Cards:** Task cards in Kanban views use a subtle background with a very subtle 1px top-highlight border in Deep Violet. This adds a "lithic" quality to the cards.
- **Input Fields:** Fields are dark, recessed rectangles with 1px borders in Deep Violet. On focus, the border glows Indigo.
- **Sidebar Nav:** Active items use a "cut-out" effect or a subtle left-accent pill in Indigo. Text for inactive items is muted using the Tertiary Lavender Mist (#c3caea) at reduced opacity.
- **Progress Bars:** Use a thick (8px) track with a soft Indigo glow on the filled portion to simulate a high-tech "light-pipe" effect.
- **Chips:** Small, rounded-full badges for status (e.g., "In Progress", "Done"). Use low-opacity Deep Violet or Lavender Mist backgrounds with centered text to avoid visual clutter.