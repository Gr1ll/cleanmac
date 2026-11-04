use bytesize::ByteSize;

pub fn human_size(bytes: u64) -> String {
    ByteSize(bytes).display().si().to_string()
}

pub fn print_biggest_header() {
    println!("\n{:<5} {:>12}  {}", "#", "Size", "Path");
    println!("{}", "─".repeat(80));
}

pub fn print_biggest_row(index: usize, size: u64, path: &str) {
    println!("{:<5} {:>12}  {}", index, human_size(size), path);
}

pub fn print_cache_header() {
    println!("\n{:<26} {:>12}  {}", "Cache", "Size", "Path");
    println!("{}", "─".repeat(90));
}

pub fn print_cache_row(name: &str, size: u64, path: &str) {
    let size_str = if size == 0 {
        "not found".to_string()
    } else {
        human_size(size)
    };
    println!("{:<26} {:>12}  {}", name, size_str, path);
}

pub fn print_apps_header() {
    println!("\n{:<34} {:<10}  {}", "App", "Source", "Path");
    println!("{}", "─".repeat(90));
}

pub fn print_apps_row(name: &str, source: &str, path: &str) {
    println!("{:<34} {:<10}  {}", name, source, path);
}
