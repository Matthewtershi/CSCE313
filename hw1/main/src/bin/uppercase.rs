fn main() { 
    let input = std::env::args().nth(1).expect("Please provide a string argument");
    let mut output = String::new();
    for c in input.chars() {
        if c.is_alphanumeric() {
            output.extend(c.to_uppercase());
        }
    }
    println!("{}", output);
}
