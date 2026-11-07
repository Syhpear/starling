// SPDX-License-Identifier: MIT; see LICENSE in this directory.
// Copyright (c) 2026 Sodium Horizon experimental contributors
// Android static-bundle adaptation: every frame, the render crate's filters
// follow the mod's switches and whether it is turned on in Options > Mods.
pub(super) fn run(game: &mut crate::game::Game<'_>) {
    use crate::game::official_sodium as sodium;
    let settings = sodium::settings(|key| game.native_setting(sodium::ID, key));
    sodium::apply(settings, game.native_on(sodium::ID));
}
