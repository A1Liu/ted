#![allow(dead_code)]
#![allow(unused_variables)]

mod graphics;
mod large_text;
mod text;
mod util;

fn main() {
    println!("size is: {}", std::mem::size_of::<Option<u64>>());
    println!(
        "better size is: {}",
        std::mem::size_of::<Option<util::Idx>>()
    );
}
