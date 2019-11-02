use colored::*;

pub fn print_error(description: &str) {
    eprintln!("{}: {}", "Error".bold().red(), description);
}

pub fn print_warning(description: &str) {
    eprintln!("{}: {}", "Warning".bold().yellow(), description);
}

pub fn print_info_1(message: &str) {
    println!("{} {}", "::".blue(), message);
}

pub fn print_info_2(message: &str) {
    println!("{} {}", "->".blue(), message);
}
