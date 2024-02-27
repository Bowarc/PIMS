use shared::message::{PayloadMessage, ServerMessage};
use winapi::{ctypes::c_void, shared::minwindef::HINSTANCE};

mod scan;
mod utils;

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_PROCESS_DETACH: u32 = 0;

static mut SOCKET: Option<networking::Socket<ServerMessage, PayloadMessage>> = None;

fn debug(msg: String) {
    socket_send(PayloadMessage::Info(msg));
}

fn socket_send(msg: PayloadMessage) {
    unsafe { SOCKET.as_mut().unwrap().send(msg).unwrap() };
}

fn socket_read(
) -> Result<(networking::socket::Header, ServerMessage), networking::socket::SocketError> {
    unsafe { SOCKET.as_mut().unwrap().try_recv() }
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
    loop {
        match socket_read() {
            Ok((_header, ServerMessage::ScanRequest(params))) => {
                let regions = utils::get_all_regions();

                let mut addresses = Vec::new();
                for (index, region) in regions.iter().enumerate() {
                    let Some(found_addresses) = scan::scan(&params.value, *region) else {
                        continue;
                    };

                    addresses.extend(found_addresses);

                    socket_send(PayloadMessage::ScanUpdate(shared::data::ScanInfo {
                        progress: (index + 1, regions.len()),
                        value_size_b: params.value.len() as u8,
                        found_addresses: addresses.clone(),
                    }))
                }
            }
            Ok(_) => {}

            Err(e) => {
                let is_would_block =
                    if let networking::socket::SocketError::StreamRead(ref io_e) = e {
                        io_e.kind() == std::io::ErrorKind::WouldBlock
                    } else {
                        // matches!(e, shared::networking::SocketError::WouldBlock)

                        false
                    };
                if is_would_block {
                    continue;
                }

                return;
            }
        }
    }

    //     println!("Base: {:?}", r.BaseAddress);
    //     println!("Size: {:?}", r.RegionSize);
    //     println!("Alloc protect: {:?}", r.AllocationProtect);
    //     println!("State: {:?}", r.State);
    //     println!("Protect: {:?}", r.Protect);
    //     println!("Type: {:?}", r.Type);
    //     println!("");
    // });

    // Result
    // Found at index: 0xa949afd18c
    // Found at index: 0xa949afd4ea
    // Found at index: 0xa949afd968
    // Found at index: 0xa949aff94c
    // Found at index: 0xa949aff984

    // let t = 0x4c6170f524 as *mut c_void; // Target
    //                                      // println!("Target region: {:?}", region.BaseAddress);

    // let scan_result = utils::scan(458459378u32);
    // println!(
    //     "Scan found {} addresses: {:#?}",
    //     scan_result.len(),
    //     scan_result
    // );

    // let new_data = 15u32;

    // for address in scan_result {
    //     unsafe { std::ptr::write(address as *mut u32, new_data) }
    // }
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
    println!("Extraction !");
}
