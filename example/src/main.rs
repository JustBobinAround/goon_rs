use std::time::Duration;

use goon::*;

declare_global!{
    A: u32 = 0;
}

#[goon_init]
fn main() {
    global!(A);
    println!("listening for peers...");

    for i in 0..10000 {
        std::thread::sleep(Duration::from_millis(100));
        lock_globals!(|a| => {
            println!("sending update");
            *a= i;
        });
        std::thread::sleep(Duration::from_millis(100));
        read_globals!(|a| => {
            println!("reading current val: {}", *a);
        });
    }
}
