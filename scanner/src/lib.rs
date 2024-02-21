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
        println!("Base: {:?}", r.BaseAddress);
        println!("Size: {:?}", r.RegionSize);
        println!("Alloc protect: {:?}", r.AllocationProtect);
        println!("State: {:?}", r.State);
        println!("Protect: {:?}", r.Protect);
        println!("Type: {:?}", r.Type);
        println!("");
    });

    // Result
    // Found at index: 0xa949afd18c
    // Found at index: 0xa949afd4ea
    // Found at index: 0xa949afd968
    // Found at index: 0xa949aff94c
    // Found at index: 0xa949aff984

    let t = 0x4c6170f524 as *mut c_void; // Target
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

    println!("Target region: {:?}", region.BaseAddress);


    println!("Scan found {:?} places", utils::scan(10u32).len());
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
