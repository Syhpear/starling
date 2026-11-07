// SPDX-License-Identifier: MIT; see LICENSE in this directory.
// Copyright (c) 2026 Sodium Horizon experimental contributors
// Android static-bundle adaptation: Sodium Horizon's row in Options > Mods.
pub(super) fn run(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::official_sodium as sodium;
    mods.push(crate::game::native_hooks::NativeMod {
        id: sodium::ID.into(),
        title: format!("Sodium Horizon {}", sodium::VERSION),
        description: sodium::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
