use super::validation::{is_nomenclature, is_number};

pub trait Consumer {
	fn consume(&mut self) -> Result<char, String>;
	fn consume_and_ignore(&mut self) -> Result<(), String>;
	fn peek(&self) -> Option<char>;
	fn look_ahead(&self, ahead: usize) -> Option<char>;
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

pub fn read_comment(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
	// ignore the first '#' character
	consumer.consume_and_ignore()?;
	conditional_reader(consumer, |x| *x != '\n')
}

pub fn read_script(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
	// ignore the first '$' character
	consumer.consume_and_ignore()?;
	scan_whitespace(consumer)?;
	conditional_reader(consumer, |x| *x != '\n')
}

pub fn read_help_block(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
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
				return Err(
					"Did not find EOL. Help block symbols must not be followed by any other content".to_string(),
				);
			}
			consumer.consume_and_ignore()?;
		}
		None => return Err("Unexpected EOF when expecting help text".to_string()),
	}

	let mut result: Vec<char> = vec![];
	let mut current: char;
	loop {
		current = consumer.consume()?;
		if current == '@' {
			let la2 = consumer.look_ahead(1);
			let la3 = consumer.look_ahead(2);
			if la2.is_some() && la2.unwrap() == '@' && la3.is_some() && la3.unwrap() == '@' {
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


#[cfg(test)]
mod tests {
	use crate::lexer::lexers::{read_comment, read_help_block, read_nomenclature, read_script, read_string};

use super::{read_number, scan_whitespace, Consumer};

	struct MockConsumer {
		pub index: usize,
		pub source: Vec<char>
	}
	impl MockConsumer {
		pub fn new(source: &str) -> MockConsumer { return MockConsumer{index:0, source: source.chars().collect()} }
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
			return Ok(value);
		}
		fn consume_and_ignore(&mut self) -> Result<(), String> {
			let _ = self.consume()?;
			return Ok(())
		}
		fn peek(&self) -> Option<char> {
			return self.source.get(self.index).copied();
		}
		fn look_ahead(&self, ahead: usize) -> Option<char> {
			if ahead == 0 { panic!("ahead parameter must be greater than zero"); }
			return self.source.get(self.index + (ahead - 1)).copied();
		}
	}
	fn res<T, E : std::fmt::Display>(result: Result<T, E>) -> T {
		match result {
			Ok(v) => v,
			Err(err) => panic!("{:?}", err.to_string())
		}
	}

	fn run_test(target: fn(&mut dyn Consumer) -> Result<Vec<char>, String>, test_data: &str, expected: &str) {
		let mut consumer = MockConsumer::new(test_data);
		let number: String = res(target(&mut consumer)).iter().collect();
		assert_eq!(number, expected);
	}

	#[test]
	fn test_scan_whitespace() {
		let mut consumer = MockConsumer::new("   abc");
		res(scan_whitespace(&mut consumer));
		assert_eq!(consumer.index, 3);
	}
	#[test]
	fn test_read_number() {
		run_test(read_number, "1234   ", "1234");
	}
	#[test]
	fn test_read_nomenclature() {
		run_test(read_nomenclature, "my_target {{", "my_target");
	}
	#[test]
	fn test_read_string() {
		run_test(read_string, "\"Hello, world!\"\n", "Hello, world!");
	}
	#[test]
	fn test_read_comment() {
		run_test(read_comment, "# everything to the end   \n", " everything to the end   ");
	}
	#[test]
	fn test_read_script() {
		run_test(read_script, "$ echo 1234 | cat  \n", "echo 1234 | cat  ");
	}
	#[test]
	fn test_read_help_block() {
		run_test(read_help_block, "@@@\n Everything in here\n@@@ nothing here   ", " Everything in here");
	}
}