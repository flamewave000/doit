use std::io::{Error, ErrorKind};

use crate::{lexer::token::TokenType, parser::nodes::{Node, NodeType}};

pub fn node_value(node: &Node) -> &str {
	let value = node.value.value.as_ref();
	return match value {
		Some(res) => AsRef::as_ref(res),
		None => "",
	};
}

pub fn generate_variable(node: &Node) -> Result<String, Error> {
	if node.value.value.is_none() {
		return Err(Error::new(ErrorKind::InvalidData, ""));
	}
	let mut expression = String::new();
	for child in &node.children {
		if child.ntype != NodeType::SYMBOL {
			return Err(Error::new(ErrorKind::InvalidData, "Unexpected node type"));
		}
		if let Some(value) = &child.value.value {
			if child.value.ttype == TokenType::LIT_STR {
				expression.push('"');
				expression.push_str(&value);
				expression.push('"');
			} else {
				expression.push_str(&value);
			}
		}
	}
	let var_type = if node.children[0].value.ttype == TokenType::LIT_NUM {"double"} else {"::std::string"};
	return Ok(format!("{} {} = {};\n", var_type, node.value.value.as_ref().unwrap(), &expression));
}
pub fn generate_script(node: &Node, vars: &[&str]) -> Result<String, Error> {
	let vars: Vec<String> = vars.iter().map(|var| format!("__VAR({})", *var)).collect();
	return Ok(format!(
		r#"__SYSTEM(R"__DOIT__({})__DOIT__", __VARS({}));{}"#,
		node_value(node),
		vars.join(","),
		'\n'
	));
}
pub fn generate_comment(node: &Node) -> Result<String, Error> {
	return Ok(format!("//{}\n", node.value.value.as_ref().unwrap_or(&String::new())));
}
pub fn generate_exit(node: &Node) -> Result<String, Error> {
	let expression: Vec<&str> = node.children.iter().map(node_value).collect();
	return Ok(format!("exit {};\n", expression.join(" ")));
}