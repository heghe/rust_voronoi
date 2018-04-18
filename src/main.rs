extern crate clap;
extern crate image;
extern crate rand;
extern crate hsl;

use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::VecDeque;
use clap::{Arg, App};
use rand::Rng;
use hsl::HSL;

const SCALE_SIZE:u32 = 16;

#[derive(Clone, Copy)]
struct Point {
    x: usize,
    y: usize,
}

#[derive(Clone, Copy)]
struct Tile {
    id: usize,
    position: Point,
    seed_position: Point,
}

impl Point {
    fn from_string(string: &str) -> Point {
        let values = string.trim().split_whitespace().collect::<Vec<&str>>();
        let x = values.get(0).unwrap().parse::<usize>().unwrap();
        let y = values.get(1).unwrap().parse::<usize>().unwrap();
        Point{x, y}
    }

    fn new(x: usize, y: usize) -> Point {
        Point{x,y}
    }

    fn distance(&self, point: &Point) -> f64 {
        let x: f64 = (self.x as f64) - (point.x as f64);
        let x = x * x;
        let y: f64 = (self.y as f64) - (point.y as f64);
        let y = y * y;
        let s: f64 = x + y;
        s.sqrt()
    }
}

impl Tile {
    fn new(position: Point) -> Tile {
        Tile{id: 0, position: position, seed_position: Point::new(0,0)}
    }

    fn closer_seed(&self, _seed_position: &Point) -> bool {
        self.position.distance(&self.seed_position) <=
            self.position.distance(&_seed_position)
    }
}

fn point_in_space_dimension(point: &Point, i:isize, j:isize, space_size: &Point) -> Option<(usize, usize)> {
    let mut x:usize = point.x;
    let mut y:usize = point.y;

    match i {
        -1 => {
            if x == 0 {
                return None
            }
            else {
                x -= 1;
            }
        }
        1 => {
            if x == space_size.x - 1 {
                return None
            }
            else {
                x += 1;
            }
        }
        _ => {}
    }

    match j {
        -1 => {
            if y == 0 {
                return None
            }
            else {
                y -= 1;
            }
        }
        1 => {
            if y == space_size.y -1 {
                return None
            }
            else {
                y += 1;
            }
        }
        _ => {}
    }

    Some((x,y))
}

fn generate_colors(n:usize) -> Vec<[u8; 3]> {
    let mut colors:Vec<[u8; 3]> = Vec::new();
    let mut rng = rand::thread_rng();
    let mut i:f64 = 0.0;
    let step:f64 = 360.0 / (n as f64);
    while i < 360.0 {
        let h:f64 = i;
        let s:f64 = 0.9 + rng.gen_range(0.0, 0.1);
        let l:f64 = 0.5 + rng.gen_range(0.0, 0.1);

        let hsl_color = HSL{h,s,l};
        let rgb_color = hsl_color.to_rgb();

        i += step;
        colors.push([rgb_color.0, rgb_color.1, rgb_color.2]);
    }
    colors
}

fn make_image(space: &Vec<Vec<Tile>>, space_size: &Point, colors: &Vec<[u8; 3]>,
              filename: &String) {
    let mut image_buffer = image::ImageBuffer::new(SCALE_SIZE * space_size.x as u32,
                                                   SCALE_SIZE * space_size.y as u32);
    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
        let index = space.get((x / SCALE_SIZE) as usize).unwrap()
            .get((y / SCALE_SIZE) as usize).unwrap().id;
        let values:&[u8; 3];
        if index == 0 {
            values = &[0 as u8, 0 as u8, 0 as u8];
        }
        else {
            values = colors.get(index - 1).unwrap();
        }
        *pixel = image::Rgb([values[0], values[1], values[2]]);
    }

    let ref mut fout = File::create(filename).unwrap();
    image::ImageRgb8(image_buffer).save(fout, image::PNG).unwrap();
}

fn main() {
    let matches = App::new("Generating voronoi diagram - sequential")
                            .version("1.0")
                            .author("Heghedus Razvan <heghedus.razvan@gmail.com>")
                            .arg(Arg::with_name("INPUT")
                                 .help("Input set file")
                                 .required(true)
                                 .index(1))
                            .get_matches();
    let filename = format!("data/{}", matches.value_of("INPUT").unwrap());
    println!("Using data file: {}", filename);
    // TODO more comments

    let mut file = BufReader::new(File::open(filename).unwrap());

    let mut line = String::new();
    // read X, Y
    file.read_line(&mut line).unwrap();
    let max_size = Point::from_string(&line);

    let mut space:Vec<Vec<Tile>>  = Vec::with_capacity(max_size.x);
    // creating application space dimension
    for i in 0..max_size.x {
        space.push(Vec::with_capacity(max_size.y));
        for j in 0..max_size.y {
            space.get_mut(i).unwrap().push(Tile::new(Point::new(i,j)));
        }
    }

    // read n
    line.clear();
    file.read_line(&mut line).unwrap();
    let n: usize = line.trim().parse::<usize>().unwrap();

    let mut queue:VecDeque<Point> = VecDeque::new();

    for (i, line) in file.lines().enumerate() {
        let position = Point::from_string(&line.unwrap());
        let mut tile = space.get_mut(position.x).unwrap().get_mut(position.y).unwrap();
        tile.id = i + 1;
        tile.seed_position = position;
        queue.push_back(position);
    }
    // directory
    let output_debug_directory = format!("output/debug_{}",
                                         matches.value_of("INPUT").unwrap());
    fs::create_dir_all(&output_debug_directory);

    //generate n colors
    let colors = generate_colors(n);

    let directions = vec![-1, 0, 1];
    // print initial set
    make_image(&space, &max_size, &colors, &format!("{}/0.png", &output_debug_directory));

    let mut step = 1;
    let mut last_id = 1;

    while !queue.is_empty() {
        let position = queue.pop_front().unwrap();
        let current_tile = space.get(position.x).unwrap().get(position.y).unwrap().clone();

        if last_id != current_tile.id {
            let output_debug_filename = format!("{}/{}.png", output_debug_directory, step);
            make_image(&space, &max_size, &colors, &output_debug_filename);
            step += 1;
            last_id = current_tile.id;
        }
        for i in &directions {
            for j in &directions {
                match point_in_space_dimension(&position, *i, *j, &max_size) {
                    // TODO move the below code in a separate function
                    Some((x,y)) => {
                        let mut next_tile = space.get_mut(x).unwrap()
                                            .get_mut(y).unwrap();
                        if current_tile.id != next_tile.id {
                            if next_tile.id == 0 {
                                next_tile.id = current_tile.id;
                                next_tile.seed_position = current_tile.seed_position;
                                queue.push_back(next_tile.position);
                            }
                            //else if next_tile.id > current_tile.id {
                            else {
                                if !next_tile.closer_seed(&current_tile.seed_position) {
                                    next_tile.id = current_tile.id;
                                    next_tile.seed_position = current_tile.seed_position;
                                    queue.push_back(next_tile.position);
                                }
                            }
                        }
                    }
                    None => {}
                }
            }
        }
    }

    // also write this to a file
    for line in &space {
        for tile in line {
            print!("{} ", tile.id);
        }
        println!();
    }

    for color in &colors {
        println!("{} {} {}", color[0], color[1], color[2]);
    }

    let output_filename = format!("output/{}.png", matches.value_of("INPUT").unwrap());
    make_image(&space, &max_size, &colors, &output_filename);
}
