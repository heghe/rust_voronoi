extern crate clap;
extern crate hsl;
extern crate image;
extern crate rand;

mod lib;
mod multithread;
mod singlethread;

use self::rand::Rng;
use clap::{App, Arg};
use hsl::HSL;
use lib::{ApplicationState, Point};
use multithread::multithreading;
use singlethread::sequantial;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
        // delete previous debug images folder for the current set
        let _ = fs::remove_dir_all(&output_debug_directory);
        // create the new debug images folder
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
