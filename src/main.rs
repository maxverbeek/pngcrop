use docopt::Docopt;
use serde::Deserialize;

use image::DynamicImage;
use image::io::Reader as ImageReader;
use image::error::ImageResult;

const USAGE: &str = "
Crop PNG files to their minimal bounding box

Usage:
    pngcrop <file>...
    pngcrop -o <output> <source>
    pngcrop -h | --help

Options:
    -h --help                   Show these options
    -o <file>, --output <file>  Specify the output path
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_file: Vec<String>,
    flag_output: Option<String>,
    arg_source: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    println!("{:?}", args);

    if let (Some(src), Some(dest)) = (args.arg_source, args.flag_output) {
        try_conversion(&src, &dest);
        return;
    }

    for path in &args.arg_file {
        let dest = find_destination(path);
        try_conversion(&path, &dest);
    }
}

fn find_destination(source: &String) -> &String {
    source
}

fn try_conversion(path: &String, dest: &String) {
    match process_file(path, dest) {
        Ok(_) => {}
        Err(e) => { println!("{} does not exist or is not a valid PNG: {}", path, e); }
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

fn process_file(path: &String, dest: &String) -> ImageResult<()> {
    println!("{}", path);

    let img = ImageReader::open(path)?.decode()?;
    
    let canvas = img.into_rgba8();

    let bounds = find_boundaries(&canvas);

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
    
    let topleft = img.get_pixel(0, 0);
    let botrght = img.get_pixel(img.width() - 1, img.height() - 1);

    // select a pixel that represents the background of the image.
    // this must be the topleft or bottomright pixel, since if it is
    // not one of those pixels, the picture cannot be cropped further.
    // if the pixels are not the same, it is ambiguous what the background
    // of the image is, unless one of those pixels is transparent.
    let sample = {
        if topleft == botrght {
            topleft
        } else if topleft[ALPHA_IDX] == 0 {
            topleft
        } else if botrght[ALPHA_IDX] == 0 {
            botrght
        } else {
            // nothing to be done, ambiguous what the background is
            bounds.minx = 0;
            bounds.maxx = img.width() - 1;
            bounds.miny = 0;
            bounds.maxy = img.height() - 1;
            return bounds;
        }
    };

    for (x, y, pixel) in img.enumerate_pixels() {
        if pixel != sample {
            // get boundary coordinates for non-background pixels.
            // can be optimized with enumerate_row and smart skipping.
            bounds.minx = min(bounds.minx, x);
            bounds.maxx = max(bounds.maxx, x);
            bounds.miny = min(bounds.miny, y);
            bounds.maxy = max(bounds.maxy, y);
        }
    }

    bounds
}

fn min<T: Ord>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

fn max<T: Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}