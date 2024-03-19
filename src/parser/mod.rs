use std::io::{Error, ErrorKind};

use crate::lexer::token::{TokenType, Tokenizer};

use self::nodes::{Node, NodeType};

pub mod nodes;

pub struct Parser<'parser> {
	tokenizer: &'parser mut dyn Tokenizer,
}

impl Parser<'_> {
	pub fn new<'new>(tokenizer: &'new mut dyn Tokenizer) -> Parser<'new> {
		return Parser { tokenizer };
	}
	fn generate_error(&self, error_kind: ErrorKind, message: &str) -> Error {
		return Error::new(
			error_kind,
			format!(
				"{}:{}:{} > {message}",
				self.tokenizer.get_filename(),
				self.tokenizer.get_lineno(),
				self.tokenizer.get_charno()
			),
		);
	}
	fn handle_error(&self, read: Result<Node, Error>) -> Result<Node, Error> {
		return match read {
			Ok(v) => Ok(v),
			Err(e) => Err(self.generate_error(ErrorKind::InvalidData, &e.to_string())),
		};
	}

	fn parse_expression(&mut self) -> Result<Vec<Node>, Error> {
		let mut nodes: Vec<Node> = vec![];
		while self.tokenizer.peek_token()?.ttype != TokenType::EOL
			&& self.tokenizer.peek_token()?.ttype != TokenType::EOF
		{
			nodes.push(Node::single(NodeType::SYMBOL, self.tokenizer.next_token()?));
		}
		return Ok(nodes);
	}

	fn parse_nomenclature(&mut self, scope: &mut Node) -> Result<(), Error> {
		let name = self.tokenizer.next_token()?;
		loop {
			let next = self.tokenizer.next_token()?;
			match next.ttype {
				// Ignore EOL tokens that occur between the NOMEN and the next token
				TokenType::EOL => continue,
				TokenType::EOF => return Err(self.generate_error(ErrorKind::UnexpectedEof, "Unexpected end of file")),
				TokenType::ASSIGN => {
					scope
						.children
						.push(Node::new(NodeType::ASSIGN, name, self.parse_expression()?))
				}
				TokenType::TGT_BEG => {
					let mut block = Node::single(NodeType::TARGET, name);
					self.parse_scope(&mut block)?;
					scope.children.push(block);
				}
				TokenType::TGT_SLE => {
					let script = Node::single(NodeType::SCRIPT, self.tokenizer.next_token()?);
					scope.children.push(Node::new(NodeType::TARGET, name, vec![script]));
				}
				token => panic!("Encountered unexpected Token: {:?}", token),
			}
			break;
		}
		return Ok(());
	}

	fn parse_scope(&mut self, scope: &mut Node) -> Result<(), Error> {
		loop {
			let next = self.tokenizer.peek_token()?;
			match next.ttype {
				// Ignore dangling EOL tokens (empty lines)
				TokenType::EOL => {
					let _ = self.tokenizer.next_token();
					continue;
				}
				TokenType::EOF | TokenType::TGT_END => {
					let _ = self.tokenizer.next_token();
					return Ok(());
				}
				TokenType::EXIT => scope.children.push(Node::new(
					NodeType::EXIT,
					self.tokenizer.next_token()?,
					self.parse_expression()?,
				)),
				TokenType::NOMEN => self.parse_nomenclature(scope)?,
				TokenType::SCRIPT => scope.children.push(Node::new(
					NodeType::SCRIPT,
					self.tokenizer.next_token()?,
					self.parse_expression()?,
				)),
				TokenType::COMMENT => scope.children.push(Node::new(
					NodeType::COMMENT,
					self.tokenizer.next_token()?,
					self.parse_expression()?,
				)),
				TokenType::HELP => {
					if scope.help.is_some() {
						return Err(Error::new(
							ErrorKind::Other,
							"Help block has already been defined for the current scope.",
						));
					}
					scope.help = Some(self.tokenizer.next_token()?);
				}
				_ => panic!("Encountered unexpected Token: {}", next),
			}
		}
	}

	pub fn parse(&mut self) -> Result<Node, Error> {
		let root_token = self.tokenizer.next_token()?;
		if root_token.ttype != TokenType::SOF {
			return Err(self.generate_error(
				ErrorKind::NotFound,
				"Missing Start of File token. Has the tokenizer already been iterated on?",
			));
		}
		let mut root_node = Node::single(NodeType::ROOT, root_token);
		self.parse_scope(&mut root_node)?;
		return Ok(root_node);
	}
}
