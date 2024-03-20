use std::{
	fs::{self, File}, hash::{DefaultHasher, Hash, Hasher}, io::{Error, ErrorKind, Read, Write}, path::Path, process::Command
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

pub fn build(filename: &str, keep: bool, force: bool, mode: CompileMode) -> Result<(), Error> {
	let mut source = String::new();
	{
		let mut file = File::open(filename)?;
		file.read_to_string(&mut source)?;
	}

	fs::create_dir_all("./.doit/")?;

	let mut hasher = DefaultHasher::new();
	source.hash(&mut hasher);
	let new_hash = hasher.finish();
	if !force && Path::new(".doit/hash").exists() && Path::new(".doit/targets").exists() {
		let old_hash = fs::read_to_string(".doit/hash")?;
		if old_hash == new_hash.to_string() {
			return Ok(());
		}
	}
	fs::write(".doit/hash", new_hash.to_string())?;

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
				compile(&source)?;
			}
		}
	}

	if !keep {
		let _ = fs::remove_file(".doit/targets.cpp");
	}
	Ok(())
}

fn compile(source: &str) -> Result<(), Error> {
	let cpp: &str = "./.doit/targets.cpp";
	{
		let mut file = File::create(cpp)?;
		file.write_all(source.as_bytes())?;
		file.flush()?;
	}

	let output = Command::new("g++").args(["--std=c++20", cpp, "-o", "./.doit/targets"]).output()?;
	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr);
		stderr.split('\n').for_each(log::error);
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
