#![allow(dead_code)]
use std::{
	env,
	path::Path,
	process::{exit, Command},
};

mod compiler;
mod generator;
mod lexer;
pub mod parser;
mod utils;

use compiler::CompileMode;

use crate::utils::log;

fn print_help(program_name: &str) {
	println!("Usage: {} [options] <target> [target_params]\n", program_name);
	println!("If the target is ommitted, command will print out a list of available targets.");
	println!();
	println!("  Options:");
	println!("    -f         Force recompile of do.it script.");
	println!("    -t <file>  Provide a file path to the do.it file if not in CWD.");
	println!();
	println!("  Dev Options:");
	println!("    --tokens   Print out the lexical tokens instead of fully compiling.");
	println!("    --nodes    Print out the parser node tree instead of fully compiling.");
	println!("    --source   Print the transpiled C++ code to stdout instead of fully compiling.");
	println!("    --keep     After compiling, do not delete the .doit/targets.cpp file.");
}

fn main() {
	let mut args: Vec<String> = env::args().collect();
	let program_name = args.remove(0);
	let mut force_recompile = false;
	let mut print_tokens = false;
	let mut print_nodes = false;
	let mut print_source = false;
	let mut keep_source = false;
	let mut filename: String = "./do.it".to_string();
	while !args.is_empty() && args[0].starts_with('-') {
		match args.remove(0).as_str() {
			"-f" => force_recompile = true,
			"--tokens" => print_tokens = true,
			"--nodes" => print_nodes = true,
			"--source" => print_source = true,
			"--keep" => keep_source = true,
			"--help" => {
				print_help(&program_name);
				exit(0);
			},
			"-t" => filename = args.remove(0),
			fail => {
				log::error(&format!("Unknown option: {}", fail));
				print_help(&program_name);
				exit(1);
			}
		}
	}

	if !Path::new(&filename).exists() {
		log::error(&format!("Could not find '{}' file in current directory", filename));
		exit(1);
	}
	if let Err(err) = compiler::build(
		&filename,
		keep_source,
		force_recompile,
		if print_tokens {
			CompileMode::PRINT_TOKENS
		} else if print_nodes {
			CompileMode::PRINT_NODES
		} else if print_source {
			CompileMode::PRINT_SOURCE
		} else {
			CompileMode::NORMAL
		},
	) {
		log::error(&err.to_string());
		exit(1);
	}
	if print_tokens || print_nodes || print_source { exit(0); }

	match Command::new(".doit/targets").args(&args).output() {
		Ok(output) => print!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)),
		Err(err) => log::error(&err.to_string()),
	}
}
