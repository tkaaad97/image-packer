use image::{ImageFormat, ImageBuffer, Rgba};
use image_packer::*;
use regex::Regex;
use std::path::PathBuf;
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct Args {
    texture_size: [usize; 2],
    prefix: String,
    spacing: usize,
    input_filename_pattern: Option<String>,
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
                clap::Arg::new("input-filename-pattern")
                    .long("input-filename-pattern")
                    .short('p')
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
            input_filename_pattern: matches.value_of("input-filename-pattern").map(String::from),
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
    let mut input_paths = Vec::<PathBuf>::new();
    let dir = std::fs::read_dir(args.input_path)?;
    let regex_option = args.input_filename_pattern.map_or(Ok(None),|a|Regex::new(&*a).map(Some))?;
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
    Ok(())
}
