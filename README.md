# GOON-RS: **GLOBALS ON OUR NETWORK?!**
That's right! Globals (ie static refs) to share program state on LANs. No need
to touch a web protocol, we do it for you.

## Usage

```rust
use std::time::Duration;

use goon::*;

// any program that has this section 
// will share variable states of the same name:
declare_global!{
    A: u32 = 0;
}

#[goon_init]
fn main() {
    // variables being used must be redeclared
    // in current scope
    global!(A);
    println!("listening for peers...");

    for i in 0..10000 {
        std::thread::sleep(Duration::from_millis(100));
        // local-global variables are 
        // then handle as lowercase to avoid name overlaps
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

```

## TODO



