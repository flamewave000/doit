use std::io::{Error, ErrorKind};

use crate::lexer::token::{TokenType, Tokenizer};

use self::nodes::{Node, NodeType};

pub mod nodes;

pub struct Parser<'parser> {
	tokenizer: &'parser mut dyn Tokenizer,
}

impl Parser<'_> {
	pub fn new(tokenizer: &mut dyn Tokenizer) -> Parser<'_> {
		Parser { tokenizer }
	}
	fn generate_error(&self, error_kind: ErrorKind, message: &str) -> Error {
		Error::new(
			error_kind,
			format!(
				"{}:{}:{} > {message}",
				self.tokenizer.get_filename(),
				self.tokenizer.get_lineno(),
				self.tokenizer.get_charno()
			),
		)
	}
	fn handle_error(&self, read: Result<Node, Error>) -> Result<Node, Error> {
		match read {
			Ok(v) => Ok(v),
			Err(e) => Err(self.generate_error(ErrorKind::InvalidData, &e.to_string())),
		}
	}

	fn parse_expression(&mut self) -> Result<Vec<Node>, Error> {
		let mut nodes: Vec<Node> = vec![];
		while self.tokenizer.peek_token()?.ttype != TokenType::EOL
			&& self.tokenizer.peek_token()?.ttype != TokenType::EOF
		{
			nodes.push(Node::single(NodeType::SYMBOL, self.tokenizer.next_token()?));
		}
		Ok(nodes)
	}

	fn parse_nomenclature(&mut self, scope: &mut Node) -> Result<(), Error> {
		let name = self.tokenizer.next_token()?;
		loop {
			let next = self.tokenizer.next_token()?;
			match next.ttype {
				// Ignore EOL tokens that occur between the NOMEN and the next token
				TokenType::EOL => continue,
				TokenType::EOF => return Err(self.generate_error(ErrorKind::UnexpectedEof, "Unexpected end of file")),
				TokenType::ASSIGN => scope
					.children
					.push(Node::new(NodeType::ASSIGN, name, self.parse_expression()?)),
				TokenType::TGT_BEG => {
					let mut block = Node::single(NodeType::TARGET, name);
					self.parse_scope(&mut block)?;
					scope.children.push(block);
				}
				TokenType::TGT_SLE => {
					let script = Node::single(NodeType::SCR_SH, self.tokenizer.next_token()?);
					scope.children.push(Node::new(NodeType::TARGET, name, vec![script]));
				}
				token => {
					return Err(self.generate_error(
						ErrorKind::InvalidData,
						&format!(
							"Encountered unexpected Token: {token:?}({})",
							next.value.unwrap_or("".to_string())
						),
					))
				}
			}
			break;
		}
		Ok(())
	}

	fn parse_argdef(&mut self, scope: &mut Node, ntype: NodeType) -> Result<(), Error> {
		let mut node = Node::single(ntype, self.tokenizer.next_token()?);
		if self.tokenizer.peek_token()?.ttype == TokenType::HELP {
			node.help = Some(self.tokenizer.next_token()?);
		}
		scope.children.push(node);
		Ok(())
	}

	fn parse_scope(&mut self, scope: &mut Node) -> Result<(), Error> {
		loop {
			let next = self.tokenizer.peek_token()?;
			match next.ttype {
				// Ignore dangling EOL tokens (empty lines)
				TokenType::EOL => {
					self.tokenizer.next_token()?;
					continue;
				},
				TokenType::EOF | TokenType::TGT_END => {
					self.tokenizer.next_token()?;
					return Ok(());
				},
				TokenType::EXIT => scope.children.push(Node::new(
					NodeType::EXIT,
					self.tokenizer.next_token()?,
					self.parse_expression()?,
				)),
				TokenType::YIELD => scope.children.push(Node::single(
					NodeType::YIELD,
					self.tokenizer.next_token()?,
				)),
				TokenType::NOMEN => self.parse_nomenclature(scope)?,
				TokenType::SCR_SH => scope.children.push(Node::new(
					NodeType::SCR_SH,
					self.tokenizer.next_token()?,
					self.parse_expression()?,
				)),
				TokenType::SCR_PY => scope.children.push(Node::new(
					NodeType::SCR_PY,
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
				TokenType::ARG_REQ => self.parse_argdef(scope, NodeType::ARG_REQ)?,
				TokenType::ARG_OPT => self.parse_argdef(scope, NodeType::ARG_OPT)?,
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
		Ok(root_node)
	}
}

#[cfg(test)]
mod tests {
	use std::io::Error;

	use crate::{
		lexer::token::{Token, TokenType, Tokenizer},
		parser::nodes::NodeType,
	};

	use super::{nodes::Node, Parser};
	struct MockTockenizer {
		garbage: Vec<Token>,
		pub tokens: Vec<(TokenType, Option<String>)>,
		pub index: usize,
	}
	impl Tokenizer for MockTockenizer {
		fn peek_token(&mut self) -> Result<&crate::lexer::token::Token, std::io::Error> {
			self.garbage.push(Token::val(
				self.tokens[self.index].0.clone(),
				self.tokens[self.index].1.clone(),
			));
			Ok(self.garbage.last().unwrap())
		}
		fn next_token(&mut self) -> Result<crate::lexer::token::Token, std::io::Error> {
			let token = Token::val(self.tokens[self.index].0.clone(), self.tokens[self.index].1.clone());
			self.index += 1;
			Ok(token)
		}
		fn get_filename(&self) -> &str {
			"test-do.it"
		}
		fn get_lineno(&self) -> usize {
			0
		}
		fn get_charno(&self) -> i32 {
			0
		}
	}

	fn check_node(node: Option<&Node>, expected_ntype: NodeType, expected_value: &str) {
		assert_eq!(node.unwrap().ntype, expected_ntype);
		assert_eq!(node.unwrap().value.value.as_ref().unwrap(), expected_value);
	}
	fn check_help(node: Option<&Node>, expected_help: &str) {
		assert_eq!(node.unwrap().help.as_ref().unwrap().ttype, TokenType::HELP);
		assert_eq!(
			node.unwrap().help.as_ref().unwrap().value.as_ref().unwrap(),
			expected_help
		);
	}
	fn some(string: &str) -> Option<String> {
		Some(string.to_string())
	}

	#[test]
	fn test_parser() -> Result<(), Error> {
		let mut mock_tockenizer = MockTockenizer {
			garbage: vec![],
			index: 0,
			tokens: vec![
				(TokenType::SOF, None),
				(TokenType::HELP, some("help1")),
				(TokenType::EOL, None),
				(TokenType::COMMENT, some("comment1")),
				(TokenType::EOL, None),
				(TokenType::NOMEN, some("test1")),
				(TokenType::TGT_BEG, None),
				(TokenType::EOL, None),
				(TokenType::HELP, some("\thelp2\t")),
				(TokenType::EOL, None),
				(TokenType::ARG_REQ, some("-a")),
				(TokenType::EOL, None),
				(TokenType::ARG_REQ, some("-b")),
				(TokenType::HELP, some("help3")),
				(TokenType::EOL, None),
				(TokenType::ARG_OPT, some("-c")),
				(TokenType::EOL, None),
				(TokenType::ARG_OPT, some("-d")),
				(TokenType::HELP, some("help4")),
				(TokenType::EOL, None),
				(TokenType::SCR_SH, some("script1")),
				(TokenType::EOL, None),
				(TokenType::COMMENT, some("comment2")),
				(TokenType::EOL, None),
				(TokenType::SCR_PY, some("python1")),
				(TokenType::EOL, None),
				(TokenType::YIELD, None),
				(TokenType::EOL, None),
				(TokenType::TGT_END, None),
				(TokenType::EOL, None),
				(TokenType::NOMEN, some("test2")),
				(TokenType::TGT_SLE, None),
				(TokenType::SCR_SH, some("script2")),
				(TokenType::EOL, None),
				(TokenType::EOF, None),
			],
		};
		let mut parser = Parser::new(&mut mock_tockenizer);
		let root = parser.parse()?;
		assert_eq!(root.ntype, NodeType::ROOT);
		check_help(Some(&root), "help1");
		check_node(root.children.first(), NodeType::COMMENT, "comment1");
		check_node(root.children.get(1), NodeType::TARGET, "test1");
		check_help(root.children.get(1), "\thelp2\t");

		check_node(root.children.get(1).unwrap().children.first(), NodeType::ARG_REQ, "-a");
		check_node(root.children.get(1).unwrap().children.get(1), NodeType::ARG_REQ, "-b");
		check_help(root.children.get(1).unwrap().children.get(1), "help3");
		check_node(root.children.get(1).unwrap().children.get(2), NodeType::ARG_OPT, "-c");
		check_node(root.children.get(1).unwrap().children.get(3), NodeType::ARG_OPT, "-d");
		check_help(root.children.get(1).unwrap().children.get(3), "help4");

		check_node(
			root.children.get(1).unwrap().children.get(4),
			NodeType::SCR_SH,
			"script1",
		);
		check_node(
			root.children.get(1).unwrap().children.get(5),
			NodeType::COMMENT,
			"comment2",
		);
		check_node(
			root.children.get(1).unwrap().children.get(6),
			NodeType::SCR_PY,
			"python1",
		);
		check_node(root.children.get(2), NodeType::TARGET, "test2");
		assert!(root.children.get(2).unwrap().help.is_none());
		check_node(
			root.children.get(2).unwrap().children.first(),
			NodeType::SCR_SH,
			"script2",
		);
		Ok(())
	}
}
