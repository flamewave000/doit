
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn calculate_hash<T: Hash>(data: &T) -> u64 {
	let mut hasher = DefaultHasher::new();
	data.hash(&mut hasher);
	hasher.finish()
}
