// Could Hashlife be a useful way of implementing a cfd simulator?
// https://en.wikipedia.org/wiki/Hashlife

import { Universe, Cell }  from "../pkg/cfd_simulator";
import { memory } from "../pkg/cfd_simulator_bg.wasm";

const CELL_SIZE = 3; // px

const pre = document.getElementById("cfd-canvas");

// Construct the universe, and get its width and height.
const universe = Universe.new();
const width = universe.width();
const height = universe.height();


// Give the canvas room for all of our cells and a 1px border
// around each of them.
const canvas = document.getElementById("cfd-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

const ctx = canvas.getContext('2d');

let frameCount = 0;

const renderLoop = () => {
  const fps = 100;
  universe.tick();

  // drawGrid();
  drawCells();

  // Log divergence and drag coefficient every 30 frames (~2 seconds)
  if (frameCount % 30 === 0) {
    const maxDiv = universe.max_divergence();
    const cd = universe.drag_coefficient();
    console.log(`Frame ${frameCount}: Max divergence = ${maxDiv.toFixed(6)}, C_d = ${cd.toFixed(4)}`);
  }
  frameCount++;

  setTimeout(() => {
    requestAnimationFrame(renderLoop);
  }, 1000 / fps);
};

const getIndex = (row, column) => {
return row * width + column;
};
 
const drawCells = () => {
  const cellsPtr = universe.cells();
  const pressurePtr = universe.pressure();
  const obstaclePtr = universe.obstacle();

  // Cell layout with #[repr(C)]: bool(1) + padding(3) + f32(4) + f32(4) + u8(3) + padding(1) = 16 bytes
  const CELL_SIZE_BYTES = 16;
  const cells = new Uint8Array(memory.buffer, cellsPtr, width * height * CELL_SIZE_BYTES);
  const cellsFloat = new Float32Array(memory.buffer, cellsPtr, width * height * CELL_SIZE_BYTES / 4);
  const pressure = new Float32Array(memory.buffer, pressurePtr, width * height);
  const obstacle = new Uint8Array(memory.buffer, obstaclePtr, width * height);

  ctx.beginPath();

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);
      const byteIdx = idx * CELL_SIZE_BYTES;
      const floatIdx = idx * (CELL_SIZE_BYTES / 4);

      // If obstacle, draw it black
      if (obstacle[idx]) {
        ctx.fillStyle = 'rgb(0, 0, 0)';
        ctx.fillRect(
            col * (CELL_SIZE) + 1,
            row * (CELL_SIZE) + 1,
            CELL_SIZE,
            CELL_SIZE
        );
        continue;
      }

      // Get velocity components
      const uX = cellsFloat[floatIdx + 1];
      const uY = cellsFloat[floatIdx + 2];

      // Velocity magnitude for intensity
      const velMag = Math.sqrt(uX * uX + uY * uY);
      const intensity = Math.min(1.0, velMag / 30.0); // Scale factor

      // Pressure for color (red = high, blue = low)
      const p = pressure[idx];
      const pNormalized = Math.tanh(p / 10.0); // Less sensitive for smoother transitions

      // Map pressure to color with smooth gradients: blue -> cyan -> white -> yellow -> red
      let r, g, b;
      if (pNormalized > 0) {
        // Positive pressure: white -> yellow -> red
        r = 255;
        g = Math.floor(255 * (1 - pNormalized * 0.7)); // Keep more green
        b = Math.floor(255 * (1 - pNormalized));
      } else {
        // Negative pressure: white -> cyan -> blue
        r = Math.floor(255 * (1 + pNormalized));
        g = Math.floor(255 * (1 + pNormalized * 0.7)); // Keep more green
        b = 255;
      }

      // Boost brightness with velocity, but keep minimum visible
      const brightnessFactor = 0.5 + 0.5 * intensity;
      r = Math.floor(r * brightnessFactor);
      g = Math.floor(g * brightnessFactor);
      b = Math.floor(b * brightnessFactor);

      ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;

      ctx.fillRect(
          col * (CELL_SIZE) + 1,
          row * (CELL_SIZE) + 1,
          CELL_SIZE,
          CELL_SIZE
      );
    }
  }

  ctx.stroke();
};

drawCells();
requestAnimationFrame(renderLoop);