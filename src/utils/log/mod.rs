#![allow(dead_code)]

pub fn debug(message: &str) {
	if cfg!(debug_assertions) {
		println!("\x1b[42m DBG \x1b[0m \x1b[32m{message}\x1b[0m")
	}
}
pub fn info(message: &str) {
	println!("\x1b[44m INF \x1b[0m {message}")
}
pub fn warn(message: &str) {
	eprintln!("\x1b[43m WRN \x1b[0m \x1b[33m{message}\x1b[0m")
}
pub fn error(message: &str) {
	eprintln!("\x1b[41m ERR \x1b[0m \x1b[31m{message}\x1b[0m")
}
