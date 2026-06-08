use std::env;
use std::fs;
use std::process;

use compiler::compile;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!(
            "It should be like that: {} <input.rs> <output.bin> [--ast <ast.txt>] [--asm <asm.txt>]",
            args[0]
        );
        process::exit(1);
    }

    let input_path = &args[1];
    let output_bin_path = &args[2];

    let source = fs::read_to_string(input_path).unwrap_or_else(|e| {
        println!("Err reading file {} : {}", input_path, e);
        process::exit(1);
    });

    let result = compile(&source).unwrap_or_else(|e| {
        println!("Compilation error: {}", e);
        process::exit(1);
    });

    if let Err(e) = fs::write(output_bin_path, &result.binary) {
        println!("Err writing bin to {} : {}", output_bin_path, e);
        process::exit(1);
    }

    let mut ast_path = None;
    let mut asm_path = None;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--ast" => {
                if i + 1 < args.len() {
                    ast_path = Some(&args[i + 1]);
                    i += 2;
                } else {
                    println!("Err no file for ast");
                    process::exit(1);
                }
            }
            "--asm" => {
                if i + 1 < args.len() {
                    asm_path = Some(&args[i + 1]);
                    i += 2;
                } else {
                    println!("Err no file for asm");
                    process::exit(1);
                }
            }
            _ => {
                process::exit(1);
            }
        }
    }

    if let Some(path) = ast_path {
        fs::write(path, &result.ast_debug).unwrap();
    }

    if let Some(path) = asm_path {
        fs::write(path, &result.assembly).unwrap();
    }

    println!("Done!");
}
