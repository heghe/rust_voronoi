extern crate clap;
extern crate image;
extern crate rand;
extern crate hsl;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::VecDeque;
use clap::{Arg, App};
use image::{GenericImage, ImageBuffer};
use rand::Rng;
use hsl::HSL;

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
}

impl Tile {
    fn new(position: Point) -> Tile {
        Tile{id: 0, position: position, seed_position: Point::new(0,0)}
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
        tile.id = i+1;
        tile.seed_position = position;
        queue.push_back(position);
    }

    let directions = vec![-1, 0, 1];

    while !queue.is_empty() {
        let position = queue.pop_front().unwrap();
        let current_tile = space.get(position.x).unwrap().get(position.y).unwrap().clone();
        for i in &directions {
            for j in &directions {
                match point_in_space_dimension(&position, *i, *j, &max_size) {
                    Some((x,y)) => {
                        let mut tile = space.get_mut(x).unwrap()
                                            .get_mut(y).unwrap();
                        if current_tile.id != tile.id {
                            if tile.id == 0 {
                                tile.id = current_tile.id;
                                tile.seed_position = current_tile.seed_position;
                                queue.push_back(tile.position);
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
            print!("{}", tile.id);
        }
        println!();
    }

    let colors = generate_colors(n);
    for color in &colors {
        println!("{} {} {}", color[0], color[1], color[2]);
    }
    let mut image_buffer = image::ImageBuffer::new(max_size.x as u32, max_size.y as u32);
    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
        let index = space.get(x as usize).unwrap().get(y as usize).unwrap().id - 1;
        let values = colors.get(index).unwrap();
        *pixel = image::Rgb([values[0], values[1], values[2]]);
    }

    let output_filename = format!("output/{}.png", matches.value_of("INPUT").unwrap());
    let ref mut fout = File::create(output_filename).unwrap();
    image::ImageRgb8(image_buffer).save(fout, image::PNG).unwrap();
}
