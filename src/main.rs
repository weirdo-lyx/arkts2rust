use std::env;
use std::fs;
use std::process;

fn main() {
    let mut args = env::args().skip(1);

    let input_path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("Usage: arkts2rust <input> -o <output>");
            process::exit(2);
        }
    };

    let mut output_path: Option<String> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" => {
                output_path = args.next();
            }
            _ => {
                eprintln!("Unknown argument: {arg}");
                eprintln!("Usage: arkts2rust <input> -o <output>");
                process::exit(2);
            }
        }
    }

    let output_path = match output_path {
        Some(p) => p,
        None => {
            eprintln!("Missing -o <output>");
            eprintln!("Usage: arkts2rust <input> -o <output>");
            process::exit(2);
        }
    };

    let src = match fs::read_to_string(&input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read input file {input_path}: {e}");
            process::exit(2);
        }
    };

    match arkts2rust::compile(&src) {
        Ok(rust_code) => {
            if let Err(e) = fs::write(&output_path, rust_code) {
                eprintln!("Failed to write output file {output_path}: {e}");
                process::exit(2);
            }
        }
        Err(e) => {
            eprintln!("Compile failed: {e}");
            process::exit(1);
        }
    }
}
