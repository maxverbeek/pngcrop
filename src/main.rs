use docopt::Docopt;
use serde::Deserialize;

use image::DynamicImage;
use image::io::Reader as ImageReader;
use image::error::ImageResult;

use std::cmp::{min,max};

const USAGE: &str = "
Crop PNG files to their minimal bounding box

Usage:
    pngcrop <file>...
    pngcrop -h | --help

Options:
    -h --help  Show these options
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_file: Vec<String>,
}

fn main() {
    println!("Hello, world!");

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    println!("{:?}", args);

    for arg in &args.arg_file {
        match process_file(arg) {
            Ok(_) => {}
            Err(e) => { println!("{} does not exist or is not a valid PNG: {}", arg, e); }
        }
    }
}

#[derive(Debug)]
struct ContentBounds {
    minx: u32,
    maxx: u32,
    miny: u32,
    maxy: u32,
}

impl ContentBounds {
    fn width(&self) -> u32 {
        self.maxx - self.minx + 1
    }

    fn height(&self) -> u32 {
        self.maxy - self.miny + 1
    }
}

fn process_file(file: &String) -> ImageResult<()> {
    println!("{}", file);

    let img = ImageReader::open(file)?.decode()?;
    let has_alpha = img.color().has_alpha();
    
    if !has_alpha {
        return Ok(());
    }

    let canvas = img.into_rgba8();

    let bounds = find_boundaries(&canvas);

    let dest = "cropped_".to_owned() + file;

    DynamicImage::ImageRgba8(canvas).crop(bounds.minx, bounds.miny, bounds.width(), bounds.height()).save(dest)
}

fn find_boundaries(img: &image::RgbaImage) -> ContentBounds {
    let mut bounds = ContentBounds {
        minx: img.width(),
        maxx: 0,
        miny: img.height(),
        maxy: 0,
    };

    const ALPHA_IDX: usize = 3;

    for (x, y, pixel) in img.enumerate_pixels() {
        if pixel[ALPHA_IDX] != 0 {
            // get boundary coordinates for non-alpha pixels.
            // can be optimized with enumerate_row and smart skipping.
            bounds.minx = min(bounds.minx, x);
            bounds.maxx = max(bounds.maxx, x);
            bounds.miny = min(bounds.miny, y);
            bounds.maxy = max(bounds.maxy, y);
        }
    }

    bounds
}