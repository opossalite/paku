use std::{fs::File, path::Path, str::FromStr};

use csv::ReaderBuilder;
use fraction::Fraction;
use serde::{Deserialize, Deserializer, Serialize};

use crate::PacError;

//pub fn try_from_file(path: &Path) {
//    match GameSettingsRow::try_from_file(path) {
//        Ok(x) => println!("{:?}", x),
//        Err(x) => println!("{}\n{:?}", x, x),
//    }
//}
pub fn try_settings_from_file(path: &Path) -> Result<Vec<GameSettingsRow>, PacError> {
    let file = File::open(path).map_err(|_| PacError::FileReadSettings)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut settings = Vec::new();
    for result in rdr.deserialize() {
        let record: GameSettingsRow = result.map_err(|err| PacError::CSVParse(err))?;
        settings.push(record);
    }

    Ok(settings)
}

/// Custom deserializer that processes Fractions a bit better than default
fn deserialize_fraction<'de, D>(deserializer: D) -> Result<Fraction, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Fraction::from_str(&s).map_err(|_| serde::de::Error::custom(format!("invalid number: {}", s)))
}

#[derive(Debug, Deserialize)]
pub struct GameSettingsRow {
    #[serde(rename = "Level")]
    pub level: String, // "1", "21+" etc.

    #[serde(rename = "Fruit")]
    pub fruit: String,

    #[serde(rename = "Pac-Man Speed", deserialize_with = "deserialize_fraction")]
    pub pacman_speed: Fraction,

    #[serde(rename = "Ghost Speed", deserialize_with = "deserialize_fraction")]
    pub ghost_speed: Fraction,

    #[serde(
        rename = "Fright Ghost Speed",
        deserialize_with = "deserialize_fraction"
    )]
    pub fright_ghost_speed: Fraction,

    #[serde(rename = "Fright Length", deserialize_with = "deserialize_fraction")]
    pub fright_length: Fraction,

    #[serde(rename = "Scatter0", deserialize_with = "deserialize_fraction")]
    pub scatter0: Fraction,

    #[serde(rename = "Chase0", deserialize_with = "deserialize_fraction")]
    pub chase0: Fraction,

    #[serde(rename = "Scatter1", deserialize_with = "deserialize_fraction")]
    pub scatter1: Fraction,

    #[serde(rename = "Chase1", deserialize_with = "deserialize_fraction")]
    pub chase1: Fraction,

    #[serde(rename = "Scatter2", deserialize_with = "deserialize_fraction")]
    pub scatter2: Fraction,

    #[serde(rename = "Chase2", deserialize_with = "deserialize_fraction")]
    pub chase2: Fraction,

    #[serde(rename = "Scatter3", deserialize_with = "deserialize_fraction")]
    pub scatter3: Fraction, // “1033” or “1/60” — we’ll handle below
}
impl GameSettingsRow {
}
