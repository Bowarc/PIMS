use dll_syringe::{
    process::{BorrowedProcess, OwnedProcess, ProcessModule},
    Syringe,
};

pub const DEFAULT_DLL_PATH: &str = "./target/debug/scanner.dll";

pub fn inject<'a>(
    dll_path: &str,
    target_name: &str,
) -> bool {
    let Some(target_process) = OwnedProcess::find_first_by_name(target_name) else{
        return false;
    };

    info!("Found the {target_name} process");

    let syringe = Syringe::for_process(target_process);

    info!("Created syringe");

    let injected_payload = syringe.inject(dll_path).unwrap();
    info!("Injected in {target_name}\n{injected_payload:?}");

    true
    // info!("Unloading {injected_payload:?}");
    // syringe.eject(injected_payload).unwrap();
}
