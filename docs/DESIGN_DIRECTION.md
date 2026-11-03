# Native interface directions for the final demo

Status: design proposals, not implemented guest screens. November 3, 2026.
The user requested a black, terminal-inspired phone interface with sparse text,
keyboard-like linework and recognizable applications. Keep native QEMU rendering
as the product target; conversational interactive mockups only explore layouts.
Example app names and rows in those mockups are illustrative, not new live evidence.

## Recommended: Tactile Terminal

A quiet home screen with three application launchers, simple bracket monograms,
a persistent prompt entry and one recent application. Use off-white primary text,
muted readable secondary text, one pale green accent and fine structural lines.
The prompt is the strongest action; applications remain easy to open without AI.
Corners and brackets provide identity without a wall of decorative ASCII text.
This best matches the user's stated mobile/terminal direction and the current
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
