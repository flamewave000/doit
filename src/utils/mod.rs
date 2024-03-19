pub mod log;

pub fn find<T : PartialEq>(array: &[T], value: &T) -> Option<usize> {
	return array.iter().position(|x| x.eq(value))
}