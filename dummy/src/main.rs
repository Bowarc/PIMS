fn main() {
    println!("Hello, world!");

    let mut x = 458459378u32;


    println!("[Dummy] X addr: {:p}", &x);
    let view = unsafe { (&x as *const _ as *const u8).offset(-1000) };
    let size = std::mem::size_of_val(&x);
    // println!("\n{size} bytes of x ({view:?})");
    // let mut mem = Vec::new();
    for i in 0..(size + 2000) {
        // print!("{:02b} ", unsafe { *view.offset(i.try_into().unwrap()) });
        // mem.push(unsafe { *view.offset(i.try_into().unwrap()) });
    }

    // find_value(
    //     &[0b1010, 0b00, 0b00, 0b00],
    //     unsafe { (&x as *const _ as *const u8).offset(0) },
    //     5000,
    // );

    println!();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("Val of x: {x}");
    }
}

fn read() {
    let view = main as *const u8;
    println!("\n10 bytes of main ({view:?})");
    for i in 0..100 {
        print!("{:02x} ", unsafe { *view.offset(i) });
    }
}

fn find_value(val: &[u8], base_addr: *const u8, max_len: usize) {
    let val_size = val.len();
    println!("\nSearching for {val:?}, from {:?} to {:?}", base_addr, unsafe{base_addr.add(max_len)});

    let mut found_index = 0;

    for index in 0..(max_len as isize) {
        let b = unsafe { *base_addr.offset(index) };

        // println!("Checking {b} & {}", val.get(found_index).unwrap());
        if b == *val.get(found_index).unwrap() {
            println!("Found {found_index} at {index}");
            found_index += 1;
        } else {
            found_index = 0;
        }
        if found_index == val_size {
            let addr = unsafe{base_addr.add((index as usize+ 1) - found_index)};
            println!("Found at index: {addr:?}",);
            println!("{}", unsafe{std::ptr::read(addr)});
            break;
        }
    }


    println!("\nexit scan");
}
