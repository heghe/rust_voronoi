extern crate image;

use lib::{point_bounderies, ApplicationState, Point, Tile, SCALE_SIZE};
use std::collections::VecDeque;
use std::fs::File;

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

pub fn sequantial(state: &mut ApplicationState) {
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
