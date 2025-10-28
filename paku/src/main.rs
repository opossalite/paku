mod pacman;
mod tyrosine_wrap;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PacError {
    #[error("Failed to read level file.")]
    FileRead,
    #[error("Level file empty or width 0.")]
    LevelEmpty,
    #[error("Level not rectangular, rows or column counts are irregular.")]
    LevelNotRectangular,

    #[error("Couldn't locate a Pac-Man spawn.")]
    NoPacSpawn,
    #[error("Found multiple Pac-Man spawns.")]
    MultiplePacSpawns,
    #[error("Stray $ used. Must be used as a horizontal pair to declare the Pac-Man spawn.")]
    InvalidPacSpawn,

    #[error("Couldn't locate a Ghost spawn.")]
    NoGhostSpawn,
    #[error("Found multiple Ghost spawns.")]
    MultipleGhostSpawns,
    #[error(
        "Stray @ used. Must be used as a 8-long x 5-wide rectangle to declare the Ghost spawn."
    )]
    InvalidGhostSpawn,
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
    println!("Hello, world!");
}
