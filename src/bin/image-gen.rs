
struct Args {
    prefix: String,
    number: usize,
    destination: String,
    width_range: [usize; 2],
    height_range: [usize; 2],
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
            width_range: [32, 512],
            height_range: [32, 512],
        };

        if let Some(prefix) = matches.value_of("prefix") {
            args.prefix = String::from(prefix);
        }

        if let Some(number_option) = matches.value_of("number") {
            let result = number_option.parse::<usize>();
            match result {
                Result::Err(error) => {
                    return Result::Err(format!("{}", error));
                },
                Result::Ok(number) => {
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
                    match min_width_option.parse::<usize>() {
                        Result::Err(error) => {
                            return Result::Err(format!("{}", error));
                        }
                        Result::Ok(min_width) => {
                            args.width_range[0] = min_width;
                        }
                    }
                    match max_width_option.parse::<usize>() {
                        Result::Err(error) => {
                            return Result::Err(format!("{}", error));
                        }
                        Result::Ok(max_width) => {
                            args.width_range[1] = max_width;
                        }
                    }
                }
                _ => {
                    return Result::Err(String::from("Parse failed at width-range option"));
                }
            }
        }

        if let Some(mut height_range_option) = matches.values_of("height-range") {
            match (height_range_option.next(), height_range_option.next()) {
                (Some(min_height_option), Some(max_height_option)) => {
                    match min_height_option.parse::<usize>() {
                        Result::Err(error) => {
                            return Result::Err(format!("{}", error));
                        }
                        Result::Ok(min_height) => {
                            args.height_range[0] = min_height;
                        }
                    }
                    match max_height_option.parse::<usize>() {
                        Result::Err(error) => {
                            return Result::Err(format!("{}", error));
                        }
                        Result::Ok(max_height) => {
                            args.height_range[1] = max_height;
                        }
                    }
                }
                _ => {
                    return Result::Err(String::from("Parse failed at height-range option"));
                }
            }
        }

        return Result::Ok(args);
}

fn main() {
}
