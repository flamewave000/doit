use core::fmt;
use std::fmt::Debug;

use crate::lexer::token::Token;

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq)]
pub enum NodeType {
	ROOT,
	EXIT,
	YIELD,
	ASSIGN,
	TARGET,
	SCR_SH,
	SCR_PY,
	COMMENT,
	SYMBOL,
	ARG_REQ,
	ARG_OPT,
}

pub struct Node {
	pub value: Token,
	pub help: Option<Token>,
	pub ntype: NodeType,
	pub children: Vec<Node>,
}
impl fmt::Display for Node {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str(&self.print("", false, true))
	}
}
impl Node {
	fn print(&self, indent: &str, last: bool, root: bool) -> String {
		let mut result = format!(
			"{}{}{:?}{}: {}\n",
			indent,
			if root { "" } else if last { "└─" } else { "├─" },
			self.ntype,
			if self.help.is_some() { " [HAS_HELP]" } else { "" },
			self.value.value.as_ref().unwrap_or(&String::new()),
		);
		let mut new_indent = format!("{}│ ", indent);
		if last {
			new_indent.pop();
			new_indent.pop();
			new_indent.push_str("  ");
		}
		for (index, child) in self.children.iter().enumerate() {
			result += &child.print(if root {indent} else {&new_indent}, index + 1 >= self.children.len(), false);
		}
		result
	}

	pub const fn new(ntype: NodeType, value: Token, children: Vec<Node>) -> Node {
		Node { ntype, value, children, help: None }
	}
	pub const fn single(ntype: NodeType, value: Token) -> Node {
		Node::new(ntype, value, vec![])
	}
}
