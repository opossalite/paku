use ndarray::Array2;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::PacError;

/*
NOTES:
https://pacman.holenet.info/

The fruit appears after 70 dots are eaten and again after 170 dots are eaten unless the first fruit is still there. They will disappear if they are not eaten after 9-10 seconds.

Possibly spawn ghosts in this way:
- Blinky instantly
- Pinky after 1 dot eaten
- Inky after 2 dots eaten
- Clyde after 3 dots eaten
as this will space the ghosts apart by 1 tile, so no overlap
*/

pub struct Game {
    /*
    note: these are 4x as fine as the tiles of the map
    - the map is 28x31, so entities have a map of 112x124
    */
    pub pacman_loc: (f64, f64),
    pub blinky_loc: (f64, f64),
    pub pinky_loc: (f64, f64),
    pub inky_loc: (f64, f64),
    pub clyde_loc: (f64, f64),

    /// left of the 2x1
    pub pacman_spawn: (usize, usize),

    /// top left of the 8x5
    pub ghost_spawn: (usize, usize),

    /// up to 3, can only gain one life at 10000pts and thats it
    pub lives: usize,

    /// exist in perfect pairs, we store their coordinates
    pub warps: HashMap<i32, ((usize, usize), (usize, usize))>,

    /*
    breakdown:
    pac-dot: 10pts
    power pellet: 50pts
    eating ghost: 200, 400, 800, 1600 (in sequence)
    fruit: 100-5000pts depending on level:
    | Level | Item                 | Points |
    | ----- | -------------------- | ------ |
    | 1     | Cherry               | 100    |
    | 2     | Strawberry           | 300    |
    | 3–4   | Orange               | 500    |
    | 5–6   | Apple                | 700    |
    | 7–8   | Melon                | 1,000  |
    | 9–10  | Galaxian (spaceship) | 2,000  |
    | 11–12 | Bell                 | 3,000  |
    | 13+   | Key                  | 5,000  |
    */
    pub points: usize,

    /*
    <0 = warp (paired)
    0 = empty
    1 = wall
    2 = pac-dot
    3 = power pellet
    */
    /// live board, updated as the game progresses
    pub board: Array2<i32>,
}

impl Game {
    pub fn try_from_file(path: &Path) -> Result<Self, PacError> {
        let txt = fs::read_to_string(path).map_err(|_| PacError::FileRead)?;
        let rows: Vec<Vec<char>> = txt.lines().map(|l| l.chars().collect::<Vec<_>>()).collect();

        if rows.is_empty() {
            return Err(PacError::LevelEmpty);
        }

        let width = rows[0].len();
        if width == 0 {
            return Err(PacError::LevelEmpty);
        }

        // ensure rectangle
        for row in rows.iter() {
            if row.len() != width {
                return Err(PacError::LevelNotRectangular);
            }
        }
        let height = rows.len();

        // track consumed cells so multi-char tokens don't get double-counted
        let mut consumed = vec![vec![false; width]; height];

        // 1) Find ghost spawn: exactly one 8x5 block of '@'
        let ghost_block_w = 8;
        let ghost_block_h = 5;
        let mut ghost_top_left: Option<(usize, usize)> = None;
        for y in 0..=height.saturating_sub(ghost_block_h) {
            for x in 0..=width.saturating_sub(ghost_block_w) {
                // check all '@'
                let mut all_at = true;
                for yy in 0..ghost_block_h {
                    for xx in 0..ghost_block_w {
                        if rows[y + yy][x + xx] != '@' {
                            all_at = false;
                            break;
                        }
                    }
                    if !all_at {
                        break;
                    }
                }
                if all_at {
                    // ensure we haven't already found one
                    if ghost_top_left.is_some() {
                        return Err(PacError::MultipleGhostSpawns);
                    }
                    ghost_top_left = Some((x, y));
                    // mark consumed
                    for yy in 0..ghost_block_h {
                        for xx in 0..ghost_block_w {
                            consumed[y + yy][x + xx] = true;
                        }
                    }
                }
            }
        }
        // Ensure no stray @ elsewhere
        for y in 0..height {
            for x in 0..width {
                if rows[y][x] == '@' && !consumed[y][x] {
                    return Err(PacError::InvalidGhostSpawn);
                }
            }
        }
        let ghost_spawn;
        match ghost_top_left {
            None => {
                return Err(PacError::NoGhostSpawn);
            }
            Some(tup) => {
                // quick bounds check
                if tup.1 == 0 || tup.1 + 5 >= height {
                    return Err(PacError::InvalidGhostSpawnPeripheral);
                }

                // ensure empty spaces above and below center for ghost and fruit spawning
                if rows[tup.1 - 1][tup.0 + 3] == ' '
                    && rows[tup.1 - 1][tup.0 + 4] == ' '
                    && rows[tup.1 + 5][tup.0 + 3] == ' '
                    && rows[tup.1 + 5][tup.0 + 4] == ' '
                {
                    ghost_spawn = tup;
                } else {
                    return Err(PacError::InvalidGhostSpawnPeripheral);
                }
            }
        }

        // 2) Find pacman spawn: exactly one horizontal "$$" (2x1)
        let mut pacman_spawn: Option<(usize, usize)> = None;
        for y in 0..height {
            for x in 0..width.saturating_sub(1) {
                if !consumed[y][x] && !consumed[y][x + 1] {
                    if rows[y][x] == '$' && rows[y][x + 1] == '$' {
                        if pacman_spawn.is_some() {
                            return Err(PacError::MultiplePacSpawns);
                        }
                        pacman_spawn = Some((x, y));
                        consumed[y][x] = true;
                        consumed[y][x + 1] = true;
                    }
                }
            }
        }
        // Ensure no stray $ elsewhere
        for y in 0..height {
            for x in 0..width {
                if rows[y][x] == '$' && !consumed[y][x] {
                    return Err(PacError::InvalidPacSpawn);
                }
            }
        }
        let pacman_spawn = pacman_spawn.ok_or_else(|| PacError::NoPacSpawn)?;

        // 3) Collect digits (warps)
        //let mut warp_counts: HashMap<u8, usize> = HashMap::new();
        let mut warp_coords: HashMap<u8, Vec<(usize, usize)>> = HashMap::new();
        for y in 0..height {
            for x in 0..width {
                if consumed[y][x] {
                    continue;
                }
                let c = rows[y][x];
                if c.is_ascii_digit() {
                    let id = c.to_digit(10).unwrap() as u8;
                    //*warp_counts.entry(id).or_default() += 1;
                    warp_coords.entry(id).or_default().push((x, y));
                    // mark consumed? we allow digits to remain to convert to warp values; but mark consumed to avoid double token handling
                    consumed[y][x] = true;
                }
            }
        }
        if !warp_coords.is_empty() {
            // check each digit count == 2
            for (_, coords) in warp_coords.iter() {
                if coords.len() != 2 {
                    return Err(PacError::InvalidWarp);
                }
            }

            // ensure digits start at 1 and contiguous
            let mut ids: Vec<u8> = warp_coords.keys().cloned().collect();
            ids.sort();
            if ids[0] != 1 {
                return Err(PacError::InvalidWarp);
            }
            for (i, &id) in ids.iter().enumerate() {
                if id as usize != i + 1 {
                    return Err(PacError::InvalidWarp);
                }
            }
            if ids.len() > 9 {
                return Err(PacError::InvalidWarp);
            }
        }
        let warps = warp_coords
            .iter()
            .map(|(id, coords)| (*id as i32 * -1, (coords[0], coords[1])))
            .collect::<HashMap<i32, ((usize, usize), (usize, usize))>>();

        // Build the numeric board
        // row-major: iterate y then x
        let mut flat: Vec<i32> = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                // if this cell was part of consumed multi-char token (like $$ or @ block) we already marked consumed and should treat it as empty (0),
                // unless it's a digit (we marked digits consumed too).
                let c = rows[y][x];
                let value = match c {
                    ' ' => 0,
                    '#' => 1,
                    '-' => 2,
                    '!' => 3,
                    '$' => {
                        // we have stored the location of the pac spawn, so $ is now treated as
                        // empty tiles
                        0
                    }
                    '@' => {
                        // similar to $, we convert @ to a wall
                        1
                    }
                    '1'..='9' => {
                        let id = c.to_digit(10).unwrap() as i32;
                        // warps stored as negative numbers: -id
                        -id
                    }
                    _ => {
                        return Err(PacError::InvalidCharacters);
                    }
                };
                flat.push(value);
            }
        }

        let board = Array2::from_shape_vec((height, width), flat)
            .map_err(|_| PacError::ConversionToArray)?;

        // spawns: place all ghosts at ghost_spawn and pacman at pacman_spawn
        let (px, py) = pacman_spawn;
        let (gx, gy) = ghost_spawn;

        Ok(Game {
            pacman_spawn,
            ghost_spawn,
            pacman_loc: (px as f64 + 0.5, py as f64), //center pacman properly in his spawn
            blinky_loc: (gx as f64 + 3.5, gy as f64 - 1.0), //place blinky above the spawn
            pinky_loc: (gx as f64 + 3.5, gy as f64 + 2.0), //place pinky at the center of spawn
            inky_loc: (gx as f64 + 1.5, gy as f64 + 2.0), //place inky on the left of pinky
            clyde_loc: (gx as f64 + 5.5, gy as f64 + 2.0), //place clyde on the right of pinky
            board,
            lives: 3,
            points: 0,
            warps,
        })
    }
}
