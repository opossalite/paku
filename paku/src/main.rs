mod pacman;
mod tyrosine_wrap;

use std::path::Path;

use thiserror::Error;

use crate::pacman::Game;

#[derive(Debug, Error)]
pub enum PacError {
    #[error("Failed to read level file.")]
    FileRead,
    #[error("Level file empty or width 0. Ensure no empty lines.")]
    LevelEmpty,
    #[error("Level not rectangular, rows or column counts are irregular.")]
    LevelNotRectangular,

    #[error("Couldn't locate a Pac-Man spawn.")]
    NoPacSpawn,
    #[error("Found multiple Pac-Man spawns.")]
    MultiplePacSpawns,
    #[error(
        "Stray $ might've been used. Must be used as a horizontal pair to declare the Pac-Man spawn. Only one Pac-Man spawn allowed."
    )]
    InvalidPacSpawn,

    #[error("Couldn't locate a Ghost spawn.")]
    NoGhostSpawn,
    #[error("Found multiple Ghost spawns.")]
    MultipleGhostSpawns,
    #[error(
        "Stray @ might've been used. Must be used as a 8-long x 5-wide rectangle to declare the Ghost spawn. Only one spawn allowed."
    )]
    InvalidGhostSpawn,
    #[error(
        "Two blank spaces above and below the center of the spawn must be available for ghost and fruit spawning."
    )]
    InvalidGhostSpawnPeripheral,
    #[error(
        "Warp numbers must appear in pairs, and must use contiguous numbers starting at 1. Each number can only be used twice (one pair)."
    )]
    InvalidWarp,

    #[error("Invalid characters found.")]
    InvalidCharacters,
    #[error("Yeah idk what causes this error yet, but it happens when converting to a 2D NDArray.")]
    ConversionToArray,
}

fn main() {
    let x = Game::try_from_file(Path::new("./config/levels/0.lvl")).unwrap();

    println!("Hello, world!");
}
