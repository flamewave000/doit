#![allow(dead_code)]
use std::{
	env, fs, path::Path, process::{exit, Command, ExitCode}
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
	println!("    -c         Clean the current directory by removing the .doit directory.");
	println!();
	println!("  Dev Options:");
	println!("    --tokens   Print out the lexical tokens instead of fully compiling.");
	println!("    --nodes    Print out the parser node tree instead of fully compiling.");
	println!("    --source   Print the transpiled C++ code to stdout instead of fully compiling.");
	println!("    --keep     After compiling, do not delete the .doit/targets.cpp file.");
}

fn main() -> ExitCode {
	let mut args: Vec<String> = env::args().collect();
	let program_name = args.remove(0);
	let mut force_recompile = false;
	let mut print_tokens = false;
	let mut print_nodes = false;
	let mut print_source = false;
	let mut keep_source = false;
	let mut filename: String = String::new();
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
			"--version" => {
				println!("v{}", env!("CARGO_PKG_VERSION"));
				exit(0);
			},
			"-t" => filename = args.remove(0),
			"-c" => {
				if Path::new("./.doit").exists() {
					if let Err(err) = fs::remove_dir_all("./.doit") {
						log::error(&err.to_string());
						return ExitCode::from(1);
					}
				}
				log::info("Cleaned!");
				exit(0);
			},
			fail => {
				log::error(&format!("Unknown option: {}", fail));
				print_help(&program_name);
				return ExitCode::from(1);
			}
		}
	}

	if filename.is_empty() {
		filename = "./doit".to_string();
		if !Path::new(&filename).exists() && Path::new("./do.it").exists() {
			filename = "./do.it".to_string();
		}
	}

	match fs::canonicalize(Path::new(&filename)) {
		Ok(path) => {
			if let Some(path_str) = path.to_str() {
				filename = path_str.to_owned();
			} else {
				log::error(&format!("Could not calculate and absolute path for the provided doit file '{}'", filename));
				return ExitCode::from(1);
			}
		},
		Err(err) => {
			log::error(&format!("Could not calculate and absolute path for the provided doit file '{}'", filename));
			log::error(&err.to_string());
			return ExitCode::from(1);
		}
	}

	if !Path::new(&filename).exists() {
		log::error(&format!("Could not find '{}' file in current directory", filename));
		return ExitCode::from(1);
	}

	let directory = &format!("./.doit/{}", utils::hash::calculate_hash(&filename));
	log::debug(&format!("Output directory: {}", directory));

	if let Err(err) = compiler::build(
		directory,
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
		return ExitCode::from(1);
	}
	if print_tokens || print_nodes || print_source {
		return ExitCode::from(0);
	}

	let child = Command::new(directory.to_owned() + "/targets").args(&args).spawn();
	if let Err(err) = child {
		log::error(&err.to_string());
		ExitCode::from(0)
	} else {
		match child.unwrap().wait() {
			Ok(status) => {
				let code = status.code().unwrap_or(if status.success() { 0 } else { 1 });
				log::debug(&format!("Exit Code: {}", code));
				ExitCode::from(code as u8)
			},
			Err(err) => {
				log::error(&err.to_string());
				ExitCode::from(1)
			}
		}
	}
}
