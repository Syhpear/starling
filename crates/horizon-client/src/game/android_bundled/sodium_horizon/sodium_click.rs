// SPDX-License-Identifier: MIT; see LICENSE in this directory.
// Copyright (c) 2026 Sodium Horizon experimental contributors
// Android static-bundle adaptation: a click on Sodium Horizon's settings page.
pub(super) fn run(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::official_sodium as sodium;
    if mod_id != sodium::ID {
        return;
    }
    let mut settings = sodium::settings(|key| game.native_setting(sodium::ID, key));
    if settings.click(button) {
        // A switch: keep it.
    } else if button == sodium::BUTTON_RENDER {
        game.settings.render_distance = sodium::next_distance(game.settings.render_distance);
        game.apply_settings();
        return;
    } else if button == sodium::BUTTON_SIMULATION {
        game.settings.simulation_distance =
            sodium::next_distance(game.settings.simulation_distance);
        game.apply_settings();
        return;
    } else if button == sodium::BUTTON_RESET {
        settings = sodium::Settings::defaults();
    } else {
        return;
    }
    sodium::set(settings);
    sodium::apply(settings, game.native_on(sodium::ID));
    for (key, value) in settings.stored() {
        game.native_set_setting(sodium::ID, key, &value);
    }
}
