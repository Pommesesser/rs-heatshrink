const FLAG_IS_FINISHING: u8 = 1;

#[repr(u8)]
enum HseState {
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
pub extern "C" fn rs_is_finishing(flags: u8) -> i32 {
    (flags & FLAG_IS_FINISHING) as i32
}

#[no_mangle]
pub extern "C" fn rs_heatshrink_encoder_finish(flags: &mut u8, state: &mut u8) -> i32 {
    *flags |= FLAG_IS_FINISHING;

    if *state == HseState::NotFull as u8 {
        *state = HseState::Filled as u8;
    }

    if *state == HseState::Done as u8 {
        0
    } else {
        1
    }
}
