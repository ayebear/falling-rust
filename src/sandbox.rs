use crate::{cell::*, element::Element};
use rand::Rng;
use rand_xoshiro::{rand_core::SeedableRng, Xoshiro256Plus};

pub struct SandBox {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    visited_state: bool,
    random: Xoshiro256Plus,
}

impl SandBox {
    pub fn new(width: usize, height: usize) -> Self {
        let mut world = SandBox::empty(width, height);
        // Set indestructible pixels at the border to ease computations
        for x in 0..world.width() {
            world.set_element(x, 0, Element::Indestructible, false);
            world.set_element(x, world.height() - 1, Element::Indestructible, false);
        }
        for y in 0..world.height() {
            world.set_element(0, y, Element::Indestructible, false);
            world.set_element(world.width() - 1, y, Element::Indestructible, false);
        }
        world
    }

    fn empty(width: usize, height: usize) -> Self {
        SandBox {
            width,
            height,
            cells: vec![
                Cell {
                    element: Element::Air,
                    variant: 0,
                    strength: 0,
                    visited: false,
                    source: false
                };
                width * height
            ],
            visited_state: false,
            random: Xoshiro256Plus::from_entropy(),
        }
    }

    pub fn get(&self, x: usize, y: usize) -> &Cell {
        let index = self.index(x, y);
        &self.cells[index]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        let index = self.index(x, y);
        &mut self.cells[index]
    }

    pub fn reduce_strength(&mut self, x: usize, y: usize) -> bool {
        let index = self.index(x, y);
        let cell = &mut self.cells[index];
        if cell.strength > 1 {
            cell.strength -= 1;
            true
        } else {
            false
        }
    }

    pub fn clear_cell(&mut self, x: usize, y: usize) {
        self.set_element(x, y, Element::Air, false);
    }

    pub fn set_element(&mut self, x: usize, y: usize, element: Element, source: bool) {
        let index = self.index(x, y);
        let mut cell = &mut self.cells[index];
        if cell.element == Element::Indestructible {
            // Cannot edit these blocks
            return;
        }
        cell.element = element;
        cell.visited = self.visited_state;
        cell.strength = element.strength();
        cell.source = source;
        if element.randomize_color_factor() > 0.0 {
            cell.variant = self.random.gen();
        }
    }

    pub fn swap(&mut self, x: usize, y: usize, x2: usize, y2: usize) {
        let index1 = self.index(x, y);
        let index2 = self.index(x2, y2);
        let mut cell = self.cells[index1].clone();
        let mut cell2 = self.cells[index2].clone();
        if cell.element == Element::Indestructible || cell2.element == Element::Indestructible {
            // Cannot edit these blocks
            return;
        }
        // cell is moved to the place of cell 2, so becomes the second cell
        cell.visited = self.visited_state;
        cell2.visited = self.visited_state;
        self.cells[index1] = cell2;
        self.cells[index2] = cell;
    }

    pub fn set_visited(&mut self, x: usize, y: usize) {
        let index = self.index(x, y);
        self.cells[index].visited = self.visited_state;
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn toggle_visited_state(&mut self) -> bool {
        self.visited_state = !self.visited_state;
        self.visited_state
    }

    pub fn is_visited_state(&self) -> bool {
        self.visited_state
    }

    pub fn random_neighbour_x(&mut self, x: usize) -> usize {
        if self.random.gen_range(0..1000) % 2 == 0 {
            x + 1
        } else {
            x - 1
        }
    }

    pub fn random(&mut self, max: usize) -> usize {
        self.random.gen_range(0..1000 * max) % max
    }

    pub fn clear(&mut self) {
        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let index = self.index(x, y);
                let mut cell = &mut self.cells[index];
                cell.element = Element::Air;
                cell.visited = self.visited_state;
            }
        }
    }

    #[inline(always)]
    fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.width
    }
}

impl Default for SandBox {
    fn default() -> Self {
        SandBox::new(512, 512)
    }
}
