#[no_mangle]
pub extern "C" fn rs_is_finishing(flags: u8) -> i32 {
    (flags & 0x01) as i32
}
