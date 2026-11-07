// SPDX-License-Identifier: MIT; see LICENSE in this directory.
// Copyright (c) 2026 Sodium Horizon experimental contributors
// Android static-bundle adaptation: Sodium Horizon's settings page.
pub(super) fn run(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::official_sodium as sodium;
    if mod_id != sodium::ID {
        return;
    }
    let settings = sodium::settings(|key| game.native_setting(sodium::ID, key));
    buttons.extend(settings.labels(
        game.settings.render_distance,
        game.settings.simulation_distance,
    ));
}
