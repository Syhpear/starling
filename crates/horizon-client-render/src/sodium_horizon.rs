// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Sodium Horizon experimental contributors
// Static Android adaptation of the Sodium Horizon runtime (extra_runtime.rs and
// the section visibility resolver). Render-only: no lighting, world simulation
// or network changes. Everything here is a no-op until `set_active(true)`, so a
// build with Sodium Horizon switched off behaves exactly as before.
use std::sync::{
    RwLock,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

/// Switch bits of `configure`'s mask.
pub const ANIM_WATER: usize = 0;
pub const ANIM_LAVA: usize = 1;
pub const ANIM_FIRE: usize = 2;
pub const ANIM_PORTAL: usize = 3;
pub const ANIM_OTHER: usize = 4;
pub const ANIM_SCULK: usize = 5;
pub const PARTICLES: usize = 6;
pub const MOD_PARTICLES: usize = 7;
pub const ITEM_FRAMES: usize = 8;
pub const ARMOR_STANDS: usize = 9;
pub const PAINTINGS: usize = 10;
pub const ADVANCEMENT_TOASTS: usize = 11;
pub const RECIPE_TOASTS: usize = 12;
pub const TUTORIAL_TOASTS: usize = 13;
pub const COORDINATES: usize = 14;
pub const STARS: usize = 15;
/// Everything on except the coordinates overlay.
pub const DEFAULT_MASK: u64 = u64::MAX & !(1 << COORDINATES);

static ACTIVE: AtomicBool = AtomicBool::new(false);
static ENABLED: AtomicU64 = AtomicU64::new(DEFAULT_MASK);
static DISABLED_PARTICLES: [AtomicU64; 8] = [const { AtomicU64::new(u64::MAX) }; 8];
static LAYERS: RwLock<Vec<u8>> = RwLock::new(Vec::new());

/// Whether the mod is switched on in Options > Mods.
pub fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}
pub fn set_active(on: bool) {
    ACTIVE.store(on, Ordering::Relaxed);
}

/// Raised chunk distance limits and the bitwise section visibility resolver
/// are part of the mod, not switches of it.
pub fn extended_distance() -> bool {
    active()
}
pub fn fast_visibility() -> bool {
    active()
}

/// Switch `bit`; always true while the mod is off.
pub fn enabled(bit: usize) -> bool {
    !active() || ENABLED.load(Ordering::Relaxed) & (1 << bit) != 0
}
pub fn mask() -> u64 {
    ENABLED.load(Ordering::Relaxed)
}
pub fn configure(mask: u64, disabled_particles: &[usize]) {
    for word in &DISABLED_PARTICLES {
        word.store(u64::MAX, Ordering::Relaxed);
    }
    for &id in disabled_particles {
        if let Some(word) = DISABLED_PARTICLES.get(id / 64) {
            word.fetch_and(!(1 << (id % 64)), Ordering::Relaxed);
        }
    }
    ENABLED.store(mask, Ordering::Relaxed);
}
pub fn particle_allowed(id: usize, named: bool) -> bool {
    if !active() {
        return true;
    }
    enabled(PARTICLES)
        && if named {
            enabled(MOD_PARTICLES)
        } else {
            DISABLED_PARTICLES
                .get(id / 64)
                .is_none_or(|word| word.load(Ordering::Relaxed) & (1 << (id % 64)) != 0)
        }
}

/// The animation switch a sprite belongs to, from its atlas name.
pub fn category(name: &str) -> u8 {
    let path = name.strip_prefix("minecraft:").unwrap_or(name);
    if path.starts_with("block/water_") {
        ANIM_WATER as u8
    } else if path.starts_with("block/lava_") {
        ANIM_LAVA as u8
    } else if path.starts_with("block/fire_") || path.starts_with("block/soul_fire_") {
        ANIM_FIRE as u8
    } else if path == "block/nether_portal" || path == "block/end_portal" {
        ANIM_PORTAL as u8
    } else if path.starts_with("block/sculk") {
        ANIM_SCULK as u8
    } else {
        ANIM_OTHER as u8
    }
}
pub fn register_layer(layer: u32, name: &str) {
    let Ok(mut layers) = LAYERS.write() else {
        return;
    };
    // A malformed pack must not request an unbounded table allocation.
    if layer >= 1_048_576 {
        return;
    }
    let length = layers.len().max(layer as usize + 1);
    layers.resize(length, ANIM_OTHER as u8);
    layers[layer as usize] = category(name);
}
pub fn animation_allowed(layer: u32) -> bool {
    if !active() {
        return true;
    }
    let mask = ENABLED.load(Ordering::Relaxed);
    if mask & 63 == 63 {
        return true;
    }
    let category = LAYERS
        .read()
        .ok()
        .and_then(|layers| layers.get(layer as usize).copied())
        .unwrap_or(ANIM_OTHER as u8);
    mask & (1 << category) != 0
}

// ---- section visibility -------------------------------------------------

/// Rows are indexed y * 16 + z, with bit x denoting an open cell.
pub const ALL_VISIBLE: u64 = (1u64 << 36) - 1;

#[inline]
fn span(seed: u16, open: u16) -> u16 {
    let (mut left, mut right) = (seed, seed);
    let (mut lmask, mut rmask) = (open, open);
    for shift in [1, 2, 4, 8] {
        left |= lmask & (left << shift);
        right |= rmask & (right >> shift);
        lmask &= lmask << shift;
        rmask &= rmask >> shift;
    }
    left | right
}

/// Face connectivity of one 16^3 section, as `mesher::visibility_reusing`
/// computes it. Only boundary-connected regions affect the matrix; each
/// discovered run is consumed once, on the mesher worker's existing stack.
pub fn resolve(open: &[u16; 256], stack: &mut Vec<usize>) -> u64 {
    stack.clear();
    if open.iter().all(|&row| row == u16::MAX) {
        return ALL_VISIBLE;
    }
    let mut remaining = *open;
    let mut result = 0u64;
    for start in 0..256 {
        let boundary = if !(16..240).contains(&start) || start & 15 == 0 || start & 15 == 15 {
            u16::MAX
        } else {
            0x8001
        };
        while remaining[start] & boundary != 0 {
            let seeds = remaining[start] & boundary;
            stack.push((start << 16) | (seeds & seeds.wrapping_neg()) as usize);
            let mut faces = 0u8;
            while let Some(entry) = stack.pop() {
                let row = entry >> 16;
                let seed = entry as u16 & remaining[row];
                if seed == 0 {
                    continue;
                }
                let run = span(seed, remaining[row]);
                remaining[row] &= !run;
                let (y, z) = (row >> 4, row & 15);
                faces |= u8::from(y == 0)
                    | (u8::from(y == 15) << 1)
                    | (u8::from(z == 0) << 2)
                    | (u8::from(z == 15) << 3)
                    | (u8::from(run & 1 != 0) << 4)
                    | (u8::from(run & 0x8000 != 0) << 5);
                if faces == 63 {
                    stack.clear();
                    return ALL_VISIBLE;
                }
                for next in [
                    row.checked_sub(16),
                    (row < 240).then_some(row + 16),
                    if z > 0 { Some(row - 1) } else { None },
                    if z < 15 { Some(row + 1) } else { None },
                ]
                .into_iter()
                .flatten()
                {
                    let seed = run & remaining[next];
                    if seed != 0 {
                        stack.push((next << 16) | seed as usize);
                    }
                }
            }
            for a in 0..6 {
                if faces & (1 << a) != 0 {
                    result |= (faces as u64) << (a * 6);
                }
            }
            if result == ALL_VISIBLE {
                return result;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cell-by-cell flood fill `mesher::visibility_reusing` runs.
    fn reference(open: &[u16; 256]) -> u64 {
        let mut seen = [false; 4096];
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    seen[y << 8 | z << 4 | x] = open[y * 16 + z] & (1 << x) == 0;
                }
            }
        }
        let mut result = 0u64;
        let mut stack = Vec::new();
        for start in 0..4096usize {
            if seen[start] {
                continue;
            }
            let mut faces = 0u8;
            seen[start] = true;
            stack.push(start);
            while let Some(c) = stack.pop() {
                let (x, y, z) = ((c & 15) as i32, (c >> 8) as i32, ((c >> 4) & 15) as i32);
                faces |= u8::from(y == 0)
                    | (u8::from(y == 15) << 1)
                    | (u8::from(z == 0) << 2)
                    | (u8::from(z == 15) << 3)
                    | (u8::from(x == 0) << 4)
                    | (u8::from(x == 15) << 5);
                for (dx, dy, dz) in [
                    (1, 0, 0),
                    (-1, 0, 0),
                    (0, 1, 0),
                    (0, -1, 0),
                    (0, 0, 1),
                    (0, 0, -1),
                ] {
                    let (nx, ny, nz) = (x + dx, y + dy, z + dz);
                    if !(0..16).contains(&nx) || !(0..16).contains(&ny) || !(0..16).contains(&nz) {
                        continue;
                    }
                    let n = (ny << 8 | nz << 4 | nx) as usize;
                    if !seen[n] {
                        seen[n] = true;
                        stack.push(n);
                    }
                }
            }
            for a in 0..6 {
                if faces & (1 << a) != 0 {
                    for b in 0..6 {
                        if faces & (1 << b) != 0 {
                            result |= 1 << (a * 6 + b);
                        }
                    }
                }
            }
        }
        result
    }

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
    }

    #[test]
    fn resolver_matches_the_flood_fill_on_random_sections() {
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        let mut stack = Vec::new();
        for round in 0..4000 {
            // Vary the opaque density from nearly empty to nearly solid.
            let density = (round % 20) as u64 * 5 + 2;
            let mut open = [0u16; 256];
            for row in &mut open {
                for x in 0..16 {
                    if rng.next() % 100 >= density {
                        *row |= 1 << x;
                    }
                }
            }
            assert_eq!(
                resolve(&open, &mut stack),
                reference(&open),
                "round {round}"
            );
        }
    }

    #[test]
    fn empty_and_solid_sections() {
        let mut stack = Vec::new();
        assert_eq!(resolve(&[u16::MAX; 256], &mut stack), ALL_VISIBLE);
        assert_eq!(resolve(&[0; 256], &mut stack), 0);
        assert_eq!(reference(&[0; 256]), 0);
    }

    #[test]
    fn filters_do_nothing_while_the_mod_is_off() {
        set_active(false);
        configure(0, &[1, 2]);
        assert!(enabled(STARS) && particle_allowed(1, false) && animation_allowed(7));
        configure(DEFAULT_MASK, &[]);
    }
}
