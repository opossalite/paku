use ndarray::Array2;
use std::fs;
use std::path::Path;

use crate::PacError;

/*
NOTES:
https://pacman.holenet.info/

The fruit appears after 70 dots are eaten and again after 170 dots are eaten unless the first fruit is still there. They will disappear if they are not eaten after 9-10 seconds.

Power Pellet Durations:
*/

pub struct Game {
    /*
    note: these are 4x as fine as the tiles of the map
    - the map is 28x31, so entities have a map of 112x124
    */
    pub pacman_loc: (usize, usize),
    pub blinky_loc: (usize, usize),
    pub pinky_loc: (usize, usize),
    pub inky_loc: (usize, usize),
    pub clyde_loc: (usize, usize),

    /// center, entrance is located 3 tiles up
    pub ghost_spawn: (usize, usize),

    /// up to 3, can only gain one life at 10000pts and thats it
    pub lives: usize,

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
    pub fn from_file(path: &Path) -> Self {
        let g = Self::try_from_file(path).unwrap_or_else(|e| panic!("level parse error: {}", e));
        g
    }

    fn try_from_file(path: &Path) -> Result<Self, PacError> {
        let txt = fs::read_to_string(path).map_err(|_| PacError::FileRead)?;
        let rows: Vec<Vec<char>> = txt.lines().map(|l| l.chars().collect::<Vec<_>>()).collect();

        if rows.is_empty() {
            return Err(PacError::LevelEmpty);
            //panic!("level file is empty");
        }

        let width = rows[0].len();
        if width == 0 {
            return Err(PacError::LevelEmpty);
            //panic!("level width is zero");
        }
        // ensure rectangle
        for (i, r) in rows.iter().enumerate() {
            if r.len() != width {
                return Err(PacError::LevelNotRectangular);
                //panic!(
                //    "level is not rectangular: row 0 has width {}, row {} has width {}",
                //    width,
                //    i,
                //    r.len()
                //);
            }
        }
        let height = rows.len();

        // track consumed cells so multi-char tokens don't get double-counted
        let mut consumed = vec![vec![false; width]; height];

        // 1) Find ghost spawn: exactly one 8x5 block of '@'
        let ghost_block_w = 8usize;
        let ghost_block_h = 5usize;
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
                        //panic!("multiple 8x5 '@' blocks (only one ghost spawn allowed)");
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
        if ghost_top_left.is_none() {
            return Err(PacError::NoGhostSpawn);
            // Also check possibility of stray single '@' chars
            //let any_at = rows.iter().flatten().any(|&c| c == '@');
            //if any_at {
            //panic!("'@' characters present but not forming a single 8x5 block");
            //} else {
            //panic!("no '@' block found: ghost spawn (8x5 of '@') is required");
            //}
        }
        let (gtx, gty) = ghost_top_left.unwrap();
        // center of 8x5 block: top_left + (3,2)
        let ghost_spawn = (gtx + 3, gty + 2);

        // 2) Find pacman spawn: exactly one horizontal "$$" (2x1)
        let mut pacman_loc: Option<(usize, usize)> = None;
        for y in 0..height {
            for x in 0..width.saturating_sub(1) {
                if !consumed[y][x] && !consumed[y][x + 1] {
                    if rows[y][x] == '$' && rows[y][x + 1] == '$' {
                        if pacman_loc.is_some() {
                            return Err(PacError::MultiplePacSpawns);
                            //panic!("multiple '$$' pacman spawns found (only one allowed)");
                        }
                        pacman_loc = Some((x, y));
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
                    //panic!(
                    //    "stray '$' found at ({},{}); pacman spawn must be exactly two adjacent '$' characters",
                    //    x, y
                    //);
                }
            }
        }
        //let pacman_loc = pacman_loc.ok_or_else(|| anyhow::anyhow!("no '$$' pacman spawn found"))?;
        let pacman_loc = pacman_loc.ok_or_else(|| PacError::NoPacSpawn)?;

        // 3) Collect digits (warps)
        use std::collections::HashMap;
        let mut warp_counts: HashMap<u8, usize> = HashMap::new();
        for y in 0..height {
            for x in 0..width {
                if consumed[y][x] {
                    continue;
                }
                let c = rows[y][x];
                if c.is_ascii_digit() {
                    let id = c.to_digit(10).unwrap() as u8;
                    *warp_counts.entry(id).or_default() += 1;
                    // mark consumed? we allow digits to remain to convert to warp values; but mark consumed to avoid double token handling
                    consumed[y][x] = true;
                }
            }
        }
        if !warp_counts.is_empty() {
            // check each digit count == 2
            for (&id, &count) in warp_counts.iter() {
                if count != 2 {
                    return Err(PacError::InvalidWarp);
                    //panic!(
                    //    "warp digit '{}' appears {} times; each warp digit must appear exactly twice",
                    //    id, count
                    //);
                }
            }
            // ensure digits start at 1 and contiguous
            let mut ids: Vec<u8> = warp_counts.keys().cloned().collect();
            ids.sort();
            if ids[0] != 1 {
                return Err(PacError::InvalidWarp);
                //panic!(
                //    "warps must start at '1' if any warps are present; found smallest warp '{}'",
                //    ids[0]
                //);
            }
            for (i, &id) in ids.iter().enumerate() {
                if id as usize != i + 1 {
                    return Err(PacError::InvalidWarp);
                    //panic!(
                    //    "warp digits must be contiguous starting at 1. expected {}, found {}",
                    //    i + 1,
                    //    id
                    //);
                }
            }
            if ids.len() > 9 {
                return Err(PacError::InvalidWarp);
                //panic!(
                //    "at most 9 distinct warp ids (1..9) supported; found {}",
                //    ids.len()
                //);
            }
        }

        // 4) Ensure no leftover '@' (should be consumed by the 8x5 search)
        for y in 0..height {
            for x in 0..width {
                if rows[y][x] == '@' && !consumed[y][x] {
                    return Err(PacError::InvalidGhostSpawn);
                    //panic!(
                    //    "'@' found outside the 8x5 ghost spawn block at ({},{})",
                    //    x, y
                    //);
                }
            }
        }

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
                        // consumed $ should not remain; if any present, error above would have caught stray $
                        0
                    }
                    '@' => {
                        // '@'s are inside the 8x5 block and treated as empty tiles in the live board.
                        0
                    }
                    '0'..='9' => {
                        let id = c.to_digit(10).unwrap() as i32;
                        // warps stored as negative numbers: -id
                        -id
                    }
                    _ => {
                        return Err(PacError::InvalidCharacters);
                        //panic!("unexpected char '{}' at ({},{})", other, x, y);
                    }
                };
                flat.push(value);
            }
        }

        let board = Array2::from_shape_vec((height, width), flat)
            .map_err(|_| PacError::ConversionToArray)?;

        // set ghosts: place all ghosts at ghost_spawn
        let (gx, gy) = ghost_spawn;
        let (pacx, pacy) = pacman_loc;

        Ok(Game {
            pacman_loc: (pacx, pacy),
            blinky_loc: (gx, gy),
            pinky_loc: (gx, gy),
            inky_loc: (gx, gy),
            clyde_loc: (gx, gy),
            ghost_spawn: (gx, gy),
            board,
        })
    }
}

