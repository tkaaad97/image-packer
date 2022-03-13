use image::{GenericImage, ImageFormat, ImageBuffer, Rgba};
use image_packer::*;
use regex::Regex;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct Args {
    texture_size: [usize; 2],
    prefix: String,
    spacing: usize,
    enable_rotate: bool,
    input_filename_pattern: Option<String>,
    output_data_filename: String,
    input_path: String,
    output_path: String,
}

impl Args {
    fn parse() -> Result<Args> {
        let matches = clap::Command::new("image-packer")
            .about("pack input images into textures")
            .arg(
                clap::Arg::new("texture-size")
                    .long("texture-size")
                    .short('s')
                    .value_delimiter(',')
                    .number_of_values(2)
            )
            .arg(
                clap::Arg::new("texture-prefix")
                    .long("texture-prefix")
                    .takes_value(true)
            )
            .arg(
                clap::Arg::new("spacing")
                    .long("spacing")
                    .takes_value(true)
            )
            .arg(
                clap::Arg::new("enable-rotate")
                    .long("enable-rotate")
                    .takes_value(false)
            )
            .arg(
                clap::Arg::new("disable-rotate")
                    .long("disable-rotate")
                    .takes_value(false)
            )
            .arg(
                clap::Arg::new("input-filename-pattern")
                    .long("input-filename-pattern")
                    .short('p')
                    .takes_value(true)
            )
            .arg(
                clap::Arg::new("output-data-filename")
                    .long("output-data-filename")
                    .takes_value(true)
            )
            .arg(
                clap::Arg::new("input-path")
                    .takes_value(true)
                    .required(true)
            )
            .arg(
                clap::Arg::new("output-path")
                    .takes_value(true)
                    .required(true)
            )
            .get_matches();

        let texture_size: [usize; 2] = if let Some(mut option) = matches.values_of("texture-size") {
            let w = option.next().unwrap().parse::<usize>()?;
            let h = option.next().unwrap().parse::<usize>()?;
            if w > MAX_TEXTURE_SIZE || h > MAX_TEXTURE_SIZE {
                return Err(From::from(format!("texture size is too large. ({}, {})", w, h)));
            }
            [w, h]
        } else {
            [1024, 1024]
        };

        Ok(Args {
            texture_size,
            prefix: matches.value_of("texture-prefix").unwrap_or("texture").to_string(),
            spacing: matches.value_of("spacing").map_or(Ok(0), usize::from_str)?,
            enable_rotate: matches.is_present("enable-rotate") && !matches.is_present("disable-rotate"),
            input_filename_pattern: matches.value_of("input-filename-pattern").map(String::from),
            output_data_filename: matches.value_of("output-data-filename").unwrap_or("texture-information.json").to_string(),
            input_path: matches.value_of("input-path").unwrap().to_string(),
            output_path: matches.value_of("output-path").unwrap().to_string(),
        })
    }
}

fn str_to_error(e: &str) -> Box<dyn std::error::Error> {
    From::from(String::from(e))
}

fn main() -> Result<()> {
    let args = Args::parse()?;

    // find out input image paths
    let regex_option = args.input_filename_pattern.map_or(Ok(None),|a|Regex::new(&*a).map(Some))?;
    let mut input_paths = Vec::<PathBuf>::new();
    let dir = std::fs::read_dir(args.input_path)?;
    for entry in dir.into_iter() {
        let path = entry?.path();
        if !path.is_dir() {
            if let Some(ref regex) = regex_option {
                let filename = path.file_name()
                        .ok_or_else(||str_to_error("file_name empty"))?
                        .to_str()
                        .ok_or_else(||str_to_error("OsStr::to_str failed"))?;
                if regex.is_match(filename) {
                    input_paths.push(path);
                }
            } else {
                input_paths.push(path);
            }
        }
    }
    input_paths.sort();

    // load input images
    let mut images = Vec::<image::ImageBuffer<Rgba<u8>, _>>::new();
    let mut image_sizes = Vec::<[usize; 2]>::new();
    for path in input_paths.iter() {
        let image = image::open(path)?.to_rgba8();
        image_sizes.push([image.width() as usize, image.height() as usize]);
        images.push(image);
    }

    // packing
    let packer = Packer {
        texture_size: args.texture_size,
        spacing: args.spacing,
        enable_rotate: args.enable_rotate,
    };
    let packed_results = packer.pack(&image_sizes)?;

    // create output directory if it dose not exist
    let output_dir = std::path::Path::new(&args.output_path);
    if !output_dir.is_dir() {
        std::fs::create_dir(output_dir)?;
    }

    // output result textures and packed information json
    let mut output_data = OutputData {
        textures: Vec::<String>::with_capacity(packed_results.len()),
        image_layouts: Vec::<ImageLayoutInfo>::with_capacity(input_paths.len()),
    };
    for _ in 0..input_paths.len() {
        output_data.image_layouts.push(ImageLayoutInfo::empty());
    }
    let mut texture_buffer: Vec<u8> = vec![0; packer.texture_size[0] * packer.texture_size[1] * 4];
    for (texture_index, layouts) in packed_results.into_iter().enumerate() {
        let mut texture =  ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(packer.texture_size[0] as u32, packer.texture_size[1] as u32, texture_buffer)
                .ok_or(str_to_error("textrue initialize error"))?;

        for layout in layouts {
            texture.copy_from(&images[layout.index], layout.position[0] as u32, layout.position[1] as u32)?;
            let image_name = input_paths[layout.index]
                    .file_name()
                    .ok_or_else(||str_to_error("file_name empty"))?
                    .to_str()
                    .ok_or_else(||str_to_error("OsStr::to_str failed"))?;
            let image_layout = ImageLayoutInfo {
                name: String::from(image_name),
                texture: texture_index,
                position: layout.position,
                size: image_sizes[layout.index],
                rotated: layout.rotated,
            };
            output_data.image_layouts[layout.index] = image_layout;
        }

        let texture_name = format!("{}{:03}.png", args.prefix, texture_index);
        let texture_path = output_dir.join(Path::new(&texture_name));
        texture.save_with_format(texture_path, ImageFormat::Png)?;
        output_data.textures.push(texture_name);
        texture_buffer = texture.into_vec();
        texture_buffer.fill(0);
    }

    // output json
    output_data.image_layouts.sort_by(|a, b|a.name.cmp(&b.name));
    let output_data_path = output_dir.join(Path::new(&args.output_data_filename));
    serde_json::to_writer(File::create(output_data_path)?, &output_data)?;

    Ok(())
}
