// Could Hashlife be a useful way of implementing a cfd simulator?
// https://en.wikipedia.org/wiki/Hashlife

import { Universe, Cell }  from "../pkg/cfd_simulator";
import { memory } from "../pkg/cfd_simulator_bg.wasm";

const CELL_SIZE = 5; // px
const GRID_colour = "#CCCCCC";
const DEAD_colour = "#FFFFFF";
const ALIVE_colour = "rgba(150, 0, 0, 1)";


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


const renderLoop = () => {


  const fps = 15;
  universe.tick();


  // drawGrid();
  drawCells();


  setTimeout(() => {
    requestAnimationFrame(renderLoop);
  }, 1000 / fps);


};


// const drawGrid = () => {
//     ctx.beginPath();
//     ctx.strokeStyle = GRID_colour;
 
//     // Vertical lines.
//     for (let i = 0; i <= width; i++) {
//       ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
//       ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
//     }
 
//     // Horizontal lines.
//     for (let j = 0; j <= height; j++) {
//       ctx.moveTo(0,                           j * (CELL_SIZE + 1) + 1);
//       ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
//     }
 
//     ctx.stroke();
// };


const getIndex = (row, column) => {
return row * width + column;
};
 
const drawCells = () => {
  const cellsPtr = universe.cells();
  const cells = new Uint8Array(memory.buffer, cellsPtr, width * height * 5);


  ctx.beginPath();


  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col) * 6;
      const isAlive = cells[idx] === 1;  
      const uX = cells[idx + 1];
      const uY = cells[idx + 2];
      const colourR = cells[idx + 3];    
      const colourG = cells[idx + 4];  
      const colourB = cells[idx + 5];


      // ctx.fillStyle = !isAlive
      // ? `rgb(255, 255, 255, 1)`
      // : `rgb(${colourR}, ${colourG}, ${colourB}, 1)`;
      const shade = 255 - ((uX + uY)*2);


      ctx.fillStyle = `rgb(${(shade)}, ${shade}, ${shade}, 1)`;
      ctx.strokeStyle = `rgb(${(shade)}, ${shade}, ${shade}, 1)`;


      ctx.fillRect(
          col * (CELL_SIZE ) + 1,
          row * (CELL_SIZE ) + 1,
          CELL_SIZE,
          CELL_SIZE
      );


      // console.log(`Row: ${row}, Col: ${col}, uX: ${uX}, uY: ${uY}, isAlive: ${isAlive}, Shade: ${255 - (uX + uY)}`);


    }
  }


  ctx.stroke();
};




// drawGrid();
drawCells();
requestAnimationFrame(renderLoop);



