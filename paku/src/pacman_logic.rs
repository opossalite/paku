use std::collections::HashSet;

use crate::pacman::{Direction, Game};

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


    /// Determine the best action for the ghost given the current situation
    fn determine_ghost_action(&mut self, ghost_position: (f64, f64), target: (f64, f64), direction: Direction) {
        let mut dirs = HashSet::from([Direction::Down, Direction::Up, Direction::Left, Direction::Right]);
        dirs.remove(&direction.opposite()); //ghost cannot turn around
        /*
        for example of moving in x coords:
        1.0 means you can check 0.0 and 2.0
        0.75 means you can definitely move towards 0.0 and 1.0
        0.5 means you can definitely move towards 0.0 and 1.0
        0.25 means you can definitely move towards 0.0 and 1.0
        0.0 means you can check -1.0 and 1.0

        aka
        if the coord has a fract, then both the floor and the ceil are valid
        if the coord has no fract, then check both -1.0 and +1.0
        */

        if ghost_position.0.fract() == 0.0 {
            // check left (-1) and right (+1) to check for a wall or outside of map
            let left = ghost_position.0 - 1.0;
            let right = ghost_position.1 + 1.0;
        }

        //if self.level[left] == 2 {
        //}
    }

    fn blinky_target(&mut self) {

    }

    fn pinky_target(&mut self) {

    }

    fn inky_target(&mut self) {

    }

    fn clyde_target(&mut self) {

    }

}
