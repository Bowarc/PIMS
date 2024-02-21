#[macro_use]
extern crate log;

use dll_syringe::{process::OwnedProcess, Syringe};

fn main() {
    let cfg = logger::LoggerConfig::new().set_level(log::LevelFilter::Debug);
    logger::init(cfg, None);

    info!("Injector start");

    let target_process_name = "dummy";

    let target_process = OwnedProcess::find_first_by_name(target_process_name).unwrap();

    info!("Found the {target_process_name} process");

    let syringe = Syringe::for_process(target_process);

    info!("Created syringe");

    let listener = std::net::TcpListener::bind(shared::DEFAULT_ADDRESS).unwrap();

    let dll_path = "target/debug/scanner.dll";

    let injected_payload = syringe.inject(dll_path).unwrap();
    info!("Injected in {target_process_name}\n{injected_payload:?}");

    let (stream, addr) = listener.accept().unwrap();
    let mut socket = networking::Socket::<
        shared::message::PayloadMessage,
        shared::message::ServerMessage,
    >::new(stream);
    info!("{addr} accepted");

    let running = set_up_ctrlc();
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        use shared::message::PayloadMessage;
        let msg = match socket.try_recv() {
            Ok((_header, message)) => message,
            Err(e) => {
                match e{
                    networking::socket::SocketError::TestError => todo!(),
                    networking::socket::SocketError::Serialization(_) => todo!(),
                    networking::socket::SocketError::Deserialization(_) => todo!(),
                    networking::socket::SocketError::StreamWrite(_) => todo!(),
                    networking::socket::SocketError::StreamRead(_) => todo!(), // Dll crash
                }
                debug!("{e}");
                std::thread::sleep(std::time::Duration::from_secs_f64(0.1));
                continue;
            }
        };

        match msg {
            PayloadMessage::Boot => {
                info!("Payload just booted")
            }
            PayloadMessage::Info(txt) => info!("Payload sent: {txt}"),
            PayloadMessage::Exit => {
                info!("Payload as exited")
            }
            PayloadMessage::Eject => {
                info!("Payload asked for extraction");
                break;
            }
            _ => (),
        }
    }
    info!("Unloading {injected_payload:?}");
    syringe.eject(injected_payload).unwrap();
}

pub fn set_up_ctrlc() -> std::sync::Arc<std::sync::atomic::AtomicBool> {
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    running
}
