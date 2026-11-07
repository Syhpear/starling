// SPDX-License-Identifier: MIT; see LICENSE in this directory.
// Copyright (c) 2026 Sodium Horizon experimental contributors
// Source: Sodium Horizon 0.3.1 (.hnmod). Android static-bundle adaptation: the
// desktop mod's mouse-driven options screen is replaced by a page of buttons
// on Options > Mods, and its injections are calls placed in the game's own
// functions (`horizon_client_render::sodium_horizon`).
//
// Settings are the mod's files in its data folder (one per switch, `true` or
// `false`); the render and simulation distances stay Horizon's own options.
use horizon_client_render::sodium_horizon as filters;
use std::cell::RefCell;

pub const ID: &str = "sodium_horizon";
pub const VERSION: &str = "0.3.1-android";
pub const DESCRIPTION: &str = "Faster section visibility, chunk distances past 32, and switches for animations, particles, frames, armor stands, paintings, toasts, stars and a coordinates overlay.";

/// The switches in page order: mask bit, file name, label.
pub const SWITCHES: [(usize, &str, &str); 16] = [
    (filters::ANIM_WATER, "anim_water", "Water animation"),
    (filters::ANIM_LAVA, "anim_lava", "Lava animation"),
    (filters::ANIM_FIRE, "anim_fire", "Fire animation"),
    (filters::ANIM_PORTAL, "anim_portal", "Portal animation"),
    (filters::ANIM_SCULK, "anim_sculk", "Sculk animation"),
    (filters::ANIM_OTHER, "anim_other", "Other animations"),
    (filters::PARTICLES, "particles", "Particles"),
    (filters::MOD_PARTICLES, "mod_particles", "Mod particles"),
    (filters::ITEM_FRAMES, "item_frames", "Item frames"),
    (filters::ARMOR_STANDS, "armor_stands", "Armor stands"),
    (filters::PAINTINGS, "paintings", "Paintings"),
    (
        filters::ADVANCEMENT_TOASTS,
        "toasts_advancement",
        "Advancement toasts",
    ),
    (filters::RECIPE_TOASTS, "toasts_recipe", "Recipe toasts"),
    (
        filters::TUTORIAL_TOASTS,
        "toasts_tutorial",
        "Tutorial toasts",
    ),
    (filters::STARS, "stars", "Stars"),
    (filters::COORDINATES, "coordinates", "Coordinates overlay"),
];
/// Buttons after the switches: render distance, then simulation distance.
pub const BUTTON_RENDER: usize = SWITCHES.len();
pub const BUTTON_SIMULATION: usize = SWITCHES.len() + 1;
pub const BUTTON_RESET: usize = SWITCHES.len() + 2;
/// The distances the two buttons cycle through, in chunks.
pub const DISTANCES: [i32; 10] = [2, 4, 8, 12, 16, 24, 32, 48, 64, 128];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub mask: u64,
}

impl Settings {
    pub fn defaults() -> Self {
        Self {
            mask: filters::DEFAULT_MASK,
        }
    }
    /// From the stored values (`get` reads one file); a switch with none keeps its default.
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let mut mask = filters::DEFAULT_MASK;
        for (bit, key, _) in SWITCHES {
            match get(key).as_deref().map(str::trim) {
                Some("true") => mask |= 1 << bit,
                Some("false") => mask &= !(1 << bit),
                _ => {}
            }
        }
        Self { mask }
    }
    pub fn on(&self, bit: usize) -> bool {
        self.mask & (1 << bit) != 0
    }
    /// The switches' labels, then the distances' and the reset button's.
    pub fn labels(&self, render: i32, simulation: i32) -> Vec<String> {
        let mut labels: Vec<String> = SWITCHES
            .iter()
            .map(|&(bit, _, label)| format!("{label}: {}", if self.on(bit) { "ON" } else { "OFF" }))
            .collect();
        labels.push(format!("Render distance: {render} chunks"));
        labels.push(format!("Simulation distance: {simulation} chunks"));
        labels.push("Reset Sodium Horizon".into());
        labels
    }
    /// A click on a switch; false when `button` is not one.
    pub fn click(&mut self, button: usize) -> bool {
        match SWITCHES.get(button) {
            Some(&(bit, _, _)) => {
                self.mask ^= 1 << bit;
                true
            }
            None => false,
        }
    }
    /// The files to keep: name and value.
    pub fn stored(&self) -> Vec<(&'static str, String)> {
        SWITCHES
            .iter()
            .map(|&(bit, key, _)| (key, self.on(bit).to_string()))
            .collect()
    }
}

/// The next distance after `current` in `DISTANCES`, wrapping to the first.
pub fn next_distance(current: i32) -> i32 {
    DISTANCES
        .iter()
        .copied()
        .find(|&d| d > current)
        .unwrap_or(DISTANCES[0])
}

thread_local! {
    static CACHED: RefCell<Option<Settings>> = const { RefCell::new(None) };
}
/// The settings in effect, read once with `get` and kept until `reset` or `set`.
pub fn settings(get: impl Fn(&str) -> Option<String>) -> Settings {
    CACHED.with(|c| *c.borrow_mut().get_or_insert_with(|| Settings::read(get)))
}
pub fn set(settings: Settings) {
    CACHED.with(|c| *c.borrow_mut() = Some(settings));
}
/// A launcher transition or profile change: forget the cache and switch the mod's filters off.
pub fn reset() {
    CACHED.with(|c| *c.borrow_mut() = None);
    filters::set_active(false);
    filters::configure(filters::DEFAULT_MASK, &[]);
}
/// Make the render crate's filters follow `settings` and whether the mod is on.
pub fn apply(settings: Settings, on: bool) {
    filters::configure(settings.mask, &[]);
    filters::set_active(on);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_files_keep_the_defaults_and_stored_ones_win() {
        let none = Settings::read(|_| None);
        assert_eq!(none, Settings::defaults());
        assert!(!none.on(filters::COORDINATES) && none.on(filters::STARS));
        let some = Settings::read(|k| match k {
            "stars" => Some("false\n".into()),
            "coordinates" => Some("true".into()),
            "paintings" => Some("garbage".into()),
            _ => None,
        });
        assert!(
            !some.on(filters::STARS)
                && some.on(filters::COORDINATES)
                && some.on(filters::PAINTINGS)
        );
    }

    #[test]
    fn a_click_flips_one_switch_and_the_files_round_trip() {
        let mut s = Settings::defaults();
        assert!(s.click(6)); // Particles
        assert!(!s.on(filters::PARTICLES));
        assert!(!s.click(BUTTON_RENDER));
        let files = s.stored();
        let back = Settings::read(|k| files.iter().find(|(n, _)| *n == k).map(|(_, v)| v.clone()));
        assert_eq!(back, s);
    }

    #[test]
    fn distances_cycle_and_wrap() {
        assert_eq!(next_distance(12), 16);
        assert_eq!(next_distance(13), 16);
        assert_eq!(next_distance(128), 2);
        assert_eq!(next_distance(1), 2);
    }

    #[test]
    fn the_page_has_a_label_per_button() {
        let labels = Settings::defaults().labels(12, 8);
        assert_eq!(labels.len(), BUTTON_RESET + 1);
        assert_eq!(labels[BUTTON_RENDER], "Render distance: 12 chunks");
        assert_eq!(labels[15], "Coordinates overlay: OFF");
    }
}
