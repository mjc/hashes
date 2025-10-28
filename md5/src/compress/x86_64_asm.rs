//! x86-64 assembly backend

macro_rules! c {
    ($($l:expr)*) => {
        concat!($($l ,)*)
    };
}

macro_rules! round0 {
    ($a:literal, $b:literal, $c:literal, $d:literal, $k:literal, $s:literal, $i:literal) => {
        c!(
            "mov    r13d, " $c ";"
            "xor    r13d, " $d ";"
            "and    r13d, " $b ";"
            "xor    r13d, " $d ";"
            roundtail!($a, $b, $k, $s, $i)
        )
    }
}

macro_rules! round1 {
    ($a:literal, $b:literal, $c:literal, $d:literal, $k:literal, $s:literal, $i:literal) => {
        c!(
            "mov    r13d, " $d ";"
            "not    r13d;"
            "and    r13d, " $c ";"
            "mov    r14d, " $d ";"
            "and    r14d, " $b ";"
            "or     r13d, r14d;"
            roundtail!($a, $b, $k, $s, $i)
        )
    }
}

macro_rules! round2 {
    ($a:literal, $b:literal, $c:literal, $d:literal, $k:literal, $s:literal, $i:literal) => {
        c!(
            "mov    r13d, " $c ";"
            "xor    r13d, " $d ";"
            "xor    r13d, " $b ";"
            roundtail!($a, $b, $k, $s, $i)
        )
    }
}

macro_rules! round3 {
    ($a:literal, $b:literal, $c:literal, $d:literal, $k:literal, $s:literal, $i:literal) => {
        c!(
            "mov    r13d, " $d ";"
            "not    r13d;"
            "or     r13d, " $b ";"
            "xor    r13d, " $c ";"
            roundtail!($a, $b, $k, $s, $i)
        )
    }
}

macro_rules! roundtail {
    ($a:literal, $b:literal, $k:literal, $s:literal, $i:literal) => {
        c!(
            "mov       r14d, dword ptr [r8 + " $i " * 4];"
            "mov       r15d, dword ptr [rsi + " $k " * 4];"
            "add       " $a ", r14d;"
            "add       " $a ", r15d;"
            "add       " $a ", r13d;"
            "rol       " $a ", " $s ";"
            "add       " $a ", " $b ";"
        )
    }
}

pub(super) fn compress(state: &mut [u32; 4], blocks: &[[u8; 64]]) {
    if blocks.is_empty() {
        return;
    }

    unsafe {
        core::arch::asm!(
            "42:",

            "mov    eax, r9d",
            "mov    r10d, r11d",
            "mov    ecx, r12d",
            "mov    edx, {state3:e}",

            /* 64 rounds of hashing */
            round0!("eax", "r10d", "ecx", "edx",  0,  7,  0),
            round0!("edx", "eax", "r10d", "ecx",  1, 12,  1),
            round0!("ecx", "edx", "eax", "r10d",  2, 17,  2),
            round0!("r10d", "ecx", "edx", "eax",  3, 22,  3),
            round0!("eax", "r10d", "ecx", "edx",  4,  7,  4),
            round0!("edx", "eax", "r10d", "ecx",  5, 12,  5),
            round0!("ecx", "edx", "eax", "r10d",  6, 17,  6),
            round0!("r10d", "ecx", "edx", "eax",  7, 22,  7),
            round0!("eax", "r10d", "ecx", "edx",  8,  7,  8),
            round0!("edx", "eax", "r10d", "ecx",  9, 12,  9),
            round0!("ecx", "edx", "eax", "r10d", 10, 17, 10),
            round0!("r10d", "ecx", "edx", "eax", 11, 22, 11),
            round0!("eax", "r10d", "ecx", "edx", 12,  7, 12),
            round0!("edx", "eax", "r10d", "ecx", 13, 12, 13),
            round0!("ecx", "edx", "eax", "r10d", 14, 17, 14),
            round0!("r10d", "ecx", "edx", "eax", 15, 22, 15),

            round1!("eax", "r10d", "ecx", "edx",  1,  5, 16),
            round1!("edx", "eax", "r10d", "ecx",  6,  9, 17),
            round1!("ecx", "edx", "eax", "r10d", 11, 14, 18),
            round1!("r10d", "ecx", "edx", "eax",  0, 20, 19),
            round1!("eax", "r10d", "ecx", "edx",  5,  5, 20),
            round1!("edx", "eax", "r10d", "ecx", 10,  9, 21),
            round1!("ecx", "edx", "eax", "r10d", 15, 14, 22),
            round1!("r10d", "ecx", "edx", "eax",  4, 20, 23),
            round1!("eax", "r10d", "ecx", "edx",  9,  5, 24),
            round1!("edx", "eax", "r10d", "ecx", 14,  9, 25),
            round1!("ecx", "edx", "eax", "r10d",  3, 14, 26),
            round1!("r10d", "ecx", "edx", "eax",  8, 20, 27),
            round1!("eax", "r10d", "ecx", "edx", 13,  5, 28),
            round1!("edx", "eax", "r10d", "ecx",  2,  9, 29),
            round1!("ecx", "edx", "eax", "r10d",  7, 14, 30),
            round1!("r10d", "ecx", "edx", "eax", 12, 20, 31),

            round2!("eax", "r10d", "ecx", "edx",  5,  4, 32),
            round2!("edx", "eax", "r10d", "ecx",  8, 11, 33),
            round2!("ecx", "edx", "eax", "r10d", 11, 16, 34),
            round2!("r10d", "ecx", "edx", "eax", 14, 23, 35),
            round2!("eax", "r10d", "ecx", "edx",  1,  4, 36),
            round2!("edx", "eax", "r10d", "ecx",  4, 11, 37),
            round2!("ecx", "edx", "eax", "r10d",  7, 16, 38),
            round2!("r10d", "ecx", "edx", "eax", 10, 23, 39),
            round2!("eax", "r10d", "ecx", "edx", 13,  4, 40),
            round2!("edx", "eax", "r10d", "ecx",  0, 11, 41),
            round2!("ecx", "edx", "eax", "r10d",  3, 16, 42),
            round2!("r10d", "ecx", "edx", "eax",  6, 23, 43),
            round2!("eax", "r10d", "ecx", "edx",  9,  4, 44),
            round2!("edx", "eax", "r10d", "ecx", 12, 11, 45),
            round2!("ecx", "edx", "eax", "r10d", 15, 16, 46),
            round2!("r10d", "ecx", "edx", "eax",  2, 23, 47),

            round3!("eax", "r10d", "ecx", "edx",  0,  6, 48),
            round3!("edx", "eax", "r10d", "ecx",  7, 10, 49),
            round3!("ecx", "edx", "eax", "r10d", 14, 15, 50),
            round3!("r10d", "ecx", "edx", "eax",  5, 21, 51),
            round3!("eax", "r10d", "ecx", "edx", 12,  6, 52),
            round3!("edx", "eax", "r10d", "ecx",  3, 10, 53),
            round3!("ecx", "edx", "eax", "r10d", 10, 15, 54),
            round3!("r10d", "ecx", "edx", "eax",  1, 21, 55),
            round3!("eax", "r10d", "ecx", "edx",  8,  6, 56),
            round3!("edx", "eax", "r10d", "ecx", 15, 10, 57),
            round3!("ecx", "edx", "eax", "r10d",  6, 15, 58),
            round3!("r10d", "ecx", "edx", "eax", 13, 21, 59),
            round3!("eax", "r10d", "ecx", "edx",  4,  6, 60),
            round3!("edx", "eax", "r10d", "ecx", 11, 10, 61),
            round3!("ecx", "edx", "eax", "r10d",  2, 15, 62),
            round3!("r10d", "ecx", "edx", "eax",  9, 21, 63),

            "add   r9d, eax",
            "add   r11d, r10d",
            "add   r12d, ecx",
            "add   {state3:e}, edx",

            // Looping over blocks
            "add  rsi, 64",
            "dec  rdi",
            "jnz  42b",

            state3 = inout(reg) state[3],
            inout("rsi") blocks.as_ptr() => _,
            inout("rdi") blocks.len() => _,
            in("r8") crate::consts::RC.as_ptr(),
            inout("r9d") state[0],
            inout("r11d") state[1],
            inout("r12d") state[2],

            // Clobbers
            out("eax") _,
            out("ecx") _,
            out("edx") _,
            out("r10d") _,
            out("r13d") _,
            out("r14d") _,
            out("r15d") _,

            options(preserves_flags, readonly, pure, nostack),
        );
    }
}
