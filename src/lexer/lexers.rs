use std::io::Error;

use super::validation::{is_nomenclature, is_number};

pub trait Consumer {
	fn consume(&mut self) -> Result<char, String>;
	fn consume_and_ignore(&mut self) -> Result<(), String>;
	fn peek(&self) -> Option<char>;
	fn look_ahead(&self, ahead: usize) -> Option<char>;
}

pub fn scan_whitespace(consumer: &mut dyn Consumer) -> Result<(), Error> {
	loop {
		let next = consumer.peek().unwrap_or('\0');
		if next == '\n' || !next.is_whitespace() {
			break;
		}
		let _ = consumer.consume_and_ignore();
	}
	return Ok(());
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
	return Ok(chars);
}

pub fn read_number(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
	return conditional_reader(consumer, is_number);
}

pub fn read_nomenclature(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
	return conditional_reader(consumer, |c| is_nomenclature(c, false));
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
	return conditional_reader(consumer, |x| *x != '\n');
}
pub fn read_script(consumer: &mut dyn Consumer) -> Result<Vec<char>, String> {
	// ignore the first '$' character
	consumer.consume_and_ignore()?;
	return conditional_reader(consumer, |x| *x != '\n');
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
	return Ok(result);
}
