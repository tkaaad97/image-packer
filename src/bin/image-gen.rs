use image::{ImageFormat, ImageBuffer, Rgba};
use rand::Rng;
use std::path::Path;

#[derive(Debug)]
struct Args {
    prefix: String,
    number: usize,
    destination: String,
    width_range: std::ops::Range<u32>,
    height_range: std::ops::Range<u32>,
}

fn parse_args() -> Result<Args, String> {
    let matches = clap::App::new("image-gen")
        .about("Generate random size and color image files")
        .arg(
            clap::Arg::new("prefix")
                .long("prefix")
                .short('p')
                .takes_value(true)
        )
        .arg(
            clap::Arg::new("number")
                .long("number")
                .short('n')
                .takes_value(true)
        )
        .arg(
            clap::Arg::new("destination")
                .long("destination")
                .short('d')
                .takes_value(true)
        )
        .arg(
            clap::Arg::new("width-range")
                .long("width-range")
                .short('w')
                .takes_value(true)
                .value_delimiter(',')
                .number_of_values(2)
        )
        .arg(
            clap::Arg::new("height-range")
                .long("height-range")
                .short('h')
                .takes_value(true)
                .value_delimiter(',')
                .number_of_values(2)
        )
        .get_matches();

        // default option values
        let mut args = Args {
            prefix: String::from("image"),
            number: 10,
            destination: String::from("."),
            width_range: 32..(512 + 1),
            height_range: 32..(512 + 1),
        };

        if let Some(prefix) = matches.value_of("prefix") {
            args.prefix = String::from(prefix);
        }

        if let Some(number_option) = matches.value_of("number") {
            let result = number_option.parse::<usize>();
            match result {
                Err(error) => {
                    return Err(format!("{}", error));
                },
                Ok(number) => {
                    args.number = number;
                }
            }
        }

        if let Some(destination) = matches.value_of("destination") {
            args.destination = String::from(destination);
        }

        if let Some(mut width_range_option) = matches.values_of("width-range") {
            match (width_range_option.next(), width_range_option.next()) {
                (Some(min_width_option), Some(max_width_option)) => {
                    match min_width_option.parse::<u32>() {
                        Err(error) => {
                            return Err(format!("{}", error));
                        }
                        Ok(min_width) => {
                            args.width_range.start = min_width;
                        }
                    }
                    match max_width_option.parse::<u32>() {
                        Err(error) => {
                            return Err(format!("{}", error));
                        }
                        Ok(max_width) => {
                            args.width_range.end = max_width + 1;
                        }
                    }
                    if args.width_range.start > args.width_range.end {
                        return Err(String::from("min width is larger than max width."));
                    }
                }
                _ => {
                    return Err(String::from("Parse failed at width-range option"));
                }
            }
        }

        if let Some(mut height_range_option) = matches.values_of("height-range") {
            match (height_range_option.next(), height_range_option.next()) {
                (Some(min_height_option), Some(max_height_option)) => {
                    match min_height_option.parse::<u32>() {
                        Err(error) => {
                            return Err(format!("{}", error));
                        }
                        Ok(min_height) => {
                            args.height_range.start = min_height;
                        }
                    }
                    match max_height_option.parse::<u32>() {
                        Err(error) => {
                            return Err(format!("{}", error));
                        }
                        Ok(max_height) => {
                            args.height_range.end = max_height + 1;
                        }
                    }
                    if args.height_range.start > args.height_range.end {
                        return Err(String::from("min height is larger than max height."));
                    }
                }
                _ => {
                    return Err(String::from("Parse failed at height-range option"));
                }
            }
        }

        return Ok(args);
}

fn image_gen(args: Args) {
    let mut rng = rand::thread_rng();
    let dir_path = Path::new(&args.destination);
    if !dir_path.is_dir() {
        std::fs::create_dir(dir_path).unwrap();
    }
    let mut buffer = Vec::<u8>::with_capacity(((args.width_range.end - 1) as usize) * ((args.height_range.end - 1) as usize) * 4);
    for _ in 0..buffer.capacity() {
        buffer.push(0);
    }

    for i in 1..(args.number + 1) {
        let path = dir_path.join(Path::new(&format!("{}{:03}", args.prefix, i))).with_extension("png");
        let w: u32 = rng.gen_range(args.width_range.clone());
        let h: u32 = rng.gen_range(args.height_range.clone());
        let r: u8 = rng.gen();
        let g: u8 = rng.gen();
        let b: u8 = rng.gen();
        let pixel_size = (w as usize) * (h as usize);
        buffer.resize(pixel_size * 4, 0);
        for p in 0..((w * h) as usize) {
            buffer[p * 4] = r;
            buffer[p * 4 + 1] = g;
            buffer[p * 4 + 2] = b;
            buffer[p * 4 + 3] = 255;
        }
        let image_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(w, h, buffer).unwrap();
        image_buffer.save_with_format(path, ImageFormat::Png).unwrap();
        buffer = image_buffer.into_vec();
    }
}

fn main() {
    let args = parse_args().unwrap();
    image_gen(args);
}
