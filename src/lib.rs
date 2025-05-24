mod utils;


use std::fmt;
use std::ops::Sub;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct U(u8, u8);


#[wasm_bindgen]
impl U {
    #[wasm_bindgen(constructor)]
    pub fn new(x: u8, y: u8) -> U {
        U(x, y)
    }


    pub fn x(&self) -> u8 {
        self.0
    }


    pub fn y(&self) -> u8 {
        self.1
    }
}
impl Sub for U {
    type Output = U;


    fn sub(self, other: U) -> U {
        U(self.0 - other.0, self.1 - other.1)
    }
}


#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    is_alive: bool,
    u: U,
    colour: (u8, u8, u8),
}


#[wasm_bindgen]
impl Cell {
    pub fn is_alive(&self) -> bool {
        self.is_alive
    }


    pub fn u(&self) -> U {
        self.u.clone()
    }
}


impl Cell {
    pub fn new(is_alive: bool, u: U, colour: (u8, u8, u8)) -> Cell {
        Cell { is_alive, u, colour }
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


    fn live_neighbour_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;


        let mut height_array = Vec::new();
        let mut width_array = Vec::new();


        if row != 0 {
            height_array.push(self.height - 1);
        }
        height_array.push(0);
        if row != self.height - 1 {
            height_array.push(1);
        }


        if column != 0 {
            width_array.push(self.width - 1);
        }
        width_array.push(0);
        if column != self.width - 1 {
            width_array.push(1);
        }


        for delta_row in height_array.iter().cloned() {
            for delta_col in width_array.iter().cloned() {
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


    fn convection_calculation(&self, row: u32, column: u32, u_initial: U) -> U {
        let mut u_final = u_initial;
        let mut height_array = Vec::new();
        let mut width_array = Vec::new();


        if row != 0 {
            height_array.push(self.height - 1);
        }
        height_array.push(0);
        if row != self.height - 1 {
            height_array.push(1);
        }


        if column != 0 {
            width_array.push(self.width - 1);
        }
        width_array.push(0);
        if column != self.width - 1 {
            width_array.push(1);
        }


        // u[1:, 1:] = (un[1:, 1:] - (c * dt / dx * (un[1:, 1:] - un[1:, :-1])) -
        // (c * dt / dy * (un[1:, 1:] - un[:-1, 1:])))
        let c = 4.0;
        let dt = 0.1;
        let dx = 1.0;
        let dy = 1.0;


        // for delta_row in height_array.iter().cloned() {
        //     for delta_col in width_array.iter().cloned() {
        //         if delta_row == 0 && delta_col == 0 {
        //             continue;
        //         }


        //         let neighbor_row = (row + delta_row) % self.height;
        //         let neighbor_col = (column + delta_col) % self.width;


        //         let idx = self.get_index(neighbor_row, neighbor_col);
        //         let u_neighbor = self.cells[idx].u;


        //         // u[j, i] = (un[j, i] - (c * dt / dx * (un[j, i] - un[j, i - 1])) - (c * dt / dy * (un[j, i] - un[j - 1, i])))
        //         //Convert python convection calculation to rust    
        //         u_final.0 = (u_initial.0 as f64 - (c * (dt / dx) * (u_initial.0 as f64 - u_neighbor.0 as f64))) as u8;
        //         u_final.1 = (u_initial.1 as f64 - (c * (dt / dy) * (u_initial.1 as f64 - u_neighbor.1 as f64))) as u8;
        //     }
        // }
        let neighbor_row = (row + 1) % self.height;
        let neighbor_col = (column + 1) % self.width;


        let idx = self.get_index(neighbor_row, neighbor_col);
        let u_neighbor = self.cells[idx].u;


        u_final.0 = (u_initial.0 as f64 - (c * (dt / dx) * (u_initial.0 as f64 - u_neighbor.0 as f64))) as u8;
        u_final.1 = (u_initial.1 as f64 - (c * (dt / dy) * (u_initial.1 as f64 - u_neighbor.1 as f64))) as u8;


        u_final
    }
}


impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell.is_alive {
                    if cell.u.x() > 128 { '◼' } else { '◻' }
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
                // let live_neighbours = self.live_neighbour_count(row, col);


                // // Determine next cell state
                // let next_cell = if cell.is_alive {
                //     match live_neighbours {
                //         2 | 3 => Cell::new(true, cell.u, ((85 * live_neighbours), 0, 0)),
                //         _ => Cell::new(false, U(0, 0), (255, 255, 255)),
                //     }
                // } else {
                //     match live_neighbours {
                //         3 => Cell::new(true, U(255, 255), (0, 0, 0)),
                //         _ => Cell::new(false, cell.u, cell.colour),
                //     }
                // };
                let next_cell = Cell::new(true, self.convection_calculation(row, col, cell.u), (0, 0, 0));


                next[idx] = next_cell;
            }
        }
   
        self.cells = next;
    }


    pub fn new() -> Universe {
        utils::set_panic_hook();


        let width = 128;
        let height = 128;


        // let cells = (0..width * height)
        //     .map(|i| {
        //         if i % width == 0  {
        //             Cell::new(true, U((i % 256) as u8, (i % 256) as u8), (0, 0, 0))
        //         } else {
        //             Cell::new(false, U(0, 0), (255, 255, 255))
        //         }
        //     })
        //     .collect();
       
        //Now begin to ignore is alive and focus on the U values that allow for convection
        let cells = (0..width * height)
            .map(|i| {
                Cell::new(true, U(0 as u8, (i / (height)) as u8), (0, 0, 0))
                // if i % width == 0  {
                //     Cell::new(true, U((255) as u8, (0) as u8), (0, 0, 0))
                // } else {
                //     Cell::new(true, U(0, 0), (255, 255, 255))
                // }
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



