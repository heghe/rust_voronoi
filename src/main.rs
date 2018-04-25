extern crate clap;
extern crate hsl;
extern crate image;
extern crate rand;

mod lib;

use self::rand::Rng;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::VecDeque;
use clap::{App, Arg};
use lib::{point_bounderies, ApplicationState, Point, Tile};
use hsl::HSL;

const SCALE_SIZE: u32 = 16;

fn make_image(
    space: &Vec<Vec<Tile>>,
    space_size: &Point,
    colors: &Vec<[u8; 3]>,
    filename: &String,
) {
    let mut image_buffer = image::ImageBuffer::new(
        SCALE_SIZE * space_size.x as u32,
        SCALE_SIZE * space_size.y as u32,
    );
    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
        let index = space
            .get((x / SCALE_SIZE) as usize)
            .unwrap()
            .get((y / SCALE_SIZE) as usize)
            .unwrap()
            .id;
        let values: &[u8; 3];
        if index == 0 {
            values = &[0 as u8, 0 as u8, 0 as u8];
        } else {
            values = colors.get(index - 1).unwrap();
        }
        *pixel = image::Rgb([values[0], values[1], values[2]]);
    }

    let ref mut fout = File::create(filename).unwrap();
    image::ImageRgb8(image_buffer)
        .save(fout, image::PNG)
        .unwrap();
}

pub fn generate_colors(n: usize) -> Vec<[u8; 3]> {
    let mut colors: Vec<[u8; 3]> = Vec::new();
    let mut rng = rand::thread_rng();
    let mut i: f64 = 0.0;
    let step: f64 = 360.0 / (n as f64);
    while i < 360.0 {
        let h: f64 = i;
        let s: f64 = 0.9 + rng.gen_range(0.0, 0.1);
        let l: f64 = 0.5 + rng.gen_range(0.0, 0.1);

        let hsl_color = HSL { h, s, l };
        let rgb_color = hsl_color.to_rgb();

        i += step;
        colors.push([rgb_color.0, rgb_color.1, rgb_color.2]);
    }
    colors
}

fn multithreading(state: &mut ApplicationState) {
    // TODO
}

fn sequantial(state: &mut ApplicationState) {
    let mut space: Vec<Vec<Tile>> = Vec::with_capacity(state.space_size.x);

    // creating application space dimension
    for i in 0..state.space_size.x {
        space.push(Vec::with_capacity(state.space_size.y));
        for j in 0..state.space_size.y {
            space.get_mut(i).unwrap().push(Tile::new(Point::new(i, j)));
        }
    }
    // put the seeds in the space
    // TODO make iterator for point struct to be able to use enumerate()
    let mut id = 1;
    for seed in &state.seeds {
        let mut tile = space.get_mut(seed.x).unwrap().get_mut(seed.y).unwrap();
        tile.id = id;
        tile.seed_position = *seed;
        id += 1;
    }

    let directions = vec![-1, 0, 1];
    // print initial set
    if state.debug_enabled {
        make_image(
            &space,
            &state.space_size,
            &state.colors,
            &format!("{}/0.png", &state.output_debug_directory),
        );
    }

    let mut queue: VecDeque<Point> = VecDeque::new();
    for seed in &state.seeds {
        queue.push_back(*seed);
    }

    let mut step = 1;
    let mut last_id = 1;

    while !queue.is_empty() {
        let position = queue.pop_front().unwrap();
        let current_tile = space
            .get(position.x)
            .unwrap()
            .get(position.y)
            .unwrap()
            .clone();

        if state.debug_enabled && last_id != current_tile.id {
            let output_debug_filename = format!("{}/{}.png", state.output_debug_directory, step);
            make_image(
                &space,
                &state.space_size,
                &state.colors,
                &output_debug_filename,
            );
            step += 1;
            last_id = current_tile.id;
        }
        for i in &directions {
            for j in &directions {
                match point_bounderies(&position, *i, *j, &state.space_size) {
                    // TODO move the below code in a separate function
                    Some((x, y)) => {
                        let mut next_tile = space.get_mut(x).unwrap().get_mut(y).unwrap();
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
    // print final image
    make_image(
        &space,
        &state.space_size,
        &state.colors,
        &state.output_filename,
    );

    // also write this to a file
    // IMAGE_ONLY flag
    if !state.image_only {
        for line in &space {
            for tile in line {
                print!("{} ", tile.id);
            }
            println!();
        }
    }
}

fn main() {
    let matches = App::new("Generating voronoi diagram")
        .version("1.0")
        .author("Heghedus Razvan <heghedus.razvan@gmail.com>")
        .args(&[
            Arg::with_name("INPUT")
                .help("Input set file")
                .long_help("Path to a valid input file as described in specification")
                .required(true)
                .index(1),
            Arg::with_name("MULTITHREADING")
                .short("m")
                .help("Enable multithreading. By default multithreading is disabled."),
            Arg::with_name("DEBUG_STEPS")
                .short("d")
                .help("Show debug images with intermediar steps"),
            Arg::with_name("IMAGE_ONLY")
                .short("i")
                .help("Don't create additionl output file with pixel id instead of color"),
        ])
        .get_matches();
    // get flags value
    let debug_enabled: bool = matches.is_present("DEBUG_STEPS");
    let multithreading_enable: bool = matches.is_present("MULTITHREADING");
    let image_only: bool = matches.is_present("IMAGE_ONLY");

    let filename = format!("data/{}", matches.value_of("INPUT").unwrap());
    println!("Using data file: {}", filename);
    // TODO more comments

    let mut file = BufReader::new(File::open(filename).unwrap());

    let mut line = String::new();
    // read X, Y
    file.read_line(&mut line).unwrap();
    let space_size = Point::from_string(&line);

    let mut seeds: Vec<Point> = Vec::new();

    // read n
    line.clear();
    file.read_line(&mut line).unwrap();
    let n: usize = line.trim().parse::<usize>().unwrap();

    for (_, line) in file.lines().enumerate() {
        let position = Point::from_string(&line.unwrap());
        seeds.push(position);
    }
    //generate n colors
    let colors = generate_colors(n);

    // directory
    let output_debug_directory = format!("output/debug_{}", matches.value_of("INPUT").unwrap());
    if debug_enabled {
        let _ = fs::create_dir_all(&output_debug_directory);
    }
    let output_filename = format!("output/{}.png", matches.value_of("INPUT").unwrap());

    let mut state: ApplicationState = ApplicationState {
        seeds,
        space_size,
        debug_enabled,
        output_debug_directory,
        colors,
        output_filename,
        image_only,
    };

    if !multithreading_enable {
        sequantial(&mut state);
    } else {
        multithreading(&mut state);
    }
}
