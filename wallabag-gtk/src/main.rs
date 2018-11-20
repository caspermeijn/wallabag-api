
extern crate wallabag_api;

use wallabag_api::add_one;

fn main() {
    let num = 10;

    println!("Hello, world! {} plus one is {}!", num, add_one(num));
}
