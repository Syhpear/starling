# Sodium Horizon (Android static bundle)

Sodium Horizon 0.3.1's `.hnmod` adapted to the fixed Android bundle. The source
revision is the desktop mod's; its Horizon pin was `c9d474dd`, while this
branch is on the 2026-11-01 nightly, so every call site below is a small edit
made in place rather than a copy of the desktop mod's replacement body.

| Desktop injection | Android |
|---|---|
| `visibility` (OVERWRITE of the mesher's flood fill) | `mesher::visibility_reusing` calls `sodium_horizon::resolve` while the mod is on |
| `distance_*` (ticket clamps 32 -> 128) | `ticket_levels::max_distance`, `reach` |
| `extra_particles`, `extra_sprite`, `extra_animations` | `ParticleEngine::create`, `Sprites::register`, `SpriteAnimator::collect` |
| `extra_frames`, `extra_stands`, `extra_paintings`, `extra_stars` | `frame_geometry`, `armor_stand_geometry`, `painting_geometry`, `Stars::draw` |
| `extra_toasts`, `extra_tutorial` | `draw_toasts`, `draw_tutorial_toasts` |
| `extra_coordinates` | `sodium_hud.rs`, from `native_hud` |
| `ui_*`, `exact_*`, `icons` (mouse-driven options screen) | not ported: a page of buttons on Options > Mods (`sodium_*.rs`) |

Not ported: per-particle-type switches, the desktop options screen's pages and
its Video/Quality/Performance rows. The render and simulation distances are
Horizon's own options; the page only steps them.

Everything in `horizon_client_render::sodium_horizon` is a no-op until the mod
is switched on, so a build with it off behaves as before.
