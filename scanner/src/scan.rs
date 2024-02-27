use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

pub fn scan(value_bytes: &Vec<u8>, region: MEMORY_BASIC_INFORMATION) -> Option<Vec<u8>> {
    use winapi::um::winnt::{
        MEM_FREE, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_GUARD, PAGE_READWRITE,
        PAGE_WRITECOPY,
    };

    if value_bytes.is_empty() {
        println!("[ERROR] Couldn't read the bytes of the given value");
        return None;
    }

    const WRITEABLE: u32 =
        PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;

    if region.State == MEM_FREE {
        // println!("Skipping region {r_index} ({:?}) (FREE)", region.BaseAddress);
        return None;
    }

    if region.Protect & PAGE_GUARD != 0 {
        // println!("Skipping region {r_index} ({:?}) (PAGE GUARD)", region.BaseAddress);
        return None;
    }

    if region.Protect & WRITEABLE == 0 {
        // println!("Skipping region {r_index} ({:?}) (NOT WRITEABLE)", region.BaseAddress);
        return None;
    }

    // println!("Scanning region {r_index}, p: {}, s: {}", region.Protect, region.State);
    // println!("Scanning region {r_index}, located at {:?}", region.BaseAddress);

    // Scan the whole region

    let mut found_offset = 0;

    let base_addr = region.BaseAddress as *const u8;

    let mut found_addresses = Vec::new();

    for offset in 0..(region.RegionSize as isize) {
        println!(
            "Checking {} & {}",
            unsafe { *base_addr.offset(offset) },
            value_bytes.get(found_offset).unwrap()
        );

        if *value_bytes.get(found_offset).unwrap() != unsafe { *base_addr.offset(offset) } {
            found_offset = 0;
            continue;
        }

        found_offset += 1;

        println!("Found {found_offset} at {offset}");

        if found_offset == value_bytes.len() {
            let addr = unsafe { base_addr.add((offset as usize + 1) - value_bytes.len()) };
            // println!("Found at index: {addr:?}" );
            found_addresses.push(addr as u8);
            found_offset = 0; // search for more
        }
    }

    Some(found_addresses)
}
