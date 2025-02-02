use core::fmt;
use std::io::Error;

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
	SOF,     // Start of File
	EOF,     // \0
	EOL,     // \n
	SYMBOL,  // /+-*%^&@!.:;'`~
	EXIT,    // exit
	LIT_NUM, // 42
	LIT_STR, // "abc"
	COMMENT, // #
	SCRIPT,  // $
	TGT_BEG, // {
	TGT_END, // }
	TGT_SLE, // :
	NOMEN,   // abc_123
	ASSIGN,  // =
	HELP,    // @@@
	ARG_REQ, // req
	ARG_OPT, // opt
}

pub trait Tokenizer {
	fn peek_token(&mut self) -> Result<&Token, Error>;
	fn next_token(&mut self) -> Result<Token, Error>;
	fn get_filename(&self) -> &str;
	fn get_lineno(&self) -> usize;
	fn get_charno(&self) -> i32;
}

pub struct Token {
	pub ttype: TokenType,
	pub value: Option<String>,
}

impl Token {
	pub const fn sym(ttype: TokenType) -> Token {
		Token { ttype, value: None }
	}
	pub const fn val(ttype: TokenType, value: Option<String>) -> Token {
		Token { ttype, value }
	}
}

impl fmt::Display for Token {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self.value {
			Some(v) => write!(f, "{:?}({})", self.ttype, v),
			None => write!(f, "{:?}", self.ttype),
		}
	}
}
