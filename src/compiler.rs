use std::{
	fs::{self, File},
	hash::{DefaultHasher, Hash, Hasher},
	io::{Error, ErrorKind, Read, Write},
	path::Path,
	process::Command,
};

use crate::{
	generator::Generator,
	lexer::{
		token::{TokenType, Tokenizer},
		Lexer,
	},
	parser::Parser,
	utils::log,
};

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(PartialEq)]
pub enum CompileMode {
	NORMAL,
	PRINT_TOKENS,
	PRINT_NODES,
	PRINT_SOURCE,
}

pub fn build(directory: &String, filename: &str, keep: bool, force: bool, mode: CompileMode) -> Result<(), Error> {
	let mut source = String::new();
	{
		let mut file = File::open(filename)?;
		file.read_to_string(&mut source)?;
	}

	fs::create_dir_all(directory)?;

	let mut hasher = DefaultHasher::new();
	source.hash(&mut hasher);
	let new_hash = hasher.finish();
	if !force && Path::new(&(directory.to_owned() + "/hash")).exists() && Path::new(&(directory.to_owned() + "/targets")).exists() {
		let old_hash = fs::read_to_string(directory.to_owned() + "/hash")?;
		if old_hash == new_hash.to_string() {
			return Ok(());
		}
	}
	fs::write(directory.to_owned() + "/hash", new_hash.to_string())?;

	match &mode {
		CompileMode::PRINT_TOKENS => print_tokens(&mut Lexer::new(filename, &source)),
		CompileMode::PRINT_NODES => print_nodes(&mut Parser::new(&mut Lexer::new(filename, &source))),
		_ => {
			let mut lexer = Lexer::new(filename, &source);
			let mut parser = Parser::new(&mut lexer);
			let source = Generator::new(&mut parser).generate()?;
			if mode == CompileMode::PRINT_SOURCE {
				println!("{source}");
			} else {
				compile(directory, &source)?;
			}
		}
	}

	if !keep {
		let _ = fs::remove_file(directory.to_owned() + "/targets.cpp");
	}
	Ok(())
}

fn compile(directory: &String, source: &str) -> Result<(), Error> {
	let cpp = &(directory.to_owned() + "/targets.cpp");
	{
		let mut file = File::create(cpp)?;
		file.write_all(source.as_bytes())?;
		file.flush()?;
	}

	let mut output = Command::new("g++")
		.args(["--std=c++20", cpp, "-o", &(directory.to_owned() + "/targets")])
		.spawn()?;
	if !output.wait()?.success() {
		return Err(Error::new(ErrorKind::Other, "Failed to compile"));
	}
	Ok(())
}

fn print_tokens(lexer: &mut Lexer) {
	loop {
		let result = lexer.next_token();
		if let Ok(token) = result {
			log::info(&token.to_string());
			if token.ttype == TokenType::EOF {
				return;
			}
		} else {
			log::error(&result.err().unwrap().to_string());
			return;
		}
	}
}
fn print_nodes(parser: &mut Parser) {
	match parser.parse() {
		Ok(root) => root.to_string().trim().split('\n').for_each(log::info),
		Err(err) => log::error(&format!("Failed to parse: {}", err)),
	};
}
