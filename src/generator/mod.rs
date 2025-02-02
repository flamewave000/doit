use std::{
	fmt::Write,
	io::{Error, ErrorKind},
};

use crate::parser::{
	nodes::{Node, NodeType},
	Parser,
};

use self::generators::{generate_comment, generate_exit, generate_script, generate_variable, node_value};

mod generators;
mod sources;

struct TargetArg {
	pub required: bool,
	pub arg_name: String,
	pub arg_help: String,
}
struct Target {
	pub target_name: String,
	pub target_help: String,
	pub target_args: Vec<TargetArg>,
}

pub struct Generator<'generator> {
	pub parser: &'generator mut Parser<'generator>,
}

impl Generator<'_> {
	pub fn new<'new>(parser: &'new mut Parser<'new>) -> Generator<'new> {
		Generator::<'new> { parser }
	}
	#[allow(clippy::only_used_in_recursion)]
	fn generate_scope(
		&mut self,
		indent: &str,
		nodes: &Vec<Node>,
		vars: &[&str],
		tgts: &mut Vec<Target>,
	) -> Result<String, Error> {
		let mut result = String::new();
		let mut locals: Vec<&str> = vec![];
		locals.resize(vars.len(), "");
		locals.copy_from_slice(vars);
		for node in nodes {
			match node.ntype {
				NodeType::ROOT => return Err(Error::new(ErrorKind::InvalidData, "Unexpected ROOT node")),
				NodeType::EXIT => {
					result.push_str(indent);
					result.push_str(&generate_exit(node)?);
				}
				NodeType::ASSIGN => {
					let var_name = node.value.value.as_ref().unwrap();
					let exists = locals.contains(&var_name.as_str());
					if !exists {
						locals.push(node.value.value.as_ref().unwrap());
					}
					result.push_str(indent);
					result.push_str(&generate_variable(node, exists)?);
				}
				NodeType::TARGET => {
					let name = node_value(node).to_string();
					result.push_str(&format!("{}void {}(int argc, const char *argv[]) {{\n", indent, &name));
					tgts.push(Target{
						target_name: name,
						target_help: node.help
							.as_ref()
							.and_then(|h| h.value.as_ref())
							.unwrap_or(&"<No help defined>".to_string())
							.clone(),
						target_args: node.children.iter()
						.filter(|it| { it.ntype == NodeType::ARG_REQ || it.ntype == NodeType::ARG_OPT })
						.map(|it| {
							let required: bool;
							if it.ntype == NodeType::ARG_REQ {
								required = true;
							} else if it.ntype == NodeType::ARG_OPT {
								required = false;
							} else {
								panic!("invalid node type");
							}
							TargetArg{
								required,
								arg_name: it.value.value.clone().expect("argument missing name"),
								arg_help: it.help.as_ref().and_then(|h| h.value.as_ref())
										.unwrap_or(&"<No help defined>".to_string())
										.clone(),
							}
						}).collect(),
					});
					result.push_str(&self.generate_scope(
						&(indent.to_string() + "\t"),
						&node.children,
						&locals,
						tgts,
					)?);
					result.push_str(&format!("{}}}\n", indent));
				}
				NodeType::SCRIPT => {
					result.push_str(indent);
					result.push_str(&generate_script(node, &locals)?);
				}
				NodeType::COMMENT => {
					result.push_str(indent);
					result.push_str(&generate_comment(node)?);
				}
				NodeType::SYMBOL => continue,
				NodeType::ARG_OPT => continue,
				NodeType::ARG_REQ => continue,
			}
		}
		Ok(result)
	}

	pub fn generate(&mut self) -> Result<String, Error> {
		let mut source = (sources::SOURCE_FILE).to_string();
		let mut targets: Vec<Target> = vec![];
		let root_node = self.parser.parse()?;

		// Generate the definitions
		let definitions = self.generate_scope("\t", &root_node.children, &[], &mut targets)?;
		source = source.replace("{{{TARGET_DEFINITIONS}}}", &definitions);

		// Generate the help
		let root_help = match root_node.help.as_ref().and_then(|help| help.value.as_ref()) {
			Some(v) => v.as_str(),
			None => "",
		};
		source = source.replace("{{{ROOT_HELP}}}", root_help);

		let help_text = targets
			.iter()
			.fold(String::new(), |mut output: String, target: &Target| {
				write!(output, "\n\t\t__HELP({}, R\"__DOIT__({})__DOIT__\"", target.target_name, target.target_help)
					.expect("failed to generate code");
				output = target.target_args.iter().fold(output, |mut output: String, arg: &TargetArg| {
					write!(
						output,
						", __ARG({}, R\"__DOIT__({})__DOIT__\", R\"__DOIT__({})__DOIT__\")",
						arg.required, arg.arg_name, arg.arg_help
					).expect("failed to generate code");
					output
				});
				write!(output, "),").expect("failed to generate code");
				output
			});
		source = source.replace("{{{TARGET_HELPS}}}", &help_text);

		// Generate Target Matches
		source = source.replace(
			"{{{TARGET_MATCHES}}}",
			&targets.iter().fold(
				String::new(),
				|mut output: String, tgt: &Target| {
					let _ = write!(output, "\n\t__MATCH({});", tgt.target_name);
					output
				},
			),
		);

		Ok(sources::DOIT_HEADER.to_string() + &source)
	}
}
