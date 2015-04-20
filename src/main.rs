extern crate getopts;
extern crate lodepng;
extern crate time;

use carving::Carver;
use getopts::Options;
use std::env;
use std::path::Path;
use std::convert::AsMut;
use std::process;

mod carving;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    fn print_usage(program: &str, opts: Options) {
        let brief = format!("Usage: {} INPUT-FILE [options]", program);
        print!("{}", opts.usage(&brief));
    }

    let mut opts = Options::new();
    opts.optopt("o", "output", "path to output the resulting image", "OUTPUT-FILE");
    opts.optopt("W", "width-reduction", "the number of pixels to reduce the width by", "WIDTH-COUNT");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("Invalid arguments: {}", f.to_string());
            print_usage(program, opts);
            process::exit(1);
        },
    };

    let input_img_path = if !matches.free.is_empty() {
        Path::new(&matches.free[0])
    } else {
        print_usage(&program, opts);
        process::exit(1);
    };
    let width_reduction: u32 = matches.opt_str("W").unwrap_or("1".to_owned())
        .parse().ok().expect("-W argument must be a number");

    let mut bitmap = match lodepng::decode24_file(input_img_path) {
        Ok(bitmap) => bitmap,
        Err(reason) => panic!("Could not load {}, because: {}", input_img_path.display(), reason),
    };

    println!("Decoded {} x {} image at {}", bitmap.width, bitmap.height,
        input_img_path.to_str().expect("path should be valid"));

    let mut carver = Carver::new(bitmap.buffer.len());
    println!("Reducing width of image by {} pixels... ", width_reduction);
    let start = time::precise_time_ns();
    for _ in 0..width_reduction {
        carver.calculate_energy(bitmap.width, bitmap.height,
            subset_by_width_and_height(bitmap.buffer.as_mut(), bitmap.width, bitmap.height));
        let seam = carver.find_seam(bitmap.width, bitmap.height);

        lazy_remove_indexes_of(subset_by_width_and_height(bitmap.buffer.as_mut(), bitmap.width, bitmap.height), &seam);
        bitmap.width = bitmap.width - 1;
    }
    let finish = time::precise_time_ns();
    println!("Finished in {} ms", (finish / 1000000) as i64 - (start / 1000000) as i64);

    match matches.opt_str("o") {
        Some(output_img_str) => {
            let output_img_path = &Path::new(&output_img_str);

            // image could be smaller now, so make sure we don't try to save more pixels than we have space for
            let portion_to_save = subset_by_width_and_height(bitmap.buffer.as_mut(), bitmap.width, bitmap.height);

            if let Err(e) = lodepng::encode24_file(output_img_path, portion_to_save, bitmap.width, bitmap.height) {
                panic!("Failed to save png to {} because: {}", output_img_str, e);
            }

            println!("Saved output image to {}", output_img_str);
        },
        None => println!("Not saving output image; specify -o flag if you want to save the result"),
    };
}

fn subset_by_width_and_height<A>(slice: &mut [A], width: usize, height: usize) -> &mut [A] {
    &mut slice[..(width * height)]
}

/// For each index `A` of `to_remove` into `slice`, set `slice[A] = slice[A + 1]`. The last `to_remove.len()` items in
/// `slice` will contain junk after this. Runs in linear time and requires `to_remove` to be sorted w.r.t. `slice`.
fn lazy_remove_indexes_of<A: Clone>(slice: &mut [A], to_remove: &Vec<usize>) {
    for (offset, start) in to_remove.iter().enumerate() {
        let finish = if offset < to_remove.len() - 1 {
            to_remove[offset + 1]
        } else {
            slice.len()
        };
        for idx in (start + 1)..finish {
            let src = idx;
            let dst = idx - offset - 1;
            // move element at src to dst; we don't care about what happens to the dst element, which we're deleting
            slice.swap(src, dst);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::lazy_remove_indexes_of;

    #[test]
    fn lazy_remove_indexes_of_works_correctly() {
        let mut vec = (0..11).collect::<Vec<u32>>();
        let to_remove = vec!(1, 3, 7);

        lazy_remove_indexes_of(&mut vec[..], &to_remove);
        let vec_len = vec.len();
        vec.truncate(vec_len - to_remove.len()); // only the first 7 elements of vec are valid now

        // 0 1 2 3 4 5 6 7 8 9 10 11 gets turned into
        // 0   2   4 5 6   8 9 10 11
        assert_eq!(vec, vec!(0, 2, 4, 5, 6, 8, 9, 10));
    }

    #[test]
    fn lazy_remove_indexes_of_works_correctly_on_edges() {
        let mut vec = (0..5).collect::<Vec<u32>>();
        let to_remove = vec!(0, 5);

        lazy_remove_indexes_of(&mut vec[..], &to_remove);
        let vec_len = vec.len();
        vec.truncate(vec_len - to_remove.len()); // only the first 3 elements of vec are valid now

        // 0 1 2 3 4 gets turned into
        //   1 2 3
        assert_eq!(vec, vec!(1, 2, 3));
    }
}
