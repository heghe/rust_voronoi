#[derive(Clone, Copy)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn from_string(string: &str) -> Point {
        let values = string.trim().split_whitespace().collect::<Vec<&str>>();
        let x = values.get(0).unwrap().parse::<usize>().unwrap();
        let y = values.get(1).unwrap().parse::<usize>().unwrap();
        Point { x, y }
    }

    pub fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }

    pub fn distance(&self, point: &Point) -> f64 {
        let x: f64 = (self.x as f64) - (point.x as f64);
        let x = x * x;
        let y: f64 = (self.y as f64) - (point.y as f64);
        let y = y * y;
        let s: f64 = x + y;
        s.sqrt()
    }
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub id: usize,
    pub position: Point,
    pub seed_position: Point,
}

impl Tile {
    pub fn new(position: Point) -> Tile {
        Tile {
            id: 0,
            position: position,
            seed_position: Point::new(0, 0),
        }
    }

    pub fn closer_seed(&self, _seed_position: &Point) -> bool {
        self.position.distance(&self.seed_position) <= self.position.distance(&_seed_position)
    }
}

pub struct ApplicationState {
    pub seeds: Vec<Point>,
    pub space_size: Point,
    pub debug_enabled: bool,
    pub output_debug_directory: String,
    pub colors: Vec<[u8; 3]>,
    pub output_filename: String,
    pub image_only: bool,
}

pub fn point_bounderies(
    point: &Point,
    i: isize,
    j: isize,
    space_size: &Point,
) -> Option<(usize, usize)> {
    let x: isize = point.x as isize + i;
    let y: isize = point.y as isize + j;

    if x < 0 || x > space_size.x as isize - 1 {
        return None;
    }
    if y < 0 || y > space_size.y as isize - 1 {
        return None;
    }

    Some((x as usize, y as usize))
}
