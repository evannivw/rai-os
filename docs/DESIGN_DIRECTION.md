# Native interface directions for the final demo

Status: compact Column, Tactile Terminal linework and Glacier Blue selected.
Native guest implementation completed in the late November 3 polish increment.
The sections below preserve the earlier design discussion; current details follow.
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

## Original polish sequence before implementation

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

At this proposal stage, the native redesign was pending selection/implementation.
The selected design has since been implemented; see **Implemented native polish**
below. The proposal itself introduced no model calls, OS behavior or performance
claims.

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

At this stage these were unselected proposals with illustrative app metadata;
Column was subsequently chosen and implemented. The concepts did not increase
runtime capacity or themselves implement a guest launcher. Keep
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
row with color. Glacier Blue was subsequently selected and implemented. Preview
examples still contain three apps; denser layout does not increase runtime capacity.

## Apps and Create refinement

The user requested a compact application library, improved prompt entry and more
breathing room between adjacent controls. The next interactive concept uses the
glacier-blue working accent and Column home. Apps has a 44 px search field, 34 px
glyphs and 54 px rows with six-pixel separation. Side inset is 16 px, between the
previous spacious and overly tight proposals. Section gaps are 20 px. These are
browser design values, not framebuffer coordinates.

Create groups an explicit label, multiline prompt and character counter inside one
composer. The primary Create action is separated from both the text field and the
bottom navigation. The concept retains the current 128 printable-ASCII-character
constraint. It provides a local optional keyboard with letters, shift, basic
numbers/symbols, space and backspace; input is retained when the keyboard closes.
The composer contracts while the keyboard is open. Preview submission sends no
request and explicitly reports that it is a design preview.

This is not an implemented guest keyboard. Current scripts/apps.py forwards VNC
key/pointer events to guest app.key/app.tap commands; src/app_ui.rs owns pointer
geometry and application interaction. A future guest-drawn soft keyboard can
reuse that coordinate route and translate key hit areas into existing text input.
It needs drawing, hit testing, focus/cursor management, layout changes, input limits
and real QEMU checks. It does not itself require introducing a physical phone
touchscreen driver. Removing the host input adapter or supporting actual phones
requires separate platform-specific input/display work.

Preview JavaScript syntax was checked; rendered-browser and native-QEMU verification
are not claimed for these design-only changes.


## Implemented native polish

The user selected Column, Tactile Terminal and Glacier Blue. The actual 480x800
framebuffer now uses near-black #090b0f, readable muted text, glacier #87bfff,
40-pixel monograms and 80-pixel launcher rows with a 16-pixel outer inset. All
three installed apps fit in a connected column. Metadata shows record counts;
app identities remain dynamic. Apps has a separate search field and compact rows.
Create has a multiline bounded composer, 128-character count and separated action.
No on-screen keyboard or new capacity was added.

Bottom navigation is Home / Apps / Create. Global Create always requests a new
app; edit app inside an installed app (or F2) retains that app as the update target.
Changing destinations cancels uncommitted work. Existing app toolbar coordinates
remain stable. The saved/live badge indicates source mode, not authentication or
inference on each action. Host connection checking is separate and explicit.
