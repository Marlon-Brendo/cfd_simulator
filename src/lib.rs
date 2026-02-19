mod utils;


use std::fmt;
use std::ops::Sub;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct U(f32, f32);


#[wasm_bindgen]
impl U {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32) -> U {
        U(x, y)
    }


    pub fn x(&self) -> f32 {
        self.0
    }


    pub fn y(&self) -> f32 {
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
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
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
    pressure: Vec<f32>,  // Pressure field for Navier-Stokes
    obstacle: Vec<bool>, // True if cell is a solid obstacle
}
impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn is_obstacle(&self, row: u32, column: u32) -> bool {
        let idx = self.get_index(row, column);
        self.obstacle[idx]
    }

    fn burgers_step(&self, row: u32, column: u32, u_current: U) -> U {
        let dt = 0.01;
        let dx = 1.0;
        let dy = 1.0;
        let nu = 0.5;  // Kinematic viscosity

        // At boundaries, use zero-gradient (copy from interior neighbor)
        if row == 0 || column == 0 || row == self.height - 1 || column == self.width - 1 {
            let neighbor_row = if row == 0 { 1 } else if row == self.height - 1 { self.height - 2 } else { row };
            let neighbor_col = if column == 0 { 1 } else if column == self.width - 1 { self.width - 2 } else { column };
            let idx = self.get_index(neighbor_row, neighbor_col);
            return self.cells[idx].u;
        }

        // Get velocity components at current cell
        let u = u_current.0;
        let v = u_current.1;

        // Get neighbors
        let u_left = self.cells[self.get_index(row, column - 1)].u;
        let u_right = self.cells[self.get_index(row, column + 1)].u;
        let u_down = self.cells[self.get_index(row - 1, column)].u;
        let u_up = self.cells[self.get_index(row + 1, column)].u;

        // === CONVECTION TERMS (upwind scheme) ===
        let du_dx = if u > 0.0 {
            (u - u_left.0) / dx
        } else {
            (u_right.0 - u) / dx
        };

        let du_dy = if v > 0.0 {
            (u - u_down.0) / dy
        } else {
            (u_up.0 - u) / dy
        };

        let dv_dx = if u > 0.0 {
            (v - u_left.1) / dx
        } else {
            (u_right.1 - v) / dx
        };

        let dv_dy = if v > 0.0 {
            (v - u_down.1) / dy
        } else {
            (u_up.1 - v) / dy
        };

        let convection_u = u * du_dx + v * du_dy;
        let convection_v = u * dv_dx + v * dv_dy;

        // === DIFFUSION TERMS (central difference, Laplacian) ===
        // ∂²u/∂x² = (u[i+1] - 2*u[i] + u[i-1]) / dx²
        let d2u_dx2 = (u_right.0 - 2.0 * u + u_left.0) / (dx * dx);
        let d2u_dy2 = (u_up.0 - 2.0 * u + u_down.0) / (dy * dy);
        let diffusion_u = nu * (d2u_dx2 + d2u_dy2);

        let d2v_dx2 = (u_right.1 - 2.0 * v + u_left.1) / (dx * dx);
        let d2v_dy2 = (u_up.1 - 2.0 * v + u_down.1) / (dy * dy);
        let diffusion_v = nu * (d2v_dx2 + d2v_dy2);

        // === TIME INTEGRATION (Forward Euler) ===
        // du/dt = -convection + diffusion
        let u_new = u + dt * (-convection_u + diffusion_u);
        let v_new = v + dt * (-convection_v + diffusion_v);

        U(u_new, v_new)
    }

    fn solve_pressure_poisson(&mut self, iterations: u32) {
        // Solve ∇²p = ρ/dt * ∇·u using Jacobi iteration
        let dx = 1.0;
        let dy = 1.0;
        let dt = 0.01;
        let rho = 1.0;

        let mut p_new = self.pressure.clone();

        for _ in 0..iterations {
            for row in 1..(self.height - 1) {
                for col in 1..(self.width - 1) {
                    let idx = self.get_index(row, col);

                    // Skip obstacles - they have zero pressure gradient
                    if self.obstacle[idx] {
                        p_new[idx] = 0.0;
                        continue;
                    }

                    // Compute divergence: ∂u/∂x + ∂v/∂y
                    let u_right = self.cells[self.get_index(row, col + 1)].u;
                    let u_left = self.cells[self.get_index(row, col - 1)].u;
                    let u_up = self.cells[self.get_index(row + 1, col)].u;
                    let u_down = self.cells[self.get_index(row - 1, col)].u;

                    let du_dx = (u_right.0 - u_left.0) / (2.0 * dx);
                    let dv_dy = (u_up.1 - u_down.1) / (2.0 * dy);
                    let div_u = du_dx + dv_dy;

                    // Jacobi iteration for Laplacian
                    let p_right = self.pressure[self.get_index(row, col + 1)];
                    let p_left = self.pressure[self.get_index(row, col - 1)];
                    let p_up = self.pressure[self.get_index(row + 1, col)];
                    let p_down = self.pressure[self.get_index(row - 1, col)];

                    p_new[idx] = ((p_right + p_left) * dy * dy + (p_up + p_down) * dx * dx
                                  - rho * dx * dx * dy * dy * div_u / dt)
                                 / (2.0 * (dx * dx + dy * dy));
                }
            }
            self.pressure = p_new.clone();
        }

        // Boundary conditions
        // Left boundary (inlet): zero pressure
        for row in 0..self.height {
            let idx_left = self.get_index(row, 0);
            p_new[idx_left] = 0.0;

            // Right boundary (outlet): zero gradient
            let idx_right = self.get_index(row, self.width - 1);
            let idx_right_in = self.get_index(row, self.width - 2);
            p_new[idx_right] = p_new[idx_right_in];
        }
        // Top and bottom boundaries: zero gradient
        for col in 0..self.width {
            let idx_top = self.get_index(0, col);
            let idx_top_in = self.get_index(1, col);
            p_new[idx_top] = p_new[idx_top_in];

            let idx_bottom = self.get_index(self.height - 1, col);
            let idx_bottom_in = self.get_index(self.height - 2, col);
            p_new[idx_bottom] = p_new[idx_bottom_in];
        }
        self.pressure = p_new;
    }

    fn navier_stokes_step(&self, row: u32, column: u32, u_current: U) -> U {
        let dt = 0.01;
        let dx = 1.0;
        let dy = 1.0;
        let nu = 0.5;
        let rho = 1.0;

        // If this cell is an obstacle, velocity is zero (no-slip)
        if self.is_obstacle(row, column) {
            return U(0.0, 0.0);
        }

        // Boundary conditions
        // Left boundary: inlet with parabolic velocity profile
        if column == 0 {
            let y = row as f32;
            let h = self.height as f32;
            let y_normalized = y / h;
            let max_velocity = 30.0;
            let u_velocity = max_velocity * 4.0 * y_normalized * (1.0 - y_normalized);
            return U(u_velocity, 0.0);
        }

        // Top, bottom, right boundaries: zero-gradient (outlet)
        if row == 0 || row == self.height - 1 || column == self.width - 1 {
            let neighbor_row = if row == 0 { 1 } else if row == self.height - 1 { self.height - 2 } else { row };
            let neighbor_col = if column == self.width - 1 { self.width - 2 } else { column };
            let idx = self.get_index(neighbor_row, neighbor_col);
            return self.cells[idx].u;
        }

        let u = u_current.0;
        let v = u_current.1;

        // Get neighbors
        let u_left = self.cells[self.get_index(row, column - 1)].u;
        let u_right = self.cells[self.get_index(row, column + 1)].u;
        let u_down = self.cells[self.get_index(row - 1, column)].u;
        let u_up = self.cells[self.get_index(row + 1, column)].u;

        // Convection (upwind)
        let du_dx = if u > 0.0 { (u - u_left.0) / dx } else { (u_right.0 - u) / dx };
        let du_dy = if v > 0.0 { (u - u_down.0) / dy } else { (u_up.0 - u) / dy };
        let dv_dx = if u > 0.0 { (v - u_left.1) / dx } else { (u_right.1 - v) / dx };
        let dv_dy = if v > 0.0 { (v - u_down.1) / dy } else { (u_up.1 - v) / dy };

        let convection_u = u * du_dx + v * du_dy;
        let convection_v = u * dv_dx + v * dv_dy;

        // Diffusion (Laplacian)
        let d2u_dx2 = (u_right.0 - 2.0 * u + u_left.0) / (dx * dx);
        let d2u_dy2 = (u_up.0 - 2.0 * u + u_down.0) / (dy * dy);
        let diffusion_u = nu * (d2u_dx2 + d2u_dy2);

        let d2v_dx2 = (u_right.1 - 2.0 * v + u_left.1) / (dx * dx);
        let d2v_dy2 = (u_up.1 - 2.0 * v + u_down.1) / (dy * dy);
        let diffusion_v = nu * (d2v_dx2 + d2v_dy2);

        // Pressure gradient
        let p_right = self.pressure[self.get_index(row, column + 1)];
        let p_left = self.pressure[self.get_index(row, column - 1)];
        let p_up = self.pressure[self.get_index(row + 1, column)];
        let p_down = self.pressure[self.get_index(row - 1, column)];

        let dp_dx = (p_right - p_left) / (2.0 * dx);
        let dp_dy = (p_up - p_down) / (2.0 * dy);

        // Navier-Stokes: du/dt = -convection + diffusion - ∇p/ρ
        let u_new = u + dt * (-convection_u + diffusion_u - dp_dx / rho);
        let v_new = v + dt * (-convection_v + diffusion_v - dp_dy / rho);

        U(u_new, v_new)
    }
}


impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell.is_alive {
                    if cell.u.x() > 64.0 { '◼' } else { '◻' }
                } else {
                    ' '
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
        // Solve pressure Poisson equation to enforce incompressibility
        self.solve_pressure_poisson(50);

        // Update velocity field using Navier-Stokes
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let next_cell = Cell::new(true, self.navier_stokes_step(row, col, cell.u), (0, 0, 0));
                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }


    pub fn new() -> Universe {
        utils::set_panic_hook();

        let width = 256;
        let height = 256;

        // Channel flow: parabolic velocity profile left to right
        // Faster in the center, slower near top/bottom (like pipe flow)
        let max_velocity = 30.0;

        // Cylinder obstacle in the middle
        let obs_center_row = (height / 2) as f32 + 20.0;  // Shifted down
        let obs_center_col = (width / 2) as f32;
        let obs_radius = 20.0;  // Cylinder radius (doubled for doubled grid)

        let cells = (0..width * height)
            .map(|i| {
                let row = i / width;
                let col = i % width;

                let y = row as f32;
                let h = height as f32;

                // Parabolic profile: u(y) = U_max * 4 * (y/h) * (1 - y/h)
                // Maximum at center (y = h/2), zero at walls (y = 0, y = h)
                let y_normalized = y / h;
                let u_velocity = max_velocity * 4.0 * y_normalized * (1.0 - y_normalized);

                // All flow is left to right (u component), no vertical flow (v = 0)
                Cell::new(true, U(u_velocity, 0.0), (0, 0, 0))
            })
            .collect();

        let obstacle = (0..width * height)
            .map(|i| {
                let row = i / width;
                let col = i % width;

                // Circular cylinder centered in the domain
                let dy = row as f32 - obs_center_row;
                let dx = col as f32 - obs_center_col;
                let distance = (dx * dx + dy * dy).sqrt();

                distance < obs_radius
            })
            .collect();

        let pressure = vec![0.0; (width * height) as usize];

        Universe {
            width,
            height,
            cells,
            pressure,
            obstacle,
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

    pub fn pressure(&self) -> *const f32 {
        self.pressure.as_ptr()
    }

    pub fn obstacle(&self) -> *const bool {
        self.obstacle.as_ptr()
    }

    pub fn max_divergence(&self) -> f32 {
        let dx = 1.0;
        let dy = 1.0;
        let mut max_div = 0.0;

        for row in 1..(self.height - 1) {
            for col in 1..(self.width - 1) {
                let u_right = self.cells[self.get_index(row, col + 1)].u;
                let u_left = self.cells[self.get_index(row, col - 1)].u;
                let u_up = self.cells[self.get_index(row + 1, col)].u;
                let u_down = self.cells[self.get_index(row - 1, col)].u;

                let du_dx = (u_right.0 - u_left.0) / (2.0 * dx);
                let dv_dy = (u_up.1 - u_down.1) / (2.0 * dy);
                let div = (du_dx + dv_dy).abs();

                if div > max_div {
                    max_div = div;
                }
            }
        }
        max_div
    }

    pub fn drag_coefficient(&self) -> f32 {
        // Calculate drag coefficient on the cylinder
        // C_d = F_d / (0.5 * rho * U^2 * A)
        // where F_d is drag force, U is free stream velocity, A is projected area (diameter)

        let rho = 1.0;
        let diameter = 40.0; // 2 * radius (doubled for doubled grid)
        let u_infinity = 30.0; // Max velocity in parabolic profile

        let mut drag_force_x = 0.0;

        // Find fluid cells adjacent to obstacle and integrate pressure forces
        for row in 1..(self.height - 1) {
            for col in 1..(self.width - 1) {
                let idx = self.get_index(row, col);

                // Only look at fluid cells (not obstacle)
                if !self.obstacle[idx] {
                    let p = self.pressure[idx];

                    // Check if this fluid cell is adjacent to obstacle on left (upstream)
                    if col > 0 && self.obstacle[self.get_index(row, col - 1)] {
                        // Pressure pushing on upstream face contributes to drag
                        drag_force_x += p;
                    }

                    // Check if this fluid cell is adjacent to obstacle on right (downstream)
                    if col < self.width - 1 && self.obstacle[self.get_index(row, col + 1)] {
                        // Low pressure on downstream face also contributes to drag
                        drag_force_x -= p;
                    }
                }
            }
        }

        // Normalize and convert to drag coefficient
        // C_d = F_d / (0.5 * rho * U^2 * D)
        let dynamic_pressure = 0.5 * rho * u_infinity * u_infinity * diameter;

        if dynamic_pressure > 0.0 {
            drag_force_x / dynamic_pressure
        } else {
            0.0
        }
    }
}