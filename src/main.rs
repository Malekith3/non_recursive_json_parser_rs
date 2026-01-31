mod json_parsing_naive;

use std::fs;
use std::path::{Path, PathBuf};
use std::env;

enum ArgError {
    MissingPath,
    TooManyArgs,
}

use std::fmt;

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgError::MissingPath =>
                write!(f, "Missing path to JSON file"),
            ArgError::TooManyArgs =>
                write!(f, "Too many arguments"),
        }
    }
}

fn read_json_file(path_to_json_file: &Path) -> Result<String, std::io::Error> {
     fs::read_to_string(path_to_json_file)
}

fn parse_args( args: &[String]) -> Result<PathBuf, ArgError>{
    match args.len() {
        2 => Ok(PathBuf::from(args[1].clone())),
        1 => Err(ArgError::MissingPath),
        _ => Err(ArgError::TooManyArgs)
    }
}

fn print_usage(error: &ArgError) {
    println!("Error: {}", error);
    println!("Usage: json_parser <path_to_json_file>");
}

fn main() {
    println!("Welcome to JSON parser");
    println!("======================");
    let args: Vec<String> = env::args().collect();
    let path = match parse_args(&args) {
        Ok(path) => path,
        Err(err) => {
            print_usage(&err);
            return;
        }
    };

    let json_string = match read_json_file(&path) {
        Ok(json_string) => json_string,
        Err(err) => {
            println!("Error reading file: {} error: {}", path.display(), err);
            return;
        }
    };
    
    let result = json_parsing_naive::process_json_string(&*json_string);
    println!("{:?}", result.unwrap())

}
