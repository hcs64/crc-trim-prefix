// Ported from https://github.com/werekraken/crc-trim
// under the Artistic License 2.0, which was based in
// turn on crc32.c from zlib.

const GF2_DIM: usize = 32;

fn gf2_matrix_times(mat: &[u32; GF2_DIM], mut vec: u32) -> u32 {
    let mut sum = 0;
    for m in mat {
        if vec == 0 {
            break;
        }

        if vec >> 31 != 0 {
            sum ^= *m;
        }
        vec <<= 1;
    }
    sum
}

fn gf2_matrix_square(square: &mut [u32; GF2_DIM], mat: &[u32; GF2_DIM]) {
    for n in 0..GF2_DIM {
        square[n] = gf2_matrix_times(mat, mat[n]);
    }
}

pub fn trim(total_crc: u32, suffix_crc: u32, mut len2: u32) -> u32 {
    if len2 == 0 {
        return total_crc;
    }

    let mut even = [0u32; GF2_DIM];

    /* put operator for one zero bit in odd */
    let mut odd = std::array::from_fn(|n| {
        if n == 0 {
            0xdb710641 /* CRC-32 "Un"polynomial */
        } else {
            1 << (32 - n)
        }
    });

    /* get crcA0 */
    let mut crc1 = total_crc ^ suffix_crc;

    /* put operator for two zero bits in even */
    gf2_matrix_square(&mut even, &odd);

    /* put operator for four zero bits in odd */
    gf2_matrix_square(&mut odd, &even);

    /* apply len2 zeros to crc1 (first square will put the operator for one
    zero byte, eight zero bits, in even) */
    while len2 != 0 {
        /* apply zeros operator for this bit of len2 */
        gf2_matrix_square(&mut even, &odd);
        if len2 & 1 != 0 {
            crc1 = gf2_matrix_times(&even, crc1);
        }
        len2 >>= 1;

        /* if no more bits set, then done */
        if len2 == 0 {
            break;
        }

        /* another iteration of the loop with odd and even swapped */
        gf2_matrix_square(&mut odd, &even);
        if len2 & 1 != 0 {
            crc1 = gf2_matrix_times(&odd, crc1);
        }
        len2 >>= 1;

        /* if no more bits set, then done */
    }

    crc1
}
