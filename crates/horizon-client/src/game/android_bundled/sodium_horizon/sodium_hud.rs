// SPDX-License-Identifier: MIT; see LICENSE in this directory.
// Copyright (c) 2026 Sodium Horizon experimental contributors
// Android static-bundle adaptation of extra_coordinates.rs: the coordinates
// overlay, in the top left corner, off by default.
pub(super) fn run(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::official_sodium as sodium;
    use horizon_client::extras::Corner;
    use horizon_client_render::sodium_horizon as filters;
    if !game.native_on(sodium::ID)
        || !filters::enabled(filters::COORDINATES)
        || game.reduced_debug_info()
        || !game.native_corner_shown(Corner::TopLeft)
    {
        return;
    }
    let Some(player) = game.native_player() else {
        return;
    };
    let line = format!("X: {:.1}  Y: {:.1}  Z: {:.1}", player.x, player.y, player.z);
    let width = game.native_text_width(&line) + 8.0;
    let (x, y) = corners.place(Corner::TopLeft, width, 16.0);
    game.native_fill(batch, [x, y, width, 16.0], 0xA010141A);
    game.native_text(batch, &line, [x + 4.0, y + 4.0], 0xE8EBEF, true);
}
