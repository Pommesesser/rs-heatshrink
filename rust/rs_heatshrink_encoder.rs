const FLAG_IS_FINISHING: u8 = 1;

#[repr(u8)]
enum EncoderState {
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

    if *state == EncoderState::NotFull as u8 {
        *state = EncoderState::Filled as u8;
    }

    if *state == EncoderState::Done as u8 {
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

/// Push COUNT (max 8) bits to the output buffer, which has room.
/// Bytes are set from the lowest bits, up.
unsafe fn push_bits(
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

#[no_mangle]
pub unsafe extern "C" fn rs_add_tag_bit(
    tag: u8,
    hse_bit_index: &mut u8,
    hse_curr_byte: &mut u8,
    out_buff: *mut u8,
    out_size: &mut usize,
) {
    push_bits(1, tag, hse_bit_index, hse_curr_byte, out_buff, out_size);
}

#[no_mangle]
pub unsafe extern "C" fn rs_push_outgoing_bits(
    hse_outgoing_bits: u16,
    hse_outgoing_bits_count: &mut u8,
    hse_bit_index: &mut u8,
    hse_curr_byte: &mut u8,
    out_buff: *mut u8,
    out_size: &mut usize,
) -> u8 {
    let (count, bits) = if *hse_outgoing_bits_count > 8 {
        (
            8,
            (hse_outgoing_bits >> (*hse_outgoing_bits_count - 8)) as u8,
        )
    } else {
        (*hse_outgoing_bits_count, hse_outgoing_bits as u8)
    };

    if count > 0 {
        unsafe {
            push_bits(
                count,
                bits,
                hse_bit_index,
                hse_curr_byte,
                out_buff,
                out_size,
            );
        }
        *hse_outgoing_bits_count -= count;
    }

    count
}

#[no_mangle]
pub unsafe extern "C" fn rs_push_literal_byte(
    input_offset: u16,
    hse_match_scan_index: &mut u16,
    hse_buff: *mut u8,
    hse_bit_index: &mut u8,
    hse_curr_byte: &mut u8,
    out_buff: *mut u8,
    out_size: &mut usize,
) {
    let processed_offset = (*hse_match_scan_index).wrapping_sub(1);
    let buffer_offset = input_offset + processed_offset;

    let c = unsafe { *hse_buff.add(buffer_offset as usize) };

    push_bits(8, c, hse_bit_index, hse_curr_byte, out_buff, out_size);
}

#[no_mangle]
pub extern "C" fn rs_get_input_buffer_size(window_bits: u8) -> u16 {
    1u16 << window_bits
}

#[no_mangle]
pub extern "C" fn rs_get_lookahead_size(lookahead_bits: u8) -> u16 {
    1u16 << lookahead_bits
}

#[no_mangle]
pub unsafe extern "C" fn rs_save_backlog(
    input_buff_size: u16,
    hse_input_size: &mut u16,
    hse_match_scan_index: &mut u16,
    hse_buff: *mut u8,
) {
    let remaining = input_buff_size - *hse_match_scan_index;
    let shift_size = input_buff_size + remaining;
    let source_offset = input_buff_size - remaining;

    unsafe {
        std::ptr::copy(
            hse_buff.add(source_offset as usize),
            hse_buff,
            shift_size as usize,
        );
    }

    *hse_match_scan_index = 0;
    *hse_input_size -= input_buff_size - remaining;
}
