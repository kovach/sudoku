// implementation of http://norvig.com/sudoku.html
use std::ops::{Index, IndexMut};
use std::io::prelude::*;
use std::io;
use std::fs::File;

const PRINT: bool = true;

type Cell = usize;
type Value = usize;
type CellSet = Vec<Cell>;
type ValueSet = Vec<Value>;
type UnitList = Vec<Vec<CellSet>>;
type PeerList = Vec<CellSet>;
#[derive(Debug, Clone)]
struct Problem {
    units: UnitList,
    peers: PeerList,
}
#[derive(Debug, Clone)]
struct Board {
    constraints: Vec<ValueSet>,
}
fn delete(set: &mut Vec<Value>, v: Value) {
    if let Some(i) = set.iter().position(|&e| e == v) {
        set.remove(i);
    }
}

impl Index<usize> for Board {
    type Output = Vec<Value>;
    fn index(&self, i: usize) -> &Vec<Value> {
        &self.constraints[i]
    }
}
impl IndexMut<usize> for Board {
    fn index_mut(&mut self, i: usize) -> &mut Vec<Value> {
        &mut self.constraints[i]
    }
}

enum Outcome {
    Done,
    Failed,
    Next(Cell),
}

impl Board {
    fn new() -> Board {
        fn any() -> CellSet {
            (1..10).collect()
        }
        Board { constraints: vec![any(); 81] }
    }

    // If a cell has 0 options -> Failed
    // If all cells have 1 option -> Done
    // Otherwise, -> index of undetermined cell with least options
    fn solved(&self) -> Outcome {
        let mut cell = 81;
        let mut best = 10;
        for (i, constraint) in self.constraints.iter().enumerate() {
            let l = constraint.len();
            if l == 0 {
                return Outcome::Failed;
            }
            if l > 1 && l < best {
                cell = i;
                best = l;
            }
        }
        if cell == 81 {
            return Outcome::Done;
        }
        return Outcome::Next(cell as usize);
    }

    fn assign(&mut self, pr: &Problem, c: Cell, v: Value) -> bool {
        let mut cs = self[c].clone();
        delete(&mut cs, v);
        for v in &cs {
            if !self.eliminate(pr, c, *v) {
                return false;
            }
        }
        true
    }

    fn eliminate(&mut self, pr: &Problem, c: Cell, v: Value) -> bool {
        if !(self[c].contains(&v)) {
            return true;
        }
        delete(&mut self[c], v);
        let others = self[c].clone();
        if others.len() == 0 {
            return false;
        }
        if others.len() == 1 {
            let val = others[0];
            for peer in &pr.peers[c] {
                if *peer != c {
                    if !self.eliminate(pr, *peer, val) {
                        return false;
                    }
                }
            }
        }
        for u in &pr.units[c] {
            let mut places = Vec::new();
            for cell in u {
                if self[*cell].contains(&v) {
                    places.push(*cell);
                }
            }
            match places.len() {
                0 => return false,
                1 => {
                    if !self.assign(pr, places[0], v) {
                        return false;
                    }
                }
                _ => continue,
            }
        }

        true
    }

    // Tries to solve the current board
    fn search(&mut self, pr: &Problem) -> Option<Board> {
        match self.solved() {
            Outcome::Failed => return None,
            Outcome::Done => return Some(self.clone()),
            // Returns the cell with smallest number of possibilities
            Outcome::Next(c) => {
                for v in &self[c] {
                    let mut new = self.clone();
                    //println!("trying {}:{}", c, *v);
                    if new.assign(pr, c, *v) {
                        if let Some(b) = new.search(pr) {
                            return Some(b);
                        }
                    }
                }
                return None;
            }
        }
    }
}

// Initializes sets representing the basic constraints of a sudoku puzzle
fn make_units() -> Problem {
    let mut unit_set: Vec<CellSet> = Vec::new();
    // columns
    for col in 0..9 {
        let mut a = Vec::new();
        for row in 0..9 {
            a.push(row * 9 + col);
        }
        unit_set.push(a)
    }
    // rows
    for row in 0..9 {
        let mut a = Vec::new();
        for col in 0..9 {
            a.push(row * 9 + col);
        }
        unit_set.push(a)
    }
    // boxes
    for rs in &[[0, 1, 2], [3, 4, 5], [6, 7, 8]] {
        for cs in &[[0, 1, 2], [3, 4, 5], [6, 7, 8]] {
            let mut a = Vec::new();
            for r in rs {
                for c in cs {
                    a.push((r * 9 + c) as usize);
                }
            }
            unit_set.push(a);
        }
    }

    let mut units: UnitList = vec![Vec::new(); 81];
    let mut peers: PeerList = Vec::new();

    for u in &unit_set {
        // Attach `u` to each cell it contains
        for cell in u.iter() {
            units[*cell as usize].push(u.clone());
        }
    }
    // Merge unit set units[i] together into peers[i]
    for us in &units {
        let mut acc = Vec::new();
        for u in us {
            // .iter.cloned() ?
            acc.extend(u.clone());
        }
        peers.push(acc);
    }

    Problem {
        peers: peers,
        units: units,
    }
}

fn load(file: &str) -> Result<Vec<String>, io::Error> {
    let mut f = try!(File::open(file));
    let mut lines = String::new();
    try!(f.read_to_string(&mut lines));
    Ok(lines.split("\n").map(|x| x.to_string()).collect())
}


fn render_board(b: &Board) -> Vec<String> {
    let mut res = vec![String::new(); 81];
    for (i, constraint) in b.constraints.iter().enumerate() {
        if constraint.len() == 1 {
            res[i] = format!("{}", constraint[0]);
        } else {
            res[i] = format!("*{}", constraint.len());
        }
    }
    res
}

fn solve_puzzle(s: &str) -> Option<Board> {
    let pr = make_units();
    let mut b = Board::new();
    let mut current = -1;
    if PRINT {
        println!("puzzle: {}", s);
    }
    for c in s.chars() {
        if c == '\n' {
            continue;
        }
        current += 1;
        if c == '0' || c == '.' {
            continue;
        }
        if let Some(d) = c.to_digit(10) {
            if !b.assign(&pr, current as usize, d as usize) {
                return None;
            }
        }
    }
    b.search(&pr)
}

fn main() {
    // http://norvig.com/top95.txt
    let file = "top95.txt";
    let strs = load(file).expect("couldn't load file");
    for (i, line) in strs.iter().enumerate() {
        if line.len() < 81 {
            continue;
        }
        let b = solve_puzzle(line).expect(&format!("failed to solve: {}", i));
        if PRINT {
            match b.solved() {
                Outcome::Done => println!("solved ({})", i),
                Outcome::Failed => println!("failed? ({})", i),
                Outcome::Next(_) => println!("not done?? ({})", i),
            }
            let pb = render_board(&b);
            for r in 0..9 {
                let row = &pb[r * 9..r * 9 + 9];
                println!("{:?}", row);
            }
        }
    }
}
