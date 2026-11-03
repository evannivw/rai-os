# An app that controls the guest display

RAI-OS now supports `display.brightness`, the first declarative app capability
with an effect outside its own records. It attenuates RGB values drawn into the
QEMU framebuffer, from 20 to 100 percent. It does not change a physical screen's
backlight, the host monitor brightness, battery telemetry or power consumption.
No energy-saving result is claimed. The user example has a real guest-visible
effect; a physical phone still needs its own display/backlight driver.

## Create and use

Run the existing app launcher with your own Meta key, then enter:

```text
Create an app to adjust the RAI-OS display brightness from 20 to 100 percent, with a default of 50.
```

```sh
python3 -B scripts/apps.py --source meta --output target/brightness-session-1
```

The exact prompt is also available in the explicitly labeled offline replay.
See [MICROAPPS.md](MICROAPPS.md) for VNC setup and [APP_STORAGE.md](APP_STORAGE.md)
for saved devices. Review displays **Allow: change display brightness**. Enter or
Install confirms the capability and creates the app icon. Installation does not
apply the default. In the app, edit the percentage and press **Apply brightness**
or Enter. The setting affects subsequent drawing throughout this guest display,
including the launcher. No model call is made on Apply.

The **Reset to 100%** button and F6 restore full brightness. F6 works throughout
the app launcher, even when another app is selected. On macOS a function key may
need Fn. Values outside 20..100 are rejected rather than silently clamped. The
white sample bar and current percentage show the actual rendering setting.

The app schema/permission persists through the existing explicit host checkpoint.
The applied level is deliberately session-local: boot starts at 100 percent.
Restoration, opening the app, installing it and updating its schema never actuate
it automatically. A restored app can apply its default after a new user action.
This also avoids surprising device actions during saved-state replay.

## Capability contract

Each app declares exactly one supported capability:

| Capability | Shape | Execute |
| --- | --- | --- |
| `records.local` | One to six typed fields, optional check filter | Save/edit local rows, tables and totals |
| `display.brightness` | Exactly one required numeric field; default string 20..100; no filter | Apply the field as software brightness |

The installed schema is the source of authority. A caller cannot grant itself a
permission by putting a capability name in an action. The domain checks the
selected installed app and requested value before returning a typed brightness
value. The controller also checks that a guest display exists. Records-only apps
cannot use app.run; display apps cannot save record rows. An update cannot change
an app's capability, so converting an existing records app requires creating a
separate explicitly approved control app. Unsupported capabilities remain denied.

Meta receives the versioned [runtime catalog](../fixtures/microapps/v1/runtime.json)
and a strict two-capability JSON schema. Its output remains a proposal. Unknown
operations, extra control fields, empty/out-of-range defaults, stale responses
and attempted capability changes are rejected before installation. Compatibility
with old records-only saved devices is retained; older binaries that do not know
this new capability cannot load a brightness app.

This is an interpreter policy boundary, not process isolation. Runtime, controller
and display adapter still share EL1 privilege. The local host and UART diagnostics
remain trusted. There is no generated native code, arbitrary MMIO access, generic
privileged shell or automatic grant of other hardware capabilities.

## Implementation

- `microapp::Capability` identifies supported app authority. `Brightness` can only
  hold the checked range. `Runtime::brightness` validates installed permission.
- `app_ui` handles explicit Apply/reset and checks display availability.
- `display::Display` attenuates new pixel colors. Alpha blending scales only the
  new source, because the destination is already attenuated; this prevents a
  repeated redraw from compounding brightness. It uses the existing exclusively
  owned, clipped framebuffer write path. No additional unsafe block was added.
- `app.schema` now reports capability explicitly. The bridge/checkpoint preserve
  it; restore does not invoke actions. `app.display` is read-only diagnostics:
  availability, current percentage and mode=software_rgb.
- `app.run` executes the current installed control app's form value; F6/input code
  1010 is a direct user recovery path. Neither command bypasses domain validation.

RGB attenuation is integer multiplication/division in display-channel space, not
perceptually calibrated physical luminance. Changing brightness redraws the view;
new drawing respects the setting. QEMU may observe transient writes as usual;
there is no new v-sync or compositor guarantee.

## Verify

```sh
python3 -B scripts/verify_app_brightness.py --debug --headless --output target/brightness-debug
python3 -B scripts/verify_app_brightness.py --output target/brightness-release
# Clean committed source only; maximum one real request, no retry:
python3 -B scripts/verify_app_brightness.py --source meta --output target/brightness-live
# Offline portable path:
docker run --rm --network none rai-os-demo apps-brightness-verify
```

The verifier reads actual QEMU RGB pixels, not only a status label. It checks
50% home-screen attenuation within integer blend rounding, an exact 127/255 white
bar at 50%, exact 51/255 at 20%, stable repeated redraws and byte-identical recovery
to the original home screen at 100%. It also rejects invalid input, missing
permission, capability escalation and late responses. A fresh guest restores the
control schema without actuating; the no-display kernel rejects execution with
display_unavailable. The records fixture used to test denied authority is explicitly
scripted and is not reported as another model-generated app.

Next system capabilities should follow this same sequence: implement the actual
backend, define bounded authority and effect, show consent in app review, verify
execution/failure/recovery, then advertise it to Meta. Merely adding an operation
name to a prompt does not implement the corresponding OS function.
