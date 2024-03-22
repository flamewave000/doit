use std::io::{Error, ErrorKind};

use crate::{
	lexer::token::TokenType,
	parser::nodes::{Node, NodeType},
};

pub fn node_value(node: &Node) -> &str {
	let value = node.value.value.as_ref();
	return match value {
		Some(res) => AsRef::as_ref(res),
		None => "",
	};
}

pub fn generate_variable(node: &Node, exists: bool) -> Result<String, Error> {
	if node.value.value.is_none() {
		return Err(Error::new(ErrorKind::InvalidData, ""));
	}
	let mut expression = String::new();
	for child in &node.children {
		if child.ntype != NodeType::SYMBOL {
			return Err(Error::new(ErrorKind::InvalidData, "Unexpected node type"));
		}
		if let Some(value) = &child.value.value {
			expression.push(' ');
			if child.value.ttype == TokenType::LIT_STR {
				expression.push('"');
				expression.push_str(value);
				expression.push('"');
			} else {
				expression.push_str(value);
			}
		}
	}
	let var_type = if exists {
		""
	} else if node.children[0].value.ttype == TokenType::LIT_NUM {
		"double "
	} else {
		"::std::string "
	};
	Ok(format!(
		"{}{} ={};\n",
		var_type,
		node.value.value.as_ref().unwrap(),
		&expression
	))
}
pub fn generate_script(node: &Node, vars: &[&str]) -> Result<String, Error> {
	let vars: Vec<String> = vars.iter().map(|var| format!("__VAR({})", *var)).collect();
	Ok(format!(
		r#"__SYSTEM(R"__DOIT__({})__DOIT__", ::doit::args_map({{{}}}));{}"#,
		node_value(node),
		vars.join(","),
		'\n'
	))
}
pub fn generate_comment(node: &Node) -> Result<String, Error> {
	Ok(format!("//{}\n", node.value.value.as_ref().unwrap_or(&String::new())))
}
pub fn generate_exit(node: &Node) -> Result<String, Error> {
	let expression: Vec<&str> = node.children.iter().map(node_value).collect();
	Ok(format!("exit({});\n", expression.join(" ")))
}

#[cfg(test)]
mod tests {
	use std::io::Error;

	use crate::{
		generator::generators::{generate_comment, generate_script, generate_variable},
		lexer::token::{Token, TokenType},
		parser::nodes::{Node, NodeType},
	};

	use super::generate_exit;
	fn some(string: &str) -> Option<String> {
		Some(string.to_string())
	}

	#[test]
	fn test_generate_variable() -> Result<(), Error> {
		let mut node = Node::single(NodeType::ASSIGN, Token::val(TokenType::ASSIGN, some("my_var")));
		node.children = vec![Node::single(
			NodeType::SYMBOL,
			Token::val(TokenType::LIT_NUM, some("42")),
		)];
		let result = generate_variable(&node, false)?;
		assert_eq!(result, "double my_var = 42;\n");
		let mut result = generate_variable(&node, true)?;
		assert_eq!(result, "my_var = 42;\n");
		node.children = vec![Node::single(
			NodeType::SYMBOL,
			Token::val(TokenType::LIT_STR, some("Hello, world!")),
		)];
		result = generate_variable(&node, false)?;
		assert_eq!(result, "::std::string my_var = \"Hello, world!\";\n");
		result = generate_variable(&node, true)?;
		assert_eq!(result, "my_var = \"Hello, world!\";\n");
		node.children = vec![
			Node::single(NodeType::SYMBOL, Token::val(TokenType::LIT_NUM, some("42"))),
			Node::single(NodeType::SYMBOL, Token::val(TokenType::SYMBOL, some("+"))),
			Node::single(NodeType::SYMBOL, Token::val(TokenType::SYMBOL, some("other_var"))),
		];
		result = generate_variable(&node, false)?;
		assert_eq!(result, "double my_var = 42 + other_var;\n");
		result = generate_variable(&node, true)?;
		assert_eq!(result, "my_var = 42 + other_var;\n");
		node.children = vec![
			Node::single(NodeType::SYMBOL, Token::val(TokenType::LIT_STR, some("test"))),
			Node::single(NodeType::SYMBOL, Token::val(TokenType::SYMBOL, some("+"))),
			Node::single(NodeType::SYMBOL, Token::val(TokenType::SYMBOL, some("other_var"))),
		];
		result = generate_variable(&node, false)?;
		assert_eq!(result, "::std::string my_var = \"test\" + other_var;\n");
		result = generate_variable(&node, true)?;
		assert_eq!(result, "my_var = \"test\" + other_var;\n");
		Ok(())
	}

	#[test]
	fn test_generate_script() -> Result<(), Error> {
		let node = Node::single(
			NodeType::SCRIPT,
			Token::val(TokenType::SCRIPT, some("echo hello world")),
		);
		let mut result = generate_script(&node, &["var1"])?;
		assert_eq!(
			result,
			"__SYSTEM(R\"__DOIT__(echo hello world)__DOIT__\", ::doit::args_map({__VAR(var1)}));\n"
		);
		result = generate_script(&node, &["var1", "var2", "var3"])?;
		assert_eq!(
			result,
			"__SYSTEM(R\"__DOIT__(echo hello world)__DOIT__\", ::doit::args_map({__VAR(var1),__VAR(var2),__VAR(var3)}));\n"
		);
		Ok(())
	}

	#[test]
	fn test_generate_comment() -> Result<(), Error> {
		let node = Node::single(NodeType::COMMENT, Token::val(TokenType::COMMENT, some(" comment")));
		let node = generate_comment(&node)?;
		assert_eq!(node, "// comment\n");
		Ok(())
	}

	#[test]
	fn test_generate_exit() -> Result<(), Error> {
		let mut node = Node::single(NodeType::EXIT, Token::sym(TokenType::EXIT));
		node.children = vec![Node::single(
			NodeType::SYMBOL,
			Token::val(TokenType::LIT_NUM, some("42")),
		)];
		let result = generate_exit(&node)?;
		assert_eq!(result, "exit(42);\n");
		node.children = vec![
			Node::single(NodeType::SYMBOL, Token::val(TokenType::SYMBOL, some("my_var"))),
			Node::single(NodeType::SYMBOL, Token::val(TokenType::SYMBOL, some("+"))),
			Node::single(NodeType::SYMBOL, Token::val(TokenType::LIT_NUM, some("21"))),
		];
		let result = generate_exit(&node)?;
		assert_eq!(result, "exit(my_var + 21);\n");
		Ok(())
	}
}
