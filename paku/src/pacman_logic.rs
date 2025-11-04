use crate::pacman::Game;

pub enum Entity {
    Blinky,
    Pinky,
    Inky,
    Clyde,
    PacMan,
}

impl Game {

    /// Given a list of entities, return all chosen entities to their default positions
    fn restart_positions(&mut self, entities: &[Entity]) {
        let (px, py) = (self.pacman_spawn.0 as f64, self.pacman_spawn.1 as f64);
        let (gx, gy) = (self.ghost_spawn.0 as f64, self.pacman_spawn.1 as f64);
        for entity in entities {
            match entity {
                Entity::Blinky => self.blinky_loc = (gx + 0.5, gy - 1.0), //place blinky above the spawn
                Entity::Pinky => self.pinky_loc = (gx + 3.5, gy + 2.0), //place pinky at the center of spawn
                Entity::Inky => self.inky_loc = (gx + 1.5, gy + 2.0), //place inky on the left of pinky
                Entity::Clyde => self.clyde_loc = (gx + 5.5, gy + 2.0), //place clyde on the right of pinky
                Entity::PacMan => self.pacman_loc = (px + 0.5, py), //center pacman properly in his spawn
            }
        }
    }

}
