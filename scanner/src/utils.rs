pub fn get_program_base() -> *const u8 {
    unsafe { winapi::um::libloaderapi::GetModuleHandleA(std::ptr::null()) as *const u8 }
}

pub fn str_ptr(s: &str) -> *const i8 {
    format!("{s}\0").as_ptr() as *const i8
}
