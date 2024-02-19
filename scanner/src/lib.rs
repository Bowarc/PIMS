use shared::message::PayloadMessage;
use winapi::shared::minwindef::HINSTANCE;

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
    read(utils::get_program_base(), 10);
}

fn read(base: *const u8, size: u8){
    if base.is_null(){
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
        print!("{:02x} ", unsafe {*base.offset(i)});
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

