use shared::message::PayloadMessage;
use winapi::{ctypes::c_void, shared::minwindef::HINSTANCE};

mod scan;
mod utils;

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_PROCESS_DETACH: u32 = 0;

static mut SOCKET: Option<
    networking::Socket<shared::message::ServerMessage, shared::message::PayloadMessage>,
> = None;

fn debug(msg: String) {
    socket_send(PayloadMessage::Info(msg));
}

fn socket_send(msg: PayloadMessage) {
    unsafe { SOCKET.as_mut().unwrap().send(msg).unwrap() };
}

#[no_mangle]
extern "system" fn DllMain(_dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    if unsafe { SOCKET.is_none() } {
        unsafe {
            SOCKET = Some(networking::Socket::<
                shared::message::ServerMessage,
                shared::message::PayloadMessage,
            >::new(
                std::net::TcpStream::connect(shared::DEFAULT_ADDRESS).unwrap(),
            ));
        }
    }

    match call_reason {
        DLL_PROCESS_ATTACH => {
            std::thread::spawn(|| {
                println!("Hi");
                socket_send(PayloadMessage::Boot);
                start();
                exit()
            });
        }
        DLL_PROCESS_DETACH => cleanup(),
        _ => (),
    }

    true
}
fn start() {
    // read(utils::get_program_base(), 10);

    utils::get_all_regions().iter().for_each(|r| {
        println!("{:?}", r.BaseAddress);
    });

    // Result
    // Found at index: 0xa949afd18c
    // Found at index: 0xa949afd4ea
    // Found at index: 0xa949afd968
    // Found at index: 0xa949aff94c
    // Found at index: 0xa949aff984

    let t = 0xa949aff984 as *mut c_void; // Target
    let region = utils::get_all_regions()
        .iter()
        .min_by_key(|&r| {

            let diff = if r.BaseAddress as u64 > t as u64 {
                u64::MAX
            } else {
                t as u64 - r.BaseAddress as u64
            };
            println!("{diff} for {:?}", r.BaseAddress);
            diff
        })
        .cloned()
        .unwrap();

    let mut found_index = 0;

    let base_addr = region.BaseAddress as *const u8;
    let value = 10u32;
    let value_bytes = unsafe {
        std::slice::from_raw_parts(
            &value as *const u32 as *const u8,
            std::mem::size_of::<u32>(),
        )
    };
    println!("Found region {:?}, let's scan it", region.BaseAddress);
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
            let addr = unsafe { base_addr.add((index as usize + 1) - found_index) };
            println!("Found at index: {addr:?}");
            // out.push(addr);
            found_index = 0; // search for more
        }
    }

    // println!("Scan found {:?} places", utils::scan(10u32).len());
}

fn read(base: *const u8, size: u8) {
    if base.is_null() {
        println!("No!");
        return;
    }
    println!("Reading {size} bytes at addr: {base:?}");
    use winapi::um::sysinfoapi::GetSystemInfo;
    use winapi::um::sysinfoapi::{LPSYSTEM_INFO, SYSTEM_INFO};
    let x = unsafe {
        let mut info: SYSTEM_INFO = std::mem::zeroed();
        GetSystemInfo(&mut info as LPSYSTEM_INFO);

        info.dwPageSize as usize
    };
    println!("page: {x}");

    for i in 0..(size as isize) {
        print!("{:02x} ", unsafe { *base.offset(i) });
    }
    println!();
}

fn exit() {
    socket_send(PayloadMessage::Eject);
    println!("Asking for extraction");
}

fn cleanup() {
    socket_send(PayloadMessage::Exit);
    unsafe { SOCKET = None };
    println!("Extraction !\n\n\n\n");
}
