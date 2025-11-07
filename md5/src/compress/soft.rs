#![allow(clippy::many_single_char_names, clippy::unreadable_literal)]
use crate::consts::RC;

#[inline(always)]
fn op_f(w: u32, x: u32, y: u32, z: u32, m: u32, c: u32, s: u32) -> u32 {
    ((x & y) | (!x & z))
        .wrapping_add(w)
        .wrapping_add(m)
        .wrapping_add(c)
        .rotate_left(s)
        .wrapping_add(x)
}
#[inline(always)]
fn op_g(w: u32, x: u32, y: u32, z: u32, m: u32, c: u32, s: u32) -> u32 {
    // Optimized G function: combines master's addition optimization with delayed x dependency
    // We replace the logical OR in `(x & z) | (y & !z)` with addition.
    // Since masked bits do not overlap, the expressions are equivalent;
    // however, addition results in better performance on high-end CPUs.
    // Additionally, we delay the x dependency to reduce pipeline stalls.
    let mut result = w;
    result = result.wrapping_add(m);        // No dependencies
    result = result.wrapping_add(c);        // No dependencies  
    result = result.wrapping_add(y & !z);   // c & ~d - no x dependency
    result = result.wrapping_add(x & z);    // b & d - finally use x (addition, not OR)
    
    result.rotate_left(s).wrapping_add(x)
}

// Optimized H function that reuses XOR computation from previous round
#[inline(always)]
fn op_h_reuse(w: u32, x: u32, y: u32, _z: u32, m: u32, c: u32, s: u32, h_tmp: &mut u32) -> u32 {
    // h_tmp contains one part, we XOR with the other two to get x ^ y ^ z
    *h_tmp ^= y;
    *h_tmp ^= x;  // Now h_tmp = x ^ y ^ z
    let result = h_tmp.wrapping_add(w).wrapping_add(m).wrapping_add(c).rotate_left(s).wrapping_add(x);
    *h_tmp = y;  // Store y for next round (next round will XOR with z and x)
    result
}

#[inline(always)]
fn op_i(w: u32, x: u32, y: u32, z: u32, m: u32, c: u32, s: u32) -> u32 {
    (y ^ (x | !z))
        .wrapping_add(w)
        .wrapping_add(m)
        .wrapping_add(c)
        .rotate_left(s)
        .wrapping_add(x)
}

#[inline]
fn compress_block(state: &mut [u32; 4], input: &[u8; 64]) {
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];

    let mut data = [0u32; 16];
    for (o, chunk) in data.iter_mut().zip(input.chunks_exact(4)) {
        *o = u32::from_le_bytes(chunk.try_into().unwrap());
    }

    // Input caching optimization: cache most frequently used inputs
    let w0 = data[0];   // Used 3 times
    let w1 = data[1];   // Used 3 times
    let w4 = data[4];   // Used 3 times
    let w5 = data[5];   // Used 3 times
    let w8 = data[8];   // Used 3 times
    let w12 = data[12]; // Used 3 times
    // Additional caching for even distribution
    let w2 = data[2];   // Used 3 times
    let w6 = data[6];   // Used 3 times
    let w9 = data[9];   // Used 3 times
    let w14 = data[14]; // Used 3 times

    // round 1
    a = op_f(a, b, c, d, w0, RC[0], 7);
    d = op_f(d, a, b, c, w1, RC[1], 12);
    c = op_f(c, d, a, b, w2, RC[2], 17);
    b = op_f(b, c, d, a, data[3], RC[3], 22);

    a = op_f(a, b, c, d, w4, RC[4], 7);
    d = op_f(d, a, b, c, w5, RC[5], 12);
    c = op_f(c, d, a, b, w6, RC[6], 17);
    b = op_f(b, c, d, a, data[7], RC[7], 22);

    a = op_f(a, b, c, d, w8, RC[8], 7);
    d = op_f(d, a, b, c, w9, RC[9], 12);
    c = op_f(c, d, a, b, data[10], RC[10], 17);
    b = op_f(b, c, d, a, data[11], RC[11], 22);

    a = op_f(a, b, c, d, w12, RC[12], 7);
    d = op_f(d, a, b, c, data[13], RC[13], 12);
    c = op_f(c, d, a, b, w14, RC[14], 17);
    b = op_f(b, c, d, a, data[15], RC[15], 22);

    // round 2 (using optimized G function)
    a = op_g(a, b, c, d, w1, RC[16], 5);
    d = op_g(d, a, b, c, w6, RC[17], 9);
    c = op_g(c, d, a, b, data[11], RC[18], 14);
    b = op_g(b, c, d, a, w0, RC[19], 20);

    a = op_g(a, b, c, d, w5, RC[20], 5);
    d = op_g(d, a, b, c, data[10], RC[21], 9);
    c = op_g(c, d, a, b, data[15], RC[22], 14);
    b = op_g(b, c, d, a, w4, RC[23], 20);

    a = op_g(a, b, c, d, w9, RC[24], 5);
    d = op_g(d, a, b, c, w14, RC[25], 9);
    c = op_g(c, d, a, b, data[3], RC[26], 14);
    b = op_g(b, c, d, a, w8, RC[27], 20);

    a = op_g(a, b, c, d, data[13], RC[28], 5);
    d = op_g(d, a, b, c, w2, RC[29], 9);
    c = op_g(c, d, a, b, data[7], RC[30], 14);
    b = op_g(b, c, d, a, w12, RC[31], 20);

    // round 3 (H function with XOR reuse optimization)
    let mut h_tmp = d;  // Start with z for the first round (d), will XOR with c and b
    
    a = op_h_reuse(a, b, c, d, w5, RC[32], 4, &mut h_tmp);
    d = op_h_reuse(d, a, b, c, w8, RC[33], 11, &mut h_tmp);
    c = op_h_reuse(c, d, a, b, data[11], RC[34], 16, &mut h_tmp);
    b = op_h_reuse(b, c, d, a, w14, RC[35], 23, &mut h_tmp);

    a = op_h_reuse(a, b, c, d, w1, RC[36], 4, &mut h_tmp);
    d = op_h_reuse(d, a, b, c, w4, RC[37], 11, &mut h_tmp);
    c = op_h_reuse(c, d, a, b, data[7], RC[38], 16, &mut h_tmp);
    b = op_h_reuse(b, c, d, a, data[10], RC[39], 23, &mut h_tmp);

    a = op_h_reuse(a, b, c, d, data[13], RC[40], 4, &mut h_tmp);
    d = op_h_reuse(d, a, b, c, w0, RC[41], 11, &mut h_tmp);
    c = op_h_reuse(c, d, a, b, data[3], RC[42], 16, &mut h_tmp);
    b = op_h_reuse(b, c, d, a, w6, RC[43], 23, &mut h_tmp);

    a = op_h_reuse(a, b, c, d, w9, RC[44], 4, &mut h_tmp);
    d = op_h_reuse(d, a, b, c, w12, RC[45], 11, &mut h_tmp);
    c = op_h_reuse(c, d, a, b, data[15], RC[46], 16, &mut h_tmp);
    b = op_h_reuse(b, c, d, a, w2, RC[47], 23, &mut h_tmp);

    // round 4
    a = op_i(a, b, c, d, w0, RC[48], 6);
    d = op_i(d, a, b, c, data[7], RC[49], 10);
    c = op_i(c, d, a, b, w14, RC[50], 15);
    b = op_i(b, c, d, a, w5, RC[51], 21);

    a = op_i(a, b, c, d, w12, RC[52], 6);
    d = op_i(d, a, b, c, data[3], RC[53], 10);
    c = op_i(c, d, a, b, data[10], RC[54], 15);
    b = op_i(b, c, d, a, w1, RC[55], 21);

    a = op_i(a, b, c, d, w8, RC[56], 6);
    d = op_i(d, a, b, c, data[15], RC[57], 10);
    c = op_i(c, d, a, b, w6, RC[58], 15);
    b = op_i(b, c, d, a, data[13], RC[59], 21);

    a = op_i(a, b, c, d, w4, RC[60], 6);
    d = op_i(d, a, b, c, data[11], RC[61], 10);
    c = op_i(c, d, a, b, w2, RC[62], 15);
    b = op_i(b, c, d, a, w9, RC[63], 21);

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
}

#[inline]
pub(super) fn compress(state: &mut [u32; 4], blocks: &[[u8; 64]]) {
    for block in blocks {
        compress_block(state, block)
    }
}
