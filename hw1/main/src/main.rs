fn main() {
    println!("Hello, world!");
}
// fn main() {
//     let mut s = String::from("hello");
//     let ref1 = &s;
//     let ref2 = &ref1;
//     let ref3 = &ref2;
//     s = String::from("goodbye");
//     println!("{}", ref3.to_uppercase());
// }
// // This program does not compile because it tries to mutate the string s while there are still immutable references to it
// fn drip_drop() -> &String {
//     let s = String::from("hello world!");
//     return &s;
// }
// Won't compile due to dangling pointer. The string s is dropped when the function returns, so the reference returned points to invalid memory.
// fn main() {
//     let s1 = String::from("hello");
//     let mut v = Vec::new();
//     v.push(s1);
//     let s2: String = v[0];
//     println!("{}", s2);
// }
// Indexing a vector gives you access to the value, but it does not let you move it out the vector. You would need to use a reference to the value instead.