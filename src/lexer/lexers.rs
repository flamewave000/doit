use super::{
	token::TokenType,
	validation::{is_nomenclature, is_number},
};

pub trait Consumer {
	fn consume(&mut self) -> Result<char, String>;
	fn consume_and_ignore(&mut self) -> Result<(), String>;
	fn peek(&self) -> Option<char>;
	fn look_ahead(&self, ahead: usize) -> Option<char>;
}

struct VecCharConsumer<'consumer> {
	data: &'consumer Vec<char>,
	position: usize
}
impl VecCharConsumer<'_> {
	pub fn new<'new>(data: &'new Vec<char>) -> VecCharConsumer<'new> {
		VecCharConsumer::<'new> { data, position: 0 }
	}
}

impl Consumer for VecCharConsumer<'_> {
	fn consume(&mut self) -> Result<char, String> {
		match self.data.get(self.position) {
			Some(c) => { self.position += 1; Ok(*c) },
			None => Err("end of string".to_string()),
		}
	}
	fn consume_and_ignore(&mut self) -> Result<(), String> {
		if self.position >= self.data.len() { return Err("end of string".to_string()); }
		self.position += 1;
		Ok(())
	}
	fn peek(&self) -> Option<char> { self.look_ahead(1) }
	fn look_ahead(&self, ahead: usize) -> Option<char> { self.data.get(self.position + (ahead - 1)).copied() }
}

pub fn scan_whitespace(consumer: &mut dyn Consumer) -> Result<(), String> {
	loop {
		let next = consumer.peek().unwrap_or('\0');
		if next == '\n' || !next.is_whitespace() {
			break;
		}
		consumer.consume_and_ignore()?;
	}
	Ok(())
}

pub fn conditional_reader(consumer: &mut dyn Consumer, predicate: fn(&char) -> bool) -> Result<Vec<char>, String> {
	let mut chars: Vec<char> = vec![];
	loop {
		let next = consumer.peek().unwrap_or('\0');
		if !predicate(&next) {
			break;
		}
		chars.push(consumer.consume()?);
	}
	Ok(chars)
}

pub fn read_number(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
	conditional_reader(consumer, is_number)
}

pub fn read_nomenclature(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
	conditional_reader(consumer, |c| is_nomenclature(c, false))
}

pub fn read_string(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
	let mut chars: Vec<char> = vec![];
	// Ignore the first character which is a '"'
	consumer.consume_and_ignore()?;
	loop {
		let next = consumer.consume()?;
		if next == '"' {
			return Ok(chars);
		} else if next == '\\' {
			match consumer.peek().unwrap_or('\0') {
				'\\' | '"' => chars.push(consumer.consume()?),
				_ => return Err("Unexpected escape character".to_string()),
			}
		} else {
			chars.push(next);
		}
	}
}

fn read_delimited_block(consumer: &mut dyn Consumer, delimiter: char, ttype: TokenType) -> Result<Vec<char>, String> {
	consumer.consume_and_ignore()?;
	consumer.consume_and_ignore()?;
	consumer.consume_and_ignore()?;
	// ignore whitespace until we reach an EOL
	if let Err(err) = scan_whitespace(consumer) {
		return Err(err.to_string());
	}
	match consumer.peek() {
		Some(next) => {
			if next != '\n' {
				return Err(format!(
					"Did not find EOL. {ttype:?} block symbols must not be followed by any other content"
				));
			}
			consumer.consume_and_ignore()?;
		}
		None => return Err(format!("Unexpected EOF when expecting {ttype:?} block text")),
	}

	let mut result: Vec<char> = vec![];
	let mut current: char;
	loop {
		current = consumer.consume()?;
		if current == delimiter {
			let la2 = consumer.look_ahead(1);
			let la3 = consumer.look_ahead(2);
			if la2.is_some() && la2.unwrap() == delimiter && la3.is_some() && la3.unwrap() == delimiter {
				consumer.consume_and_ignore()?;
				consumer.consume_and_ignore()?;
				break;
			}
		}
		result.push(current);
	}
	// Remove the last newline character
	if let Some(index) = result.iter().rposition(|x| *x == '\n') {
		result.remove(index);
	}
	Ok(result)
}

pub fn read_comment(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
	if consumer.look_ahead(1).unwrap_or('\0') == '#' && consumer.look_ahead(2).unwrap_or('\0') == '#' {
		return read_delimited_block(consumer, '#', TokenType::SCR_SH);
	}
	// ignore the first '#' character
	consumer.consume_and_ignore()?;
	conditional_reader(consumer, |x| *x != '\n')
}

pub fn read_script(consumer: &mut dyn Consumer, delimiter: char) -> Result<Vec<char>, String> {
	if consumer.look_ahead(1).unwrap_or('\0') == delimiter && consumer.look_ahead(2).unwrap_or('\0') == delimiter {
		return read_delimited_block(consumer, delimiter, TokenType::SCR_SH);
	}
	// ignore the first '$' character
	consumer.consume_and_ignore()?;
	scan_whitespace(consumer)?;
	conditional_reader(consumer, |x| *x != '\n')
}

pub fn convert_help_block_escapes(parsed: Result<Vec<char>, String>) -> Result<Vec<char>, String> {
	parsed.as_ref()?;
	let text = parsed.unwrap();
	let mut cons = VecCharConsumer::new(&text);
	let mut result: Vec<char> = Vec::new();
	while let Ok(mut next) = cons.consume() {
		// let mut next = c;
		if next != '\\' {
			result.push(next);
			continue;
		}
		next = cons.consume()?;
		match next {
			'n' => result.push('\x0a'),
			'r' => result.push('\x0d'),
			'e' => result.push('\x1B'),
			_ => {
				result.push('\\');
				result.push(next);
			}
		}
	}
	Ok(result)
}
pub fn read_help_block(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
	if consumer.look_ahead(1).unwrap_or('\0') == '@' && consumer.look_ahead(2).unwrap_or('\0') == '@' {
		return convert_help_block_escapes(read_delimited_block(consumer, '@', TokenType::HELP));
	}
	// ignore the first '@' character
	consumer.consume_and_ignore()?;
	scan_whitespace(consumer)?;
	convert_help_block_escapes(conditional_reader(consumer, |x| *x != '\n'))
}

#[cfg(test)]
mod tests {
	use crate::lexer::lexers::{read_comment, read_help_block, read_nomenclature, read_script, read_string, convert_help_block_escapes};

	use super::{read_number, scan_whitespace, Consumer};

	fn read_script_sh(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
		read_script(consumer, '$')
	}
	fn read_script_py(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
		read_script(consumer, '%')
	}

	struct MockConsumer {
		pub index: usize,
		pub source: Vec<char>,
	}
	impl MockConsumer {
		pub fn new(source: &str) -> MockConsumer {
			MockConsumer { index: 0, source: source.chars().collect() }
		}
	}
	impl Consumer for MockConsumer {
		fn consume(&mut self) -> Result<char, String> {
			if self.index >= self.source.len() {
				return Err("End of file".to_string());
			}
			let value = *self.source.get(self.index).unwrap();
			self.index += 1;
			if value == '\r' {
				return self.consume();
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
	fn res<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
		match result {
			Ok(v) => v,
			Err(err) => panic!("{:?}", err.to_string()),
		}
	}

	fn run_test(target: fn(&mut dyn Consumer) -> Result<Vec<char>, String>, test_data: &str) -> String {
		let mut consumer = MockConsumer::new(test_data);
		res(target(&mut consumer)).iter().collect()
	}

	#[test]
	fn test_scan_whitespace() {
		let mut consumer = MockConsumer::new("   abc");
		res(scan_whitespace(&mut consumer));
		assert_eq!(consumer.index, 3);
	}
	#[test]
	fn test_read_number() {
		assert_eq!(run_test(read_number, "1234   "), "1234");
	}
	#[test]
	fn test_read_nomenclature() {
		assert_eq!(run_test(read_nomenclature, "my_target {{"), "my_target");
	}
	#[test]
	fn test_read_string() {
		assert_eq!(run_test(read_string, "\"Hello, world!\"\n"), "Hello, world!");
	}
	#[test]
	fn test_convert_help_block_escapes() {
		fn test(input: &str, expect: &str) {
			let result = convert_help_block_escapes(Ok(input.chars().collect()));
			assert!(result.is_ok());
			assert_eq!(result.unwrap().iter().collect::<String>(), expect);
		}
		test("hello \\e world", "hello \x1b world");
		test("hello \\r world", "hello \x0d world");
		test("hello \\n world", "hello \x0a world");
		test("hello \\f world", "hello \\f world");
		test("hello \\e world \\e", "hello \x1b world \x1b");
		test("hello a world", "hello a world");
		test("Special escape character: \\e", "Special escape character: \x1b");
	}
	#[test]
	fn test_read_comment() {
		assert_eq!(
			run_test(read_comment, "# everything to the end   \n"),
			" everything to the end   ",
		);
		assert_eq!(
			run_test(read_comment, "###\n Everything in here\n### nothing here   "),
			" Everything in here",
		);
	}
	#[test]
	fn test_read_script_sh() {
		assert_eq!(run_test(read_script_sh, "$ echo 1234 | cat  \n"), "echo 1234 | cat  ");
		assert_eq!(
			run_test(
				read_script_sh,
				"$$$\necho 1234 | cat\necho hello world\n$$$\n nothing here   "
			),
			"echo 1234 | cat\necho hello world",
		);
	}
	fn test_read_script_py() {
		assert_eq!(run_test(read_script_sh, "% print(\"hello world\")  \n"), "print(\"hello world\")  ");
		assert_eq!(
			run_test(
				read_script_sh,
				"%%%\noutput='1234'\nprint(output)\n%%%\n nothing here   "
			),
			"output='1234'\nprint(output)",
		);
	}
	#[test]
	fn test_read_help_block() {
		assert_eq!(run_test(read_help_block, "@ everything to the end   \n"), "everything to the end   ");
		assert_eq!(run_test(read_help_block, "@@@\n Everything in here\n@@@ nothing here   "), " Everything in here");
		assert_eq!(run_test(read_help_block, "@ Special escape character: \\e\n"), "Special escape character: \x1b");
	}
}
