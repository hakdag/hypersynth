---
name: Deep Intellect
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
  on-surface-variant: '#444651'
  inverse-surface: '#2d3133'
  inverse-on-surface: '#eff1f3'
  outline: '#757682'
  outline-variant: '#c5c5d3'
  surface-tint: '#4059aa'
  primary: '#00236f'
  on-primary: '#ffffff'
  primary-container: '#1e3a8a'
  on-primary-container: '#90a8ff'
  inverse-primary: '#b6c4ff'
  secondary: '#006a61'
  on-secondary: '#ffffff'
  secondary-container: '#86f2e4'
  on-secondary-container: '#006f66'
  tertiary: '#0d0097'
  on-tertiary: '#ffffff'
  tertiary-container: '#2724b8'
  on-tertiary-container: '#a1a4ff'
  error: '#ba1a1a'
  on-error: '#ffffff'
  error-container: '#ffdad6'
  on-error-container: '#93000a'
  primary-fixed: '#dce1ff'
  primary-fixed-dim: '#b6c4ff'
  on-primary-fixed: '#00164e'
  on-primary-fixed-variant: '#264191'
  secondary-fixed: '#89f5e7'
  secondary-fixed-dim: '#6bd8cb'
  on-secondary-fixed: '#00201d'
  on-secondary-fixed-variant: '#005049'
  tertiary-fixed: '#e1e0ff'
  tertiary-fixed-dim: '#c0c1ff'
  on-tertiary-fixed: '#07006c'
  on-tertiary-fixed-variant: '#2f2ebe'
  background: '#f7f9fb'
  on-background: '#191c1e'
  surface-variant: '#e0e3e5'
typography:
  h1:
    fontFamily: Manrope
    fontSize: 36px
    fontWeight: '700'
    lineHeight: '1.2'
    letterSpacing: -0.02em
  h2:
    fontFamily: Manrope
    fontSize: 24px
    fontWeight: '600'
    lineHeight: '1.3'
    letterSpacing: -0.01em
  h3:
    fontFamily: Manrope
    fontSize: 20px
    fontWeight: '600'
    lineHeight: '1.4'
    letterSpacing: '0'
  body-lg:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '400'
    lineHeight: '1.6'
    letterSpacing: '0'
  body-md:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: '1.5'
    letterSpacing: '0'
  body-sm:
    fontFamily: Inter
    fontSize: 13px
    fontWeight: '400'
    lineHeight: '1.5'
    letterSpacing: '0'
  label-md:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '600'
    lineHeight: '1'
    letterSpacing: 0.05em
  code:
    fontFamily: Inter
    fontSize: 13px
    fontWeight: '400'
    lineHeight: '1.5'
    letterSpacing: '0'
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
  xl: 48px
  container-max: 1440px
  gutter: 20px
---

## Brand & Style

The design system is engineered for high-performance project management where data density and cognitive clarity are paramount. The brand personality is rooted in reliability and systemic intelligence, catering to professional teams who require a "calm" interface to manage complex workflows.

The visual style follows a **Corporate / Modern** aesthetic. It prioritizes functional minimalism, utilizing generous white space and a structured information hierarchy to reduce decision fatigue. By leaning into a "SaaS-native" look, the design system ensures immediate familiarity for power users while maintaining a premium, polished feel through precise alignment and subtle refinements.

## Colors

The palette is anchored by a trustworthy **Deep Indigo** primary, chosen for its professional gravitas and excellent contrast ratios. A **Teal** accent is utilized specifically for growth, success states, and progress indicators, providing a refreshing counterpoint to the primary blue.

Backgrounds utilize a tiered gray scale to separate global navigation from content canvas areas. Surfaces are predominantly white to maximize readability, while borders use a subtle cool gray to define structure without creating visual noise. Status-based colors (Error, Warning, Info) should follow standard accessibility patterns but remain desaturated to fit the professional tone.

## Typography

This design system employs a dual-font strategy. **Manrope** is used for headlines to provide a modern, slightly geometric character that feels refined and contemporary. **Inter** is the workhorse for all body text, data tables, and UI controls, selected for its exceptional legibility at small sizes and its neutral, systematic appearance.

Hierarchy is established through weight and color rather than excessive scale. Headers use the primary text color (Deep Slate), while secondary information and labels use a softer gray-blue to de-emphasize meta-information.

## Layout & Spacing

The layout philosophy follows a **Fixed-Fluid Hybrid Grid**. Main dashboards use a 12-column grid system with a maximum container width of 1440px, centering the content on ultra-wide displays to maintain focus. 

A strict 4px baseline grid ensures vertical rhythm. Spacing between major sections (cards, modules) defaults to 24px (lg), while internal padding for elements like input fields and list items defaults to 12px or 16px to maintain a dense but breathable data environment.

## Elevation & Depth

Visual hierarchy is achieved through **Tonal Layers** and **Ambient Shadows**. This design system avoids heavy drop shadows, instead using "Soft Depth" — a combination of 1px borders in a slightly darker neutral shade and a very diffused, low-opacity shadow (e.g., 10% opacity with a 15px blur) for elevated elements like modals or active cards.

Standard interface cards should appear flat against the background, defined primarily by their borders. Elevation is reserved for interactive states (hover) and temporary overlays (dropdowns, tooltips), ensuring the main workspace feels grounded and stable.

## Shapes

The design system utilizes a **Soft** shape language. A 0.25rem (4px) radius is the standard for most UI components (buttons, inputs, checkboxes), providing a professional and precise appearance that maximizes screen real estate. 

Larger containers like cards or panels may use the `rounded-lg` (8px) token to subtly soften the overall UI without veering into a playful or "consumer" aesthetic. This balance maintains the system's "Productive" and "Reliable" vibe.

## Components

### Buttons
Buttons are defined by high-contrast fills for primary actions (Deep Indigo) and "Ghost" or "Outline" styles for secondary actions. Use a consistent 12px horizontal and 8px vertical padding for standard sizes.

### Data Tables
Tables are the heart of the system. They feature a "no-border" vertical style, using only subtle horizontal dividers. Header rows are slightly darker with uppercase labels for clear distinction. Row hovering should trigger a subtle background tint change to #F1F5F9.

### Cards
Cards are used to group related project data. They utilize a white background, a 1px border (#E2E8F0), and no shadow in their default state. Header areas within cards should be separated by a thin hairline stroke.

### Form Inputs
Input fields use a 1px border with a soft blue focus ring. Labels are always positioned above the input for clarity. Placeholder text should be significantly lighter than user-entered text to avoid confusion.

### Chips & Badges
Small, rounded indicators used for status (e.g., "In Progress," "Done"). These use the Teal accent for positive progress and a neutral light gray for pending items, employing low-saturation background tints with high-saturation text for readability.