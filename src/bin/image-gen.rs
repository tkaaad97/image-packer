use image::{ImageFormat, ImageBuffer, Rgba};
use rand::Rng;
use std::error;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
struct Args {
    prefix: String,
    number: usize,
    destination: String,
    width_range: [u32; 2],
    height_range: [u32; 2],
}

impl Args {
    fn parse() -> Result<Args> {
        let matches = clap::Command::new("image-gen")
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
                    .takes_value(true)
                    .value_delimiter(',')
                    .number_of_values(2)
            )
            .arg(
                clap::Arg::new("height-range")
                    .long("height-range")
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
            width_range: [32, 512],
            height_range: [32, 512],
        };

        if let Some(prefix) = matches.value_of("prefix") {
            args.prefix = String::from(prefix);
        }

        if let Some(number_option) = matches.value_of("number") {
            args.number = number_option.parse::<usize>()?;
        }

        if let Some(destination) = matches.value_of("destination") {
            args.destination = String::from(destination);
        }

        if let Some(mut width_range_option) = matches.values_of("width-range") {
            let min_width = width_range_option.next().unwrap().parse::<u32>()?;
            let max_width = width_range_option.next().unwrap().parse::<u32>()?;
            if min_width > max_width {
                return Err(From::from(format!("min width is larger than max width. {} > {}", min_width, max_width)));
            }
            args.width_range = [min_width, max_width];
        }

        if let Some(mut height_range_option) = matches.values_of("height-range") {
            let min_height = height_range_option.next().unwrap().parse::<u32>()?;
            let max_height = height_range_option.next().unwrap().parse::<u32>()?;
            if min_height > max_height {
                return Err(From::from(format!("min height is larger than max height. {} > {}", min_height, max_height)));
            }
        }

        return Ok(args);
    }
}

fn image_gen(args: Args) -> Result<()> {
    let mut rng = rand::thread_rng();
    let dir_path = Path::new(&args.destination);
    if !dir_path.is_dir() {
        std::fs::create_dir(dir_path)?;
    }
    let mut buffer = Vec::<u8>::with_capacity((args.width_range[1] as usize) * (args.height_range[1] as usize) * 4);
    for _ in 0..buffer.capacity() {
        buffer.push(0);
    }

    for i in 1..(args.number + 1) {
        let path = dir_path.join(Path::new(&format!("{}{:03}", args.prefix, i))).with_extension("png");
        let w: u32 = rng.gen_range(args.width_range[0]..(args.width_range[1] + 1));
        let h: u32 = rng.gen_range(args.height_range[0]..(args.height_range[1] + 1));
        let r: u8 = rng.gen();
        let g: u8 = rng.gen();
        let b: u8 = rng.gen();
        let pixel_size = (w as usize) * (h as usize);
        buffer.resize(pixel_size * 4, 0);
        for p in 0..pixel_size {
            buffer[p * 4] = r;
            buffer[p * 4 + 1] = g;
            buffer[p * 4 + 2] = b;
            buffer[p * 4 + 3] = 255;
        }
        let image_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(w, h, buffer)
            .ok_or::<Box<dyn error::Error>>(From::from("ImageBuffer::from_raw failed"))?;
        image_buffer.save_with_format(path, ImageFormat::Png)?;
        buffer = image_buffer.into_vec();
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse()?;
    image_gen(args)?;
    Ok(())
}
