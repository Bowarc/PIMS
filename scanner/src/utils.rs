use winapi::um::{sysinfoapi::SYSTEM_INFO, };
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;
pub fn program_base() -> *const u8 {
    unsafe { winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null()) as *const u8 }
}

pub fn page_size() -> usize {
    use winapi::um::sysinfoapi::GetSystemInfo;
    use winapi::um::sysinfoapi::LPSYSTEM_INFO;
    unsafe {
        let mut info: SYSTEM_INFO = std::mem::zeroed();
        GetSystemInfo(&mut info as LPSYSTEM_INFO);

        info.dwPageSize as usize
    }
}

pub fn str_ptr(s: &str) -> *const i8 {
    format!("{s}\0").as_ptr() as *const i8
}

// https://github.com/darfink/region-rs/blob/68c137d6e752c4ab12626850bf46cd0c3df6799d/src/os/windows.rs#L150
pub fn system_info() -> &'static SYSTEM_INFO {
    use std::mem::MaybeUninit;
    use std::sync::Once;
    use winapi::um::sysinfoapi::GetNativeSystemInfo;

    static INIT: Once = Once::new();
    static mut INFO: MaybeUninit<SYSTEM_INFO> = MaybeUninit::uninit();

    unsafe {
        INIT.call_once(|| GetNativeSystemInfo(INFO.as_mut_ptr()));
        &*INFO.as_ptr()
    }
}

pub fn get_all_regions() -> Vec<MEMORY_BASIC_INFORMATION> {
    use winapi::ctypes::c_void;
    use winapi::um::memoryapi::VirtualQuery;

    let sysinfo = system_info();

    let mut out = Vec::new();

    let mut ptr = sysinfo.lpMinimumApplicationAddress;

    loop {
        if ptr >= sysinfo.lpMaximumApplicationAddress {
            // println!("Ptr is out of application address range");
            break;
        }

        let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        let read = unsafe {
            VirtualQuery(
                ptr as winapi::um::winnt::PVOID,
                &mut info,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>() as winapi::shared::basetsd::SIZE_T,
            )
        };

        if read == 0 {
            println!("Read 0, exiting");
            break;
        }
        out.push(info);

        // Move the ptr to the next page
        ptr = (info.BaseAddress as usize).saturating_add(info.RegionSize) as *mut c_void;

    }

    out
}

pub fn scan<T>(value: T) -> Vec<*const u8> {
    use winapi::um::winnt::{
        MEM_COMMIT, PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
        PAGE_EXECUTE_WRITECOPY, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY,
        MEM_FREE,PAGE_GUARD
    };

    let value_bytes = unsafe {
        std::slice::from_raw_parts(&value as *const T as *const u8, std::mem::size_of::<T>())
    };

    if value_bytes.is_empty(){
        println!("[ERROR] Couldn't read the bytes of the given value");
        return Vec::new();
    }

    let mut out = Vec::new();

    const WRITEABLE: u32 =
        PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;

    let regions = get_all_regions();

    for (r_index, region) in regions.iter().enumerate() {
        // if (region.State & MEM_COMMIT) == 0 {
        //     println!("Skipping region {r_index} ({:?})(not commit)", region.BaseAddress);
        //     continue;
        // }else if (region.Protect & WRITEABLE) != 0 {
        //     println!("Skipping region {r_index} ({:?})(not WRITEABLE)", region.BaseAddress);
        //     continue;
        // }
        println!("");
        println!("Writable: {WRITEABLE:b}");
        println!("Base: {:?}", region.BaseAddress);
        println!("Size: {:?}", region.RegionSize);
        println!("Alloc protect: {:?}", region.AllocationProtect);
        println!("State: {:?}", region.State);
        println!("Protect: {:b}", region.Protect);
        println!("Type: {:?}", region.Type);

        // if !((region.State == MEM_COMMIT /*0x1000*/ ) && (region.Protect == 4096/*& WRITEABLE ==0*/)){
        //     println!("Skipping region {r_index} ({:?})", region.BaseAddress);
        //     continue;
        // }

        if region.State == MEM_FREE{   
            println!("Skipping region {r_index} ({:?}) (FREE)", region.BaseAddress);
            continue;
        }

        if region.Protect & PAGE_GUARD != 0{
            println!("Skipping region {r_index} ({:?}) (PAGE GUARD)", region.BaseAddress);
            continue;
        }
        println!("GUARD: {}", region.Protect & PAGE_GUARD);

        if region.Protect & WRITEABLE == 0{
            println!("Skipping region {r_index} ({:?}) (NOT WRITEABLE)", region.BaseAddress);
            continue;
        } 


        // println!("Scanning region {r_index}, p: {}, s: {}", region.Protect, region.State);
        println!("Scanning region {r_index}, located at {:?}", region.BaseAddress);

        // Scan the whole region

        let mut found_index = 0;

        let base_addr= region.BaseAddress as *const u8;

        for index in 0..(region.RegionSize as isize) {
            let b = unsafe { *base_addr.offset(index) };

            // println!("Checking {b} & {}", val.get(found_index).unwrap());
            if b == *value_bytes.get(found_index).unwrap() {
                // println!("Found {found_index} at {index}");
                found_index += 1;
            } else {
                found_index = 0;
            }
            if found_index == value_bytes.len() {
                let addr = unsafe{base_addr.add((index as usize+ 1) - found_index)};
                // println!("Found at index: {addr:?}" );
                out.push(addr);
                found_index = 0; // search for more
            }
        }
    }

    out
}
