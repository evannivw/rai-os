# Native interface directions for the final demo

Status: Column selected; compact spacing and color proposals under review.
Not yet implemented as guest screens. November 3, 2026.
The user requested a black, terminal-inspired phone interface with sparse text,
keyboard-like linework and recognizable applications. Keep native QEMU rendering
as the product target; conversational interactive mockups only explore layouts.
Example app names and rows in those mockups are illustrative, not new live evidence.

## Initial proposal: Tactile Terminal (superseded by feedback)

A quiet home screen with three application launchers, simple bracket monograms,
a persistent prompt entry and one recent application. Use off-white primary text,
muted readable secondary text, one pale green accent and fine structural lines.
The prompt is the strongest action; applications remain easy to open without AI.
Corners and brackets provide identity without a wall of decorative ASCII text.
The initial proposal was based on the user's mobile/terminal direction and the current
three-app runtime capacity. Monograms derive from app metadata, not hardcoded
application identities. Color, typography and hit targets must be translated to
the actual framebuffer renderer and verified at its supported resolution.

## Alternative: Workbench

Give the last opened application the main area, with a clear resume action.
Other applications appear as compact rows; creation stays near the bottom.
This emphasizes continuing useful work and is well suited to a workshop workflow
presentation. Last-opened state would need implementation and persistence rules;
its placement in the mockup does not establish that the guest already tracks it.

## Alternative: Canvas

Give the prompt the main area with a short editorial headline and a compact
application list below it. This makes creation the opening visual, but provides
less immediate emphasis on everyday application use. Avoid expanding it into a
chat transcript that displaces the installed applications.

## Polish sequence after choosing a direction

1. Apply one visual system to home, prompt, review, app and restore. Keep text
   contrast, touch/keyboard focus, labels and navigation consistent.
2. Make the existing generation stages legible: request sent, proposal received,
   validation and review. Show actual state and elapsed waiting; do not invent a
   percentage or claim that generated code is compiled to native machine code.
3. Simplify review around what changes, what data stays, and the action to confirm.
   Keep permissions and errors accessible without filling every screen with logs.
4. Rehearse one continuous create/use/evolve/restore journey, checking persisted
   data and retained history. Current separate live evidence does not establish
   a newly recorded end-to-end journey; verify the exact recording sequence.
5. Record a three-minute video from real QEMU, label inference waits/time cuts,
   and finish clean-machine instructions, public links and submission checks.

Do not add new kernel mechanisms for appearance alone. Native redesign is pending
selection/implementation. No new model calls, OS behavior or performance claims
are introduced by this design document.

## Home-screen correction after user review

The user found the initial home crowded and rejected putting the creation prompt
on the landing screen. Home must prioritize finding and opening applications.
Move prompt entry to a separate Create destination. Remove the marketing headline,
recent-activity block and explanatory copy from Home. Keep a compact, predictable
Home / Apps / Create navigation; Apps holds the searchable library.

Three revised conversational concepts explore the same navigation:

- Column (recommended for review): icon/name pairs connected by a fine vertical
  line. All three current apps are visible without an extra navigation step.
- Focus: one large app launcher at a time with previous/next controls. More distinct
  visually, but opening another app can require additional interactions.
- Shelf: three compact icon/name launchers above one horizontal rule. Familiar,
  sparse and easy to scan, with less departure from existing launchers.

These are unselected design proposals with illustrative app metadata. They do not
increase the current runtime capacity or implement a new guest launcher. Keep
application identity dynamic when implementing the chosen layout. The preview
script passed syntax validation; rendered browser verification was not performed.

## Selected Column layout: compact spacing and cooler accents

The user approved Column and requested smaller icons, less side padding and no
yellow/lime accent, while keeping black. The next conversational preview uses
40 px icons (previously 58), 12 px side insets (previously 24 plus an extra 12 for
the list), and 60 px app rows. The whole row remains a generous interaction target.
Applications start toward the top rather than occupying a centered decorative
block. Create remains separate. Pixel values describe the browser concept and
must be adapted to the actual guest resolution.

Four color options share a near-black #090b0f background, #e7ebf1 primary text,
#9fa9b8 secondary text and #303743 structural lines:

- Glacier blue: #87bfff (recommended for review).
- Ice: #dce5f2.
- Cool mint: #79d8c1.
- Lavender: #b8a6f4.

Keep accents on launcher glyphs, focus and selected navigation. Do not fill every
row with color. No palette is selected yet. Preview examples still contain three
apps; denser layout does not increase runtime app capacity.
