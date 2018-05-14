extern crate clap;
extern crate futures;
extern crate futures_cpupool;
extern crate hsl;
extern crate image;
extern crate rand;

mod lib;

use self::rand::Rng;
use clap::{App, Arg};
use futures::Future;
use futures_cpupool::CpuPool;
use hsl::HSL;
use lib::{point_bounderies, ApplicationState, Point, Tile};
use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex, MutexGuard};

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
        .write_to(fout, image::PNG)
        .unwrap();
}

fn make_image_mt(
    space: &Arc<Vec<Vec<Mutex<Tile>>>>,
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
            .lock()
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
        .write_to(fout, image::PNG)
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
    let mut space: Vec<Vec<Mutex<Tile>>> = Vec::with_capacity(state.space_size.x);

    for i in 0..state.space_size.x {
        space.push(Vec::with_capacity(state.space_size.y));
        for j in 0..state.space_size.y {
            space
                .get_mut(i)
                .unwrap()
                .push(Mutex::new(Tile::new(Point::new(i, j))));
        }
    }

    // vector of queues for each seed
    let mut queues: Vec<VecDeque<Point>> = Vec::with_capacity(state.seeds.len());

    let mut id = 1;
    for seed in &state.seeds {
        let mut seed_queue = VecDeque::new();
        let mut tile = space
            .get_mut(seed.x)
            .unwrap()
            .get_mut(seed.y)
            .unwrap()
            .lock()
            .unwrap();
        (*tile).id = id;
        (*tile).seed_position = *seed;
        id += 1;
        seed_queue.push_back(*seed);
        queues.push(seed_queue);
    }

    let mut steps: Vec<isize> = Vec::new();
    steps.push(1);
    let mut i: isize = state.space_size.x as isize / 2;
    while i > 0 {
        steps.push(i);
        i /= 2;
    }

    let pool: CpuPool = CpuPool::new_num_cpus();
    let space = Arc::new(space);

    let mut debug_step = 1;

    for step in steps {
        // if debug enable write debug image
        if state.debug_enabled {
            let output_debug_filename =
                format!("{}/{}.png", state.output_debug_directory, debug_step);
            make_image_mt(
                &space,
                &state.space_size,
                &state.colors,
                &output_debug_filename,
            );
            debug_step += 1;
        }
        //
        // spawn threads
        let mut _futures = Vec::new();

        for queue in &queues {
            let mut _queue = queue.clone();
            let _space = space.clone();
            let _step = step;
            let directions: Vec<isize> = vec![-1, 0, 1];
            let space_size = state.space_size.clone();
            _futures.push(pool.spawn_fn(move || {
                let mut next_queue: VecDeque<Point> = VecDeque::new();
                while !_queue.is_empty() {
                    let position = _queue.pop_front().unwrap();
                    let current_tile_mutex: MutexGuard<_> = _space
                        .get(position.x)
                        .unwrap()
                        .get(position.y)
                        .unwrap()
                        .lock()
                        .unwrap();
                    let current_tile: Tile = (*current_tile_mutex).clone();
                    drop(current_tile_mutex);
                    next_queue.push_back(current_tile.position);

                    for i in &directions {
                        for j in &directions {
                            match point_bounderies(
                                &position,
                                (*i) * _step,
                                (*j) * _step,
                                &space_size,
                            ) {
                                Some((x, y)) => {
                                    let mut next_tile: MutexGuard<
                                        _,
                                    > = _space.get(x).unwrap().get(y).unwrap().lock().unwrap();
                                    if current_tile.id != next_tile.id {
                                        if next_tile.id == 0 {
                                            (*next_tile).id = current_tile.id;
                                            (*next_tile).seed_position = current_tile.seed_position;
                                            next_queue.push_back(next_tile.position);
                                        } else {
                                            if !next_tile.closer_seed(&current_tile.seed_position) {
                                                (*next_tile).id = current_tile.id;
                                                (*next_tile).seed_position =
                                                    current_tile.seed_position;
                                                next_queue.push_back(next_tile.position);
                                            }
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                }
                let result: Result<VecDeque<Point>, ()> = Ok(next_queue);
                result
            }));
        }
        queues.clear();
        // wait for threads and update queues
        for _future in _futures {
            queues.push(_future.wait().unwrap().clone());
        }
    }
    make_image_mt(
        &space,
        &state.space_size,
        &state.colors,
        &state.output_filename,
    );
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
    let mut queue: VecDeque<Point> = VecDeque::new();
    // put the seeds in the space
    // TODO make iterator for point struct to be able to use enumerate()
    let mut id = 1;
    for seed in &state.seeds {
        let mut tile = space.get_mut(seed.x).unwrap().get_mut(seed.y).unwrap();
        tile.id = id;
        tile.seed_position = *seed;
        id += 1;
        queue.push_back(*seed);
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
