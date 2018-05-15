extern crate futures;
extern crate futures_cpupool;
extern crate image;

use self::futures::Future;
use self::futures_cpupool::CpuPool;
use lib::{point_bounderies, ApplicationState, Point, Tile, SCALE_SIZE};
use std::collections::VecDeque;
use std::fs::File;
use std::sync::{Arc, Mutex, MutexGuard};

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

pub fn multithreading(state: &mut ApplicationState) {
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
