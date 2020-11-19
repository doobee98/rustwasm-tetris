extern crate wasm_bindgen;
extern crate rand;

mod utils;

use rand::Rng;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty = 0,
    Wall = 1,
    SkyBlue = 2,
    Blue = 3,
    Orange = 4,
    Yellow = 5,
    Green = 6,
    Purple = 7,
    Red = 8,
}

#[wasm_bindgen]
pub struct Point(u32, u32);

#[wasm_bindgen]
impl Point {
    pub fn row(&self) -> u32 { self.0 }
    pub fn col(&self) -> u32 { self.1 }
    
}

impl Clone for Point {
    fn clone(&self) -> Point { Point(self.0, self.1) }
}

#[wasm_bindgen]
pub struct Block {
    size: u32,
    color: Cell,
    shape: Vec<Point>,
}

#[wasm_bindgen]
impl Block {
    pub fn kindCount() -> u32 { 7 }

    pub fn getSize(&self) -> u32 { self.size }
    pub fn getColor(&self) -> Cell { self.color }
    pub fn getShape(&self) -> *const Point { self.shape.as_ptr() }

    pub fn new(kind: u32) -> Block {
        assert!(kind < Block::kindCount());

        let (color, shape) = (
            match kind {
                | 0 => (Cell::SkyBlue, vec![
                    Point(0, 1),
                    Point(1, 1),
                    Point(2, 1),
                    Point(3, 1),
                ]),
                | 1 => (Cell::Blue, vec![
                    Point(1, 0),
                    Point(1, 1),
                    Point(1, 2),
                    Point(2, 2),
                ]),
                | 2 => (Cell::Orange, vec![
                    Point(2, 1),
                    Point(1, 1),
                    Point(1, 2),
                    Point(1, 3),
                ]),
                | 3 => (Cell::Yellow, vec![
                    Point(1, 1),
                    Point(1, 2),
                    Point(2, 1),
                    Point(2, 2),
                ]),
                | 4 => (Cell::Green, vec![
                    Point(2, 0),
                    Point(2, 1),
                    Point(1, 1),
                    Point(1, 2),
                ]),
                | 5 => (Cell::Purple, vec![
                    Point(1, 1),
                    Point(1, 2),
                    Point(1, 3),
                    Point(2, 2),
                ]),
                | 6 => (Cell::Red, vec![
                    Point(1, 1),
                    Point(1, 2),
                    Point(2, 2),
                    Point(2, 3),
                ]),
                | _ => panic!(""),
            }
        );

        Block {
            size: 4,
            color,
            shape,
        }
    }

    pub fn new_rand() -> Block {
        let rand_kind = rand::thread_rng().gen_range(0, Block::kindCount());
        // let rand_kind = 1;
        Block::new(rand_kind)
    }

    pub fn turn(&self) -> Block {
        Block {
            shape: self.shape.iter()
                    .map(|p| Point(p.col(), self.size - 1 - p.row()))
                    .collect(),
            ..(*self)
        }
    }
}

impl Clone for Block {
    fn clone(&self) -> Block {
        Block {
            shape: self.shape.iter()
                    .map(|p| Point(p.row(), p.col()))
                    .collect(),
            ..(*self)
        }
    }
}


/*
    1. turn 구현
    2. direction 여부
    3. tick, 블록 움직이기, 가능여부 등
*/


#[wasm_bindgen]
pub struct Board {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    currentBlockPos: Point,
    currentBlock: Block,
    nextBlock: Block,
    keepingBlock: Option<Block>
}

#[wasm_bindgen] 
impl Board {
    pub fn new() -> Board {
        let width = 12;
        let height = 30;
        
        let cells = (0..width * (height - 1))
            .map(|col| {
                if col % width == 0 || col % width == width - 1 {
                    Cell::Wall
                }
                else {
                    Cell::Empty
                }
            })
            .chain(vec![Cell::Wall; width as usize])
            .collect();
        
        return Board { 
            width, 
            height, 
            cells,
            currentBlockPos: Point(0, width / 2 - 2),
            currentBlock: Block::new_rand(),
            nextBlock: Block::new_rand(),
            keepingBlock: None,
        };
    }

    pub fn getWidth(&self) -> u32 { self.width }
    pub fn getHeight(&self) -> u32 { self.height }
    pub fn getCurrentBlock(&self) -> Block { self.currentBlock.clone() }
    pub fn getCurrentBlockPos(&self) -> Point { self.currentBlockPos.clone() }
    pub fn getNextBlock(&self) -> Block { self.nextBlock.clone() }
    pub fn getKeepingBlock(&self) -> Option<Block> { self.keepingBlock.clone() }
    pub fn getCells(&self) -> *const Cell { self.cells.as_ptr() }
    pub fn getCell(&self, row: u32, col: u32) -> Cell {
        self.checkValidPos(row, col);
        return self.cells[self.getIndex(row, col)];
    }

    pub fn tick(&mut self) {
        if !self.moveCurrentBlock(Dir::Down) {
            self.next();
        }
    }

    pub fn moveCurrentBlock(&mut self, dir: Dir) -> bool {
        let current_row = self.currentBlockPos.row();
        let current_col = self.currentBlockPos.col();
        let (new_row, new_col) = match dir {
            Dir::Up => (current_row - 1, current_col),
            Dir::Down => (current_row + 1, current_col),
            Dir::Left => (current_row, current_col - 1),
            Dir::Right => (current_row, current_col + 1),
        };
        if self.enableBlockPos(&self.currentBlock, new_row, new_col) {
            self.currentBlockPos = Point(new_row, new_col);
            return true;
        }
        else {
            return false;
        }
    }

    pub fn dropCurrentBlock(&mut self) {
        while self.moveCurrentBlock(Dir::Down) {};
        self.next();
    }

    pub fn turnCurrentBlock(&mut self) -> bool {
        let current_row = self.currentBlockPos.row();
        let current_col = self.currentBlockPos.col();
        let turned_block = self.currentBlock.turn();

        if self.enableBlockPos(&turned_block, current_row, current_col) {
            self.currentBlock = turned_block;
            return true;
        }
        else {
            return false;
        }
    }

    pub fn keepCurrentBlock(&mut self) {
        let current_row = self.currentBlockPos.row();
        let current_col = self.currentBlockPos.col();
        let next_block = match &self.keepingBlock {
            | Some(b) => b.clone(),
            | None => {
                let temp = self.nextBlock.clone();
                self.nextBlock = Block::new_rand();
                temp
            },
        };
        self.keepingBlock = Some (self.currentBlock.clone());
        self.currentBlock = next_block;
    }

    pub fn resetCurrentBlockPos(&mut self) {
        self.currentBlockPos = Point(0, self.width / 2 - 2);
    }

    pub fn next(&mut self) {
        let mut next_cells = self.cells.clone();
        for p in &self.currentBlock.shape {
            let row = self.currentBlockPos.row() + p.row();
            let col = self.currentBlockPos.col() + p.col();
            next_cells[self.getIndex(row, col)] = self.currentBlock.color;
        }
        self.cells = next_cells;
        self.clearFilledLine();
        self.resetCurrentBlockPos();
        self.currentBlock = self.nextBlock.clone();     // TODO: clone 말곤 방법 없나?
        self.nextBlock = Block::new_rand();
    }
}

impl Board {
    fn getIndex(&self, row: u32, col: u32) -> usize {
        self.checkValidPos(row, col);
        return (row * self.width + col) as usize;
    }

    fn checkValidPos(&self, row: u32, col: u32) {
        assert!(row < self.height);
        assert!(col < self.width);
    }
    
    fn enableBlockPos(&self, block: &Block, newRow: u32, newCol: u32) -> bool {
        block.shape.iter()
            .map(|p| Point(newRow + p.row(), newCol + p.col()))
            .all(|p| self.getCell(p.row(), p.col()) == Cell::Empty)
    }

    fn clearFilledLine(&mut self) -> bool {
        let mut new_cells = self.cells.clone();
        
        let mut new_row = self.height - 1;
        for row in (0..self.height-1).rev() {
            let start = self.getIndex(row, 0);
            let end = self.getIndex(row, self.width-1);
            let filled = self.cells[start+1..end].iter()
                            .all(|&cell| cell != Cell::Empty);
            if !filled {
                new_row -= 1;
                let cur_start = self.getIndex(new_row, 0);
                for col in 1..self.width-1 {
                    let _col = col as usize;
                    new_cells[cur_start+_col] = self.cells[start+_col];
                }
            }
        }

        if new_row == 0 {
            return false;
        }

        else {
            for row in 0..new_row {
                let start = self.getIndex(row, 0);
                for col in 1..self.width-1 {
                    let _col = col as usize;
                    new_cells[start+_col] = Cell::Empty;
                }
            }
            assert_eq!(self.cells.len(), new_cells.len());
            self.cells = new_cells;

            return true;
        }
    }
}
