const FLAG_IS_FINISHING: u8 = 1;

#[repr(u8)]
enum HSEState {
    NotFull,
    Filled,
    Search,
    YieldTagBit,
    YieldLiteral,
    YieldBrIndex,
    YieldBrLength,
    SaveBacklog,
    FlushBits,
    Done,
}

#[no_mangle]
pub extern "C" fn rs_heatshrink_encoder_finish(flags: &mut u8, state: &mut u8) -> i32 {
    *flags |= FLAG_IS_FINISHING;

    if *state == HSEState::NotFull as u8 {
        *state = HSEState::Filled as u8;
    }

    if *state == HSEState::Done as u8 {
        0
    } else {
        1
    }
}

#[no_mangle]
pub extern "C" fn rs_is_finishing(flags: u8) -> i32 {
    (flags & FLAG_IS_FINISHING) as i32
}

#[no_mangle]
pub extern "C" fn rs_can_take_byte(output_size: usize, buff_size: usize) -> i32 {
    (output_size < buff_size) as i32
}

#[no_mangle]
pub unsafe extern "C" fn rs_push_bits(
    count: u8,
    bits: u8,
    hse_bit_index: &mut u8,
    hse_curr_byte: &mut u8,
    out_buff: *mut u8,
    out_size: &mut usize,
) {
    // magic number huh...
    if count == 8 && *hse_bit_index == 0x80 {
        unsafe {
            *out_buff.add(*out_size) = bits;
        }
        *out_size += 1;
    } else {
        for i in (0..count).rev() {
            let bit = bits & (1u8 << i);
            if bit != 0 {
                *hse_curr_byte |= *hse_bit_index;
            }

            *hse_bit_index >>= 1u8;
            if *hse_bit_index == 0 {
                *hse_bit_index = 0x80;
                unsafe {
                    *out_buff.add(*out_size) = *hse_curr_byte;
                }
                *out_size += 1;
                *hse_curr_byte = 0;
            }
        }
    }
}
