pub fn is_whitespace(value: &char) -> bool {
	value.is_whitespace()
}

pub fn is_number(value: &char) -> bool {
	value.is_numeric() || *value == '.'
}

/** first char must be alpha, subsequent chars can be '_' or digits */
pub fn is_nomenclature(value: &char, first: bool) -> bool {
	value.is_alphabetic() || (!first && (value.is_numeric() || *value == '_' || *value == '-'))
}

#[cfg(test)]
mod tests {
	use crate::lexer::validation::is_number;
	use super::{is_nomenclature, is_whitespace};
	#[test]
	fn test_is_whitespace() {
		" \n\t\r".chars().for_each(|c|assert!(is_whitespace(&c)));
		"abcdefghijklmnopqrstuvwxyz!@#$%^&*()-=`~'\":;/\\,<>+_".chars().for_each(|c|assert!(!is_whitespace(&c)));
	}
	#[test]
	fn test_is_number() {
		"0123456789.".chars().for_each(|c|assert!(is_number(&c)));
		"abcdefghijklmnopqrstuvwxyz \r\n\t!@#$%^&*()-=`~'\":;/\\,<>+_".chars().for_each(|c|assert!(!is_number(&c)));
	}
	#[test]
	fn test_is_nomenclature() {
		"abcdefghijklmnopqrstuvwxyz0123456789_".chars().for_each(|c|assert!(is_nomenclature(&c, false)));
		"abcdefghijklmnopqrstuvwxyz".chars().for_each(|c|assert!(is_nomenclature(&c, true)));
		" \r\n\t!@#$%^&*()-=`~'\":;/\\,<>+".chars().for_each(|c|assert!(!is_nomenclature(&c, false)));
	}
}
