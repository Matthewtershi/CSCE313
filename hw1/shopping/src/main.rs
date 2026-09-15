use std::io;
use std::io::Write;

fn main() {
    let mut input = String::new();
    let mut list: Vec<String> = Vec::new();

    io::stdout().write_all(b"Enter your shopping list, one item at a time.\n").unwrap();
    loop {
        input.clear();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        let item = input.trim();
        if item == "done" {
            break;
        }
        list.push(item.to_string());
    }

    for item in &list {
        println!("* {}", item);
    }
}
