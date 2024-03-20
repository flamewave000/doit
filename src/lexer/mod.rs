// #![allow(dead_code)]

use std::{
	cmp::max,
	io::{Error, ErrorKind},
};

mod validation;
use validation::{is_nomenclature, is_number};
pub mod token;
use token::{Token, TokenType, Tokenizer};

use crate::lexer::lexers::read_comment;

use self::lexers::{read_nomenclature, read_number, read_script, read_string, scan_whitespace, Consumer};
mod lexers;

pub struct Lexer<'lexer> {
	filename: &'lexer str,
	source: Vec<char>,
	index: usize,
	row: usize,
	col: i32,
	peeked_token: Option<Token>,
	first: bool,
}

impl Consumer for Lexer<'_> {
	fn consume(&mut self) -> Result<char, String> {
		if self.index >= self.source.len() {
			return Err("End of file".to_string());
		}
		let value = *self.source.get(self.index).unwrap();
		self.index += 1;
		self.col += 1;
		if value == '\r' {
			return self.consume();
		}
		if value == '\n' {
			self.col = 0;
			self.row += 1;
		}
		Ok(value)
	}
	fn consume_and_ignore(&mut self) -> Result<(), String> {
		let _ = self.consume()?;
		Ok(())
	}
	fn peek(&self) -> Option<char> {
		self.source.get(self.index).copied()
	}
	fn look_ahead(&self, ahead: usize) -> Option<char> {
		if ahead == 0 {
			panic!("ahead parameter must be greater than zero");
		}
		self.source.get(self.index + (ahead - 1)).copied()
	}
}

impl Lexer<'_> {
	pub fn new<'new>(filename: &'new str, source: &'new str) -> Lexer<'new> {
		Lexer {
			filename,
			index: 0,
			source: source.chars().collect(),
			row: 1,
			col: 0,
			peeked_token: None,
			first: true,
		}
	}

	fn consume_and_ignore(&mut self) -> Result<(), Error> {
		match self.consume() {
			Ok(_) => Ok(()),
			Err(err) => Err(Error::new(ErrorKind::Other, err)),
		}
	}

	fn generate_error<T>(&self, error_kind: ErrorKind, message: &str) -> Result<T, Error> {
		Err(Error::new(
			error_kind,
			format!("{}:{}:{} > {message}", self.filename, self.row, self.col),
		))
	}

	fn peek_next(&self) -> Option<char> {
		if self.index >= self.source.len() {
			return None;
		}
		Some(self.source[self.index + 1])
	}

	fn put_back(&mut self) {
		self.index = max(0, self.index - 1);
		if self.source.get(self.index) == Some(&'\n') {
			self.row -= 1;
		}
	}

	fn handle_error(&self, read: Result<Vec<char>, String>) -> Result<String, Error> {
		match read {
			Ok(v) => Ok(v.iter().collect()),
			Err(e) => self.generate_error(ErrorKind::InvalidData, &e),
		}
	}

	fn consume_token(&mut self) -> Result<Token, Error> {
		if self.first {
			self.first = false;
			return Ok(Token::sym(TokenType::SOF));
		}
		if let Err(err) = scan_whitespace(self) {
			return self.generate_error(ErrorKind::InvalidData, &err);
		}
		let next = match self.peek() {
			Some(v) => v,
			None => return Ok(Token::sym(TokenType::EOF)),
		};
		return if is_number(&next) {
			match read_number(self) {
				Ok(v) => Ok(Token::val(TokenType::LIT_NUM, Some(v.iter().collect()))),
				Err(e) => self.generate_error(ErrorKind::InvalidData, &e),
			}
		} else if is_nomenclature(&next, true) {
			let symbol: String = match read_nomenclature(self) {
				Ok(v) => v.iter().collect(),
				Err(e) => return self.generate_error(ErrorKind::InvalidData, &e),
			};
			match symbol.as_str() {
				"exit" => Ok(Token::sym(TokenType::EXIT)),
				_ => Ok(Token::val(TokenType::NOMEN, Some(symbol))),
			}
		} else if next == '"' {
			let result = read_string(self);
			let value = self.handle_error(result)?;
			Ok(Token::val(TokenType::LIT_STR, Some(value)))
		} else if next == '#' {
			let result = read_comment(self);
			let value = self.handle_error(result)?;
			Ok(Token::val(TokenType::COMMENT, Some(value)))
		} else if next == '$' {
			let result = read_script(self);
			let value = self.handle_error(result)?.trim().to_string();
			Ok(Token::val(TokenType::SCRIPT, Some(value)))
		} else if next == '{' {
			self.consume_and_ignore()?;
			Ok(Token::sym(TokenType::TGT_BEG))
		} else if next == '}' {
			self.consume_and_ignore()?;
			Ok(Token::sym(TokenType::TGT_END))
		} else if next == ':' {
			// We are swapping the ':' for a '$' to force a script token on the next iteration.
			// This essentially creates an implicit script tag for SLE targets.
			self.source[self.index] = '$';
			Ok(Token::sym(TokenType::TGT_SLE))
		} else if next == '=' {
			self.consume_and_ignore()?;
			Ok(Token::sym(TokenType::ASSIGN))
		} else if next == '\n' {
			self.consume_and_ignore()?;
			Ok(Token::sym(TokenType::EOL))
		} else if next == '@' {
			let help_block = lexers::read_help_block(self);
			Ok(Token::val(TokenType::HELP, Some(self.handle_error(help_block)?)))
		} else {
			Ok(Token::val(TokenType::SYMBOL, Some(self.consume().unwrap().to_string())))
		};
	}
}

impl Tokenizer for Lexer<'_> {
	fn get_filename(&self) -> &str {
		self.filename
	}
	fn get_lineno(&self) -> usize {
		self.row
	}
	fn get_charno(&self) -> i32 {
		self.col
	}
	fn peek_token(&mut self) -> Result<&Token, Error> {
		if self.peeked_token.is_none() {
			self.peeked_token = Some(self.consume_token()?);
		}
		Ok(self.peeked_token.as_ref().unwrap())
	}

	fn next_token(&mut self) -> Result<Token, Error> {
		let token = self.peeked_token.take();
		match token {
			Some(t) => Ok(t),
			None => self.consume_token(),
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::lexer::token::TokenType;

use super::{token::{Token, Tokenizer}, Lexer};
	use std::io::Error;

	fn check(token: Token, expected_type: TokenType, expected_value: &str) {
		assert_eq!(token.ttype, expected_type);
		assert_eq!(token.value.unwrap_or("".to_string()).as_str(), expected_value);
	}

	#[test]
	fn test_lexer() -> Result<(), Error> {
		let mut lexer = Lexer::new("test-source.it", r"@@@
help1
@@@
# comment1
test1 {
	@@@
	help2
	@@@
	$ script1
	# comment2
}
test2: script2
");
		check(lexer.next_token()?, TokenType::SOF, "");
		check(lexer.next_token()?, TokenType::HELP, "help1");
		check(lexer.next_token()?, TokenType::EOL, "");
		check(lexer.next_token()?, TokenType::COMMENT, " comment1");
		check(lexer.next_token()?, TokenType::EOL, "");
		check(lexer.next_token()?, TokenType::NOMEN, "test1");
		check(lexer.next_token()?, TokenType::TGT_BEG, "");
		check(lexer.next_token()?, TokenType::EOL, "");
		check(lexer.next_token()?, TokenType::HELP, "\thelp2\t");
		check(lexer.next_token()?, TokenType::EOL, "");
		check(lexer.next_token()?, TokenType::SCRIPT, "script1");
		check(lexer.next_token()?, TokenType::EOL, "");
		check(lexer.next_token()?, TokenType::COMMENT, " comment2");
		check(lexer.next_token()?, TokenType::EOL, "");
		check(lexer.next_token()?, TokenType::TGT_END, "");
		check(lexer.next_token()?, TokenType::EOL, "");
		check(lexer.next_token()?, TokenType::NOMEN, "test2");
		check(lexer.next_token()?, TokenType::TGT_SLE, "");
		check(lexer.next_token()?, TokenType::SCRIPT, "script2");
		check(lexer.next_token()?, TokenType::EOL, "");
		check(lexer.next_token()?, TokenType::EOF, "");
		Ok(())
	}
}
