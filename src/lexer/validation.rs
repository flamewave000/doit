pub fn is_whitespace(value: &char) -> bool {
	return value.is_whitespace();
}

pub fn is_number(value: &char) -> bool {
	return value.is_numeric() || *value == '.';
}

/** first char must be alpha, subsequent chars can be '_' or digits */
pub fn is_nomenclature(value: &char, first: bool) -> bool {
	return value.is_alphabetic() || (!first && (value.is_numeric() || *value == '_'));
}
