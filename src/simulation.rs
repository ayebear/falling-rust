use crate::element::{Element, ElementForm};
use crate::sandbox::*;
use bevy::prelude::*;
use bevy::utils::Instant;

#[derive(Clone, Debug)]
pub struct Simulation {
    pub running: bool,
    pub step: bool,
    pub frame_time_ms: u128,
}

impl Default for Simulation {
    fn default() -> Self {
        Self {
            running: true,
            step: false,
            frame_time_ms: 0,
        }
    }
}

pub fn level_updater(mut level: ResMut<SandBox>, mut simulation: ResMut<Simulation>) {
    let start = Instant::now();
    if simulation.running || simulation.step {
        simulation.step = false;
        let visited = level.toggle_visited_state();
        let (width, height) = (level.width() - 1, level.height() - 1);
        for y in (1..height).rev() {
            // Switch X order every frame to avoid simulation artifacts
            if visited {
                for x in 1..width {
                    update_cell(x, y, &mut level);
                }
            } else {
                for x in (1..width).rev() {
                    update_cell(x, y, &mut level);
                }
            }
        }
    }
    let duration = Instant::now() - start;
    simulation.frame_time_ms = duration.as_millis();
}

fn update_cell(x: usize, y: usize, level: &mut SandBox) {
    let cell = level.get(x, y);
    if cell.visited == level.is_visited_state() {
        // Visited this one already
        return;
    }
    let marked_as_visited = match cell.element {
        Element::Air => update_air(x, y, level),
        Element::Sand => update_sand(x, y, level),
        Element::Water => update_water(x, y, level),
        Element::Acid => update_acid(x, y, level),
        Element::Oil => update_oil(x, y, level),
        Element::Drain => update_drain(x, y, level),
        Element::Fire => update_fire(x, y, level),
        Element::Ash => update_ash(x, y, level),
        Element::Lava => update_lava(x, y, level),
        Element::Smoke => update_smoke(x, y, level),
        Element::Life => update_life(x, y, level),
        Element::Iron => update_iron(x, y, level),
        Element::Rust => update_sand(x, y, level),
        Element::Plant => update_plant(x, y, level),
        Element::Wood => false,
        Element::Rock => false,
        Element::Indestructible => false,
        Element::WaterSource => update_source(x, y, Element::Water, level),
        Element::AcidSource => update_source(x, y, Element::Acid, level),
        Element::OilSource => update_source(x, y, Element::Oil, level),
        Element::LavaSource => update_source(x, y, Element::Lava, level),
        Element::FireSource => update_source(x, y, Element::Fire, level),
    };
    if !marked_as_visited {
        level.set_visited(x, y);
    }
}

fn update_sand(x: usize, y: usize, level: &mut SandBox) -> bool {
    let element_below = level.get(x, y + 1).element;
    if element_below == Element::Air
        || element_below == Element::Water
        || element_below == Element::Fire
        || element_below == Element::Oil
    {
        // Fall down
        level.swap(x, y, x, y + 1);
        return true;
    }
    if element_below == Element::Acid {
        // Dissolve into the acid below
        if level.get_mut(x, y).dissolve_to(Element::Air) {
            level.clear_cell(x, y + 1);
            return false;
        } else {
            level.swap(x, y, x, y + 1);
            return true;
        }
    }
    let neighbour_x = level.random_neighbour_x(x);
    let neighbour_element = level.get(neighbour_x, y + 1).element;
    if neighbour_element == Element::Air || neighbour_element == Element::Water {
        // Slide to random neighbour diagonally
        level.swap(x, y, neighbour_x, y + 1);
        return true;
    }
    if neighbour_element == Element::Acid {
        // Dissolve in acid diagonally
        if level.get_mut(neighbour_x, y + 1).dissolve_to(Element::Air) {
            level.clear_cell(x, y + 1);
            return false;
        } else {
            level.swap(x, y, neighbour_x, y + 1);
            return true;
        }
    }
    false
}

fn update_water(x: usize, y: usize, level: &mut SandBox) -> bool {
    let random = level.random(60);
    let check_x = if random < 58 {
        x
    } else if random == 58 {
        x - 1
    } else {
        x + 1
    };
    // First see what hapens if we touch the below neighbour
    if let Some(value) = touch_water(level, x, y, check_x, y + 1, random) {
        return value;
    }
    // Water flows sideways
    for n in 1..16 {
        let check_x_opt = if random < 30 {
            if x > n {
                Some(x - n)
            } else {
                None
            }
        } else {
            if x + n < level.width() - 1 {
                Some(x + n)
            } else {
                None
            }
        };
        if let Some(check_x) = check_x_opt {
            let neighbour = level.get(check_x, y);
            let neighbour_element = neighbour.element;
            if let Some(value) = touch_water(level, x, y, check_x, y, random) {
                return value;
            }
            if neighbour_element != Element::Water {
                break;
            }
        }
    }
    return false;
}

fn touch_water(
    level: &mut SandBox,
    water_x: usize,
    water_y: usize,
    other_x: usize,
    other_y: usize,
    random: usize,
) -> Option<bool> {
    let other_element = level.get(other_x, other_y).element;
    if other_element == Element::Air || other_element == Element::Oil {
        level.swap(water_x, water_y, other_x, other_y);
        return Some(true);
    }
    if other_element == Element::Acid {
        level.get_mut(other_x, other_y).dissolve_to(Element::Water);
        if water_y < other_y && random % 2 == 0 {
            level.swap(water_x, water_y, other_x, other_y)
        }
        return Some(false);
    }
    if other_element == Element::Lava {
        // Lava cools down
        if level.get_mut(other_x, other_y).dissolve_to(Element::Rock) {
            level.clear_cell(water_x, water_y);
        }
        return Some(false);
    }
    if other_element == Element::Fire {
        level.clear_cell(water_x, water_y);
        level.set_element(other_x, other_y, Element::Water);
        return Some(true);
    }
    None
}

fn update_acid(x: usize, y: usize, level: &mut SandBox) -> bool {
    let random = level.random(60);
    let check_x = if random < 50 {
        x
    } else if random < 55 {
        x - 1
    } else {
        x + 1
    };
    let element_below = level.get(check_x, y + 1).element;
    if element_below == Element::Air || element_below == Element::Fire {
        // Acid falls down in air and fire
        level.swap(x, y, check_x, y + 1);
        return true;
    }
    if element_below == Element::Water {
        // Acid turns to water when in contact
        level.get_mut(x, y).dissolve_to(Element::Water);
        return false;
    }
    if element_below.dissolves_in_acid() {
        if level.get_mut(check_x, y + 1).dissolve_to(Element::Air) {
            level.clear_cell(x, y);
            return true;
        }
        return false;
    }
    // Acid flows sideways in air (somewhat more slowly than water)
    for n in 1..8 {
        let check_x_opt = if random < 30 {
            if x > n {
                Some(x - n)
            } else {
                None
            }
        } else {
            if x + n < level.width() - 1 {
                Some(x + n)
            } else {
                None
            }
        };
        if let Some(check_x) = check_x_opt {
            let neighbour = level.get(check_x, y);
            let neighbour_element = neighbour.element;
            if neighbour_element == Element::Air {
                level.swap(x, y, check_x, y);
                return true;
            }
            if neighbour_element.dissolves_in_acid() {
                if level.get_mut(check_x, y).dissolve_to(Element::Air) {
                    level.clear_cell(x, y);
                    return true;
                }
                return true;
            }
            if neighbour_element != Element::Acid {
                break;
            }
        }
    }
    false
}

fn update_oil(x: usize, y: usize, level: &mut SandBox) -> bool {
    let random = level.random(500);
    let check_x = if random > 50 {
        x
    } else if random > 25 {
        x - 1
    } else {
        x + 1
    };
    let element_below = level.get(check_x, y + 1).element;
    if element_below == Element::Air || element_below == Element::Acid {
        // Oil falls down in air and acid
        level.swap(x, y, check_x, y + 1);
        return true;
    }
    // Oil flows sideways in air and water (somewhat more slowly than water)
    for n in 1..8 {
        let check_x_opt = if random < 250 {
            if x > n {
                Some(x - n)
            } else {
                None
            }
        } else {
            if x + n < level.width() - 1 {
                Some(x + n)
            } else {
                None
            }
        };
        if let Some(check_x) = check_x_opt {
            let neighbour = level.get(check_x, y);
            let neighbour_element = neighbour.element;
            if neighbour_element == Element::Air || (n == 1 && neighbour_element == Element::Acid) {
                level.swap(x, y, check_x, y);
                return true;
            }
            if neighbour_element != Element::Oil {
                break;
            }
        }
    }
    false
}

fn update_drain(x: usize, y: usize, level: &mut SandBox) -> bool {
    // Remove any liquid on top, left or right of this cell
    let element_form = level.get(x, y - 1).element.form();
    if element_form == ElementForm::Liquid {
        level.clear_cell(x, y - 1);
        return true;
    }
    let element_form = level.get(x - 1, y).element.form();
    if element_form == ElementForm::Liquid {
        level.clear_cell(x - 1, y);
        return true;
    }
    let element_form = level.get(x + 1, y).element.form();
    if element_form == ElementForm::Liquid {
        level.clear_cell(x + 1, y);
        return true;
    }
    false
}

fn update_fire(x: usize, y: usize, level: &mut SandBox) -> bool {
    let random = level.random(5);
    // Reduce fire strength over time
    if random > 3 && !level.reduce_strength(x, y) {
        level.set_element(x, y, Element::Smoke);
        return true;
    }
    // Make fire flicker
    let cell = level.get_mut(x, y);
    cell.variant = (cell.variant + random as u8 * 10) % 255;
    // Move in a random direction, with a tendency upwards
    let (nx, ny) = match random {
        0 => (x, y + 1),
        1 => (x + 1, y),
        2 => (x - 1, y),
        _ => (x, y - 1),
    };
    let element = level.get(nx, ny).element;
    if element == Element::Air {
        level.swap(x, y, nx, ny);
        return true;
    }
    if element.burns() {
        if element.form() == ElementForm::Solid && random > 3 {
            // Sometimes burnable elements turn into ash
            level.get_mut(nx, ny).dissolve_to(Element::Ash);
        } else {
            level.get_mut(nx, ny).dissolve_to(Element::Fire);
        }
        return false;
    }
    false
}

fn update_ash(x: usize, y: usize, level: &mut SandBox) -> bool {
    update_sand(x, y, level)
}

fn update_lava(x: usize, y: usize, level: &mut SandBox) -> bool {
    let random = level.random(500);
    // Make lava glow
    let cell = level.get_mut(x, y);
    cell.variant = (cell.variant + random as u8) % 255;
    // Cool down when no longer at max hotness
    if random < 250 && cell.strength < 64 {
        if level.get_mut(x, y).dissolve_to(Element::Rock) {
            return true;
        }
    }
    // Give off sparks
    if random == 0 && level.get(x, y - 1).element == Element::Air {
        level.set_element(x, y - 1, Element::Fire);
    }
    // Fall down
    if let Some(visited) = touch_lava(level, x, y, x, y + 1) {
        return visited;
    }
    // Slide down diagonally
    let neighbour_x = level.random_neighbour_x(x);
    if let Some(visited) = touch_lava(level, x, y, neighbour_x, y + 1) {
        return visited;
    }
    // Slide horizontally
    if let Some(visited) = touch_lava(level, x, y, neighbour_x, y) {
        return visited;
    }
    false
}

fn touch_lava(
    level: &mut SandBox,
    lava_x: usize,
    lava_y: usize,
    other_x: usize,
    other_y: usize,
) -> Option<bool> {
    let element = level.get(other_x, other_y).element;
    if element == Element::Air
        || element == Element::Acid
        || element == Element::Water
        || element == Element::Fire
    {
        level.swap(lava_x, lava_y, other_x, other_y);
        return Some(true);
    }
    if element.burns() {
        level.get_mut(other_x, other_y).dissolve_to(Element::Fire);
        return Some(false);
    }
    None
}

fn update_smoke(x: usize, y: usize, level: &mut SandBox) -> bool {
    let random = level.random(5);
    // Reduce fire strength over time
    if random > 2 && !level.reduce_strength(x, y) {
        level.clear_cell(x, y);
        return true;
    }
    // Move in a random direction, with a tendency upwards
    let (nx, ny) = match random {
        0 => (x + 1, y),
        1 => (x - 1, y),
        _ => (x, y - 1),
    };
    let neighbour_element = level.get(nx, ny).element;
    if neighbour_element == Element::Air {
        level.swap(x, y, nx, ny);
        return true;
    }
    if neighbour_element == Element::Fire || neighbour_element.form() == ElementForm::Liquid {
        level.clear_cell(x, y);
        return true;
    }
    false
}

fn update_iron(x: usize, y: usize, level: &mut SandBox) -> bool {
    let rusty_neighbour = level.get(x - 1, y).element.causes_rust()
        || level.get(x + 1, y).element.causes_rust()
        || level.get(x, y - 1).element.causes_rust()
        || level.get(x, y + 1).element.causes_rust();

    if rusty_neighbour {
        // Rust iron by reducing its strength somewhat randomly
        let random = level.random(5);
        if random > 2 && !level.reduce_strength(x, y) {
            // Turn into rust
            level.set_element(x, y, Element::Rust);
            return true;
        }
    }
    false
}

fn update_plant(x: usize, y: usize, level: &mut SandBox) -> bool {
    let mut count = 0;
    for (xx, yy) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
        if level.random(10) <= 1 && level.get(xx, yy).element.grows_plant() {
            level.set_element(xx, yy, Element::Plant);
            count += 1;
        }
    }
    count > 0
}

fn update_source(x: usize, y: usize, element: Element, level: &mut SandBox) -> bool {
    if level.get(x, y + 1).element != element {
        level.set_element(x, y + 1, element);
        return true;
    }
    false
}

fn update_air(x: usize, y: usize, level: &mut SandBox) -> bool {
    let mut living_neighbours = 0;
    if level.get(x - 1, y - 1).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x, y - 1).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x + 1, y - 1).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x - 1, y).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x + 1, y).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x - 1, y + 1).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x, y + 1).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x + 1, y + 1).element == Element::Life {
        living_neighbours += 1;
    }
    if living_neighbours == 3 {
        level.set_element(x, y, Element::Life);
        return true;
    }
    false
}

fn update_life(x: usize, y: usize, level: &mut SandBox) -> bool {
    let mut living_neighbours = 0;
    if level.get(x - 1, y - 1).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x, y - 1).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x + 1, y - 1).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x - 1, y).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x + 1, y).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x - 1, y + 1).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x, y + 1).element == Element::Life {
        living_neighbours += 1;
    }
    if level.get(x + 1, y + 1).element == Element::Life {
        living_neighbours += 1;
    }
    if living_neighbours < 2 || living_neighbours > 3 {
        level.set_element(x, y, Element::Air);
        return true;
    }
    // Keep on living
    false
}
