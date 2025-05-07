mod utils;

use std::fmt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    is_alive: bool,
    brightness: u8,
    colour: (u8, u8, u8),
}

#[wasm_bindgen]
impl Cell {
    pub fn is_alive(&self) -> bool {
        self.is_alive
    }

    pub fn brightness(&self) -> u8 {
        self.brightness
    }
}

impl Cell {
    pub fn new(is_alive: bool, brightness: u8, colour: (u8, u8, u8)) -> Cell {
        Cell { is_alive, brightness, colour }
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}
impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                if self.cells[idx].is_alive {
                    count += 1;
                }
            }
        }
        count
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell.is_alive {
                    if cell.brightness > 128 { '◼' } else { '◻' }
                } else {
                    ' ' // Empty space for dead cells
                };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}


#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();
    
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbours = self.live_neighbor_count(row, col);
    
                // Determine next cell state
                let next_cell = if cell.is_alive {
                    match live_neighbours {
                        2 | 3 => Cell::new(true, cell.brightness, ((85 * live_neighbours), 0, 0)), // Survives
                        _ => Cell::new(false, 0, (255, 255, 255)), // Dies
                    }
                } else {
                    match live_neighbours {
                        3 => Cell::new(true, 255, (0, 0, 0)), // Becomes alive
                        _ => Cell::new(false, cell.brightness, cell.colour), // Stays dead
                    }
                };
    
                next[idx] = next_cell;
            }
        }
    
        self.cells = next;
    }

    pub fn new() -> Universe {
        utils::set_panic_hook();
    
        let width = 128;
        let height = 128;
    
        let cells = (0..width * height)
            .map(|i| {
                if i % 3 == 0 || i % 7 == 0 {
                    Cell::new(true, (i % 256) as u8, (0, 0, 0)) 
                } else {
                    Cell::new(false, 0, (255, 255, 255)) 
                }
            })
            .collect();
    
        Universe {
            width,
            height,
            cells,
        }
    }
    

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }
}
