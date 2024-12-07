use map::{Cell, Map, Position, Terrain};
use std::{
    collections::{BinaryHeap, HashMap},
    f32::INFINITY,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = match args.get(1) {
        Some(name) => name,
        None => {
            eprintln!("Usage: {} <filename>", args[0]);
            std::process::exit(1);
        }
    };
    let input = input::read_file(filename);
    let map = Map::new(&input);

    let mut min_cost = INFINITY;

    for attrs in generate_combinations() {
        let (cost, path) = astar(&map, &attrs);
        if cost < min_cost {
            min_cost = cost;
            println!("{}{}{}{}", attrs[0], attrs[1], attrs[2], path);
        }
    }
}

fn generate_combinations() -> Vec<[i32; 3]> {
    let mut combinations = Vec::new();

    for i in 0..=5 {
        for j in 0..=5 {
            for k in 0..=5 {
                if i + j + k == 10 {
                    combinations.push([i, j, k]);
                }
            }
        }
    }
    combinations
}

fn astar(map: &Map, attrs: &[i32; 3]) -> (f32, String) {
    let mut open_list: BinaryHeap<Cell> = BinaryHeap::new();
    let mut came_from: HashMap<Position, Position> = HashMap::new();
    let mut g_cost: HashMap<Position, f32> = HashMap::new();

    open_list.push(map.start);
    g_cost.insert(map.start.pos, 0.);

    while !open_list.is_empty() {
        let current: Cell = open_list.pop().unwrap();
        if current.pos == map.goal.pos {
            let mut path = vec![];

            let mut current_pos = current.pos;
            while let Some(prev) = came_from.get(&current_pos) {
                path.push(current_pos.came_from_direction(*prev));
                current_pos = *prev;
            }
            return (g_cost[&map.goal.pos], path.iter().rev().collect::<String>());
        }
        // came_from.insert(current.pos, current.tile_cost);
        for neighbour in map.get_neighbours(current.pos).iter() {
            let cost = g_cost[&current.pos] as f32 + movement_cost(&map, current, neighbour, attrs);

            if !g_cost.contains_key(&neighbour.pos) || cost < g_cost[&neighbour.pos] {
                let mut neighbour = neighbour.clone();
                g_cost.insert(neighbour.pos, cost);
                came_from.insert(neighbour.pos, current.pos);
                let f_cost = calc_h_cost(neighbour.pos, map.goal.pos, map.grid.len() as i32) + cost;
                neighbour.f_cost = f_cost;
                neighbour.g_cost = cost;
                open_list.push(neighbour);
            }
        }
    }
    (INFINITY, "".to_string())
}

#[inline(always)]
fn movement_cost(map: &Map, current: Cell, neighbour: &Cell, attrs: &[i32; 3]) -> f32 {
    // consider calc_h_cost(current_pos, goal_pos)
    calc_g_cost(neighbour.tile_cost, neighbour.terrain, attrs)
}

#[inline(always)]
fn calc_g_cost(tile_cost: i32, terrain: Terrain, attrs: &[i32; 3]) -> f32 {
    let idx = match terrain {
        Terrain::Water => attrs[0],
        Terrain::Air => attrs[1],
        Terrain::Earth => attrs[2],
        _ => 0,
    } as usize;
    let factors = [4.0, 3.0, 2.5, 2.0, 1.5, 1.0];
    let factor = factors.get(idx).unwrap_or(&1.0);

    tile_cost as f32 * factor
}

#[inline(always)]
fn calc_h_cost(current_pos: Position, goal_pos: Position, max_step: i32) -> f32 {
    let dist = current_pos.distance(goal_pos) as f32;
    let p = 1. / max_step as f32;
    dist * (1.0 + p)
}

pub mod map {
    use std::cmp::Ordering;

    #[derive(PartialEq, Clone, Copy, PartialOrd, Eq)]
    pub enum Terrain {
        Water,
        Air,
        Earth,
        Start,
        Goal,
    }

    #[derive(PartialEq, Clone, Copy, PartialOrd, Eq, Hash)]
    pub struct Position {
        row: i32,
        col: i32,
    }

    impl Position {
        fn new(row: i32, col: i32) -> Self {
            Position { row, col }
        }

        pub fn distance(&self, other: Position) -> i32 {
            (self.col - other.col).abs() + (self.row - other.row).abs()
        }

        pub fn came_from_direction(&self, prev: Position) -> char {
            match (self.row - prev.row, self.col - prev.col) {
                (-1, 0) => 'U',
                (1, 0) => 'D',
                (0, 1) => 'R',
                (0, -1) => 'L',
                _ => panic!(),
            }
        }
    }

    impl std::fmt::Display for Position {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "({},{})", self.row, self.col)
        }
    }

    #[derive(Clone, Copy, PartialEq, PartialOrd)]
    pub struct Cell {
        pub terrain: Terrain,
        pub tile_cost: i32,
        // pub reach_cost: f32,
        pub g_cost: f32,
        pub f_cost: f32,
        pub pos: Position,
    }

    impl Ord for Cell {
        #[inline(always)]
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            let self_f_cost = self.f_cost as i32;
            let other_f_cost = other.f_cost as i32;
            
            match self_f_cost.cmp(&other_f_cost) {
                Ordering::Equal => {
                    // Only convert g_costs if f_costs are equal
                    let self_g_cost = self.g_cost as i32;
            let other_g_cost = other.g_cost as i32;
                    self_g_cost.cmp(&other_g_cost)
                }
                ordering => ordering
            }
        }
    }

    impl Eq for Cell {
        fn assert_receiver_is_total_eq(&self) {}
    }

    impl std::fmt::Display for Cell {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let c1 = match self.terrain {
                Terrain::Water => 'W',
                Terrain::Air => 'A',
                Terrain::Earth => 'E',
                Terrain::Start => 'M',
                Terrain::Goal => 'G',
            };
            let c2 = char::from_digit(self.tile_cost as u32, 10).unwrap();
            write!(f, "{}{}", c1, c2)
        }
    }

    impl Cell {
        pub fn new(chars: &[u8], row: usize, col: usize) -> Self {
            let terrain: Terrain;
            let tile_cost: i32;
            match chars[0] {
                b'W' => terrain = Terrain::Water,
                b'A' => terrain = Terrain::Air,
                b'E' => terrain = Terrain::Earth,
                b'M' => terrain = Terrain::Start,
                b'G' => terrain = Terrain::Goal,
                _ => panic!("Invalid character in map found!"),
            }
            match chars[1] {
                (b'0'..=b'9') => tile_cost = (chars[1] - b'0') as i32,
                b'M' | b'G' => tile_cost = 0,
                _ => panic!("Invalid character in map found!"),
            }
            Cell {
                terrain,
                tile_cost,
                // reach_cost: 0.,
                g_cost: 0.,
                f_cost: 0.,
                pos: Position::new(row as i32, col as i32),
            }
        }
    }

    pub struct Map {
        pub grid: Vec<Cell>,
        pub height: u32,
        pub width: u32,
        pub start: Cell,
        pub goal: Cell,
    }

    impl Map {
        pub fn new(input: &String) -> Self {
            let mut grid: Vec<Cell> = Vec::new();
            let mut height: u32 = 0;
            let mut start: Cell = Cell::new(&[b'M', b'M'], 0, 0);
            let mut goal: Cell = Cell::new(&[b'M', b'M'], 0, 0);

            for (row, line) in input.lines().enumerate() {
                for (col, chars) in line.as_bytes().chunks(2).enumerate() {
                    let cell = Cell::new(chars, row, col);
                    grid.push(cell);
                    if cell.terrain == Terrain::Start {
                        start = cell;
                    } else if cell.terrain == Terrain::Goal {
                        goal = cell;
                    }
                }
                height += 1;
            }
            Map {
                grid,
                height,
                width: (input.lines().nth(0).unwrap().len() / 2) as u32,
                start,
                goal,
            }
        }

        pub fn get_neighbours(&self, current_pos: Position) -> Vec<Cell> {
            let mut neighbours = Vec::new();

            if current_pos.row - 1 >= 0 {
                neighbours.push(
                    self.grid[(current_pos.row as usize - 1) * self.width as usize
                        + current_pos.col as usize],
                );
            }
            if current_pos.row + 1 < self.height as i32 {
                neighbours.push(
                    self.grid[(current_pos.row as usize + 1) * self.width as usize
                        + current_pos.col as usize],
                );
            }
            if current_pos.col - 1 >= 0 {
                neighbours.push(
                    self.grid[current_pos.row as usize * self.width as usize
                        + current_pos.col as usize
                        - 1],
                );
            }
            if current_pos.col + 1 < self.width as i32 {
                neighbours.push(
                    self.grid[current_pos.row as usize * self.width as usize
                        + current_pos.col as usize
                        + 1],
                );
            }
            neighbours
        }
    }

    impl std::fmt::Display for Map {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut string = String::new();

            for i in 0..self.grid.len() {
                if (i % self.width as usize) == 0 {
                    string += "\n";
                }
                string += &self.grid[i as usize].to_string();
            }
            write!(
                f,
                "height: {}\nwidth: {}\nstart: {}\ngoal: {}\ngrid: {}",
                self.height, self.width, self.start.pos, self.goal.pos, string
            )
        }
    }
}

pub mod input {
    use std::env;
    use std::fs;
    use std::path;

    pub fn read_input() -> String {
        read_file("input.txt")
    }

    pub fn read_example() -> String {
        read_file("input_example.txt")
    }

    pub fn read_file(filename: &str) -> String {
        let dir = match env::var("CARGO_MANIFEST_DIR") {
            Ok(dir) => path::PathBuf::from(dir),
            Err(_) => env::current_dir().expect("Failed to get current directory"),
        };
        let path = dir.join(filename);
        fs::read_to_string(&path).expect(&format!("Failed to read file {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use input::read_example;
    use input::read_input;

    #[test]
    fn test_read_input() {
        let result = read_input();
        println!("{}", result);
    }

    #[test]
    fn test_read_example() {
        let result = read_example();
        println!("{}", result);
    }
}
