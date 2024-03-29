use std::env;
use std::fs::File;
use std::io::Read;

use crc32fast::Hasher;

mod crc_trim_trailing;

fn main() {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 4 {
        eprintln!(r"Usage: crc-trim-prefix prefix_file target_size target_crc");
        return;
    }
    let mut in_file = File::open(&args[1]).expect("couldn't open prefix file");

    let target_size = args[2]
        .to_str()
        .and_then(|s| s.parse().ok())
        .expect("bad target size string");
    assert!(target_size as u32 as usize == target_size);

    let target_crc = u32::from_str_radix(args[3].to_str().expect("bad checksum string"), 16)
        .expect("bad checksum string");

    let mut input = Vec::new();
    in_file.read_to_end(&mut input).expect("couldn't read file");

    let suffix_crc = suffix_crc(&input, target_size, target_crc);
    let suffix_len = target_size - input.len();
    println!("suffix len {suffix_len} crc {suffix_crc:08x}");
}

const INIT_CRC: u32 = !0;
const IEEE_TABLE: [u32; 256] = [
    0, 1996959894, 3993919788, 2567524794, 124634137, 1886057615, 3915621685, 2657392035,
    249268274, 2044508324, 3772115230, 2547177864, 162941995, 2125561021, 3887607047, 2428444049,
    498536548, 1789927666, 4089016648, 2227061214, 450548861, 1843258603, 4107580753, 2211677639,
    325883990, 1684777152, 4251122042, 2321926636, 335633487, 1661365465, 4195302755, 2366115317,
    997073096, 1281953886, 3579855332, 2724688242, 1006888145, 1258607687, 3524101629, 2768942443,
    901097722, 1119000684, 3686517206, 2898065728, 853044451, 1172266101, 3705015759, 2882616665,
    651767980, 1373503546, 3369554304, 3218104598, 565507253, 1454621731, 3485111705, 3099436303,
    671266974, 1594198024, 3322730930, 2970347812, 795835527, 1483230225, 3244367275, 3060149565,
    1994146192, 31158534, 2563907772, 4023717930, 1907459465, 112637215, 2680153253, 3904427059,
    2013776290, 251722036, 2517215374, 3775830040, 2137656763, 141376813, 2439277719, 3865271297,
    1802195444, 476864866, 2238001368, 4066508878, 1812370925, 453092731, 2181625025, 4111451223,
    1706088902, 314042704, 2344532202, 4240017532, 1658658271, 366619977, 2362670323, 4224994405,
    1303535960, 984961486, 2747007092, 3569037538, 1256170817, 1037604311, 2765210733, 3554079995,
    1131014506, 879679996, 2909243462, 3663771856, 1141124467, 855842277, 2852801631, 3708648649,
    1342533948, 654459306, 3188396048, 3373015174, 1466479909, 544179635, 3110523913, 3462522015,
    1591671054, 702138776, 2966460450, 3352799412, 1504918807, 783551873, 3082640443, 3233442989,
    3988292384, 2596254646, 62317068, 1957810842, 3939845945, 2647816111, 81470997, 1943803523,
    3814918930, 2489596804, 225274430, 2053790376, 3826175755, 2466906013, 167816743, 2097651377,
    4027552580, 2265490386, 503444072, 1762050814, 4150417245, 2154129355, 426522225, 1852507879,
    4275313526, 2312317920, 282753626, 1742555852, 4189708143, 2394877945, 397917763, 1622183637,
    3604390888, 2714866558, 953729732, 1340076626, 3518719985, 2797360999, 1068828381, 1219638859,
    3624741850, 2936675148, 906185462, 1090812512, 3747672003, 2825379669, 829329135, 1181335161,
    3412177804, 3160834842, 628085408, 1382605366, 3423369109, 3138078467, 570562233, 1426400815,
    3317316542, 2998733608, 733239954, 1555261956, 3268935591, 3050360625, 752459403, 1541320221,
    2607071920, 3965973030, 1969922972, 40735498, 2617837225, 3943577151, 1913087877, 83908371,
    2512341634, 3803740692, 2075208622, 213261112, 2463272603, 3855990285, 2094854071, 198958881,
    2262029012, 4057260610, 1759359992, 534414190, 2176718541, 4139329115, 1873836001, 414664567,
    2282248934, 4279200368, 1711684554, 285281116, 2405801727, 4167216745, 1634467795, 376229701,
    2685067896, 3608007406, 1308918612, 956543938, 2808555105, 3495958263, 1231636301, 1047427035,
    2932959818, 3654703836, 1088359270, 936918000, 2847714899, 3736837829, 1202900863, 817233897,
    3183342108, 3401237130, 1404277552, 615818150, 3134207493, 3453421203, 1423857449, 601450431,
    3009837614, 3294710456, 1567103746, 711928724, 3020668471, 3272380065, 1510334235, 755167117,
];

fn update_0(crc: u32) -> u32 {
    IEEE_TABLE[usize::from(crc as u8)] ^ (crc >> 8)
}

fn zeroes(block_size: u32) -> Hasher {
    // Get the CRC of a block of zeroes by adding powers of 2
    let mut pow2_zero_block_crc = crc32fast::hash(&[0]);
    let mut acc = Hasher::new();
    for n in 0..=31 {
        let pow2 = 1u32 << n;
        let mut h = Hasher::new_with_initial_len(pow2_zero_block_crc, pow2 as u64);
        if (block_size & pow2) != 0 {
            acc.combine(&h);
        } else if block_size < pow2 {
            break;
        }
        h.combine(&h.clone());
        pow2_zero_block_crc = h.finalize();
    }
    acc
}

fn block_advance_xor(block_size: u32) -> [u32; 256] {
    let zero_block = zeroes(block_size);
    let zero_crc = zero_block.clone().finalize();
    (0..=255u8)
        .map(|b| {
            let mut acc = Hasher::new();
            acc.update(&[b]);
            acc.combine(&zero_block);
            acc.finalize() ^ zero_crc
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn suffix_crc(prefix: &[u8], target_size: usize, target_crc: u32) -> u32 {
    assert!(prefix.len() < target_size);
    let trim_len = prefix.len().try_into().unwrap();

    let advance = block_advance_xor(target_size as u32);

    let mut crc = target_crc ^ INIT_CRC;
    for b in prefix {
        // Advance target crc to remove prefix byte from the
        // beginning, add 0 padding to the end
        crc = update_0(crc) ^ advance[usize::from(*b)];
    }
    crc ^= INIT_CRC;

    let pad_crc = zeroes(trim_len).finalize();

    crc_trim_trailing::trim(crc, pad_crc, trim_len)
}

#[cfg(test)]
mod test {
    use super::suffix_crc;
    use rand_xoshiro::rand_core::RngCore;
    use rand_xoshiro::rand_core::SeedableRng;
    use rand_xoshiro::Xoshiro256StarStar;

    fn test_target(target: &[u8]) -> Result<(), String> {
        let target_crc = crc32fast::hash(target);

        for i in 1..target.len() {
            let (prefix, suffix) = target.split_at(i);
            let expected = crc32fast::hash(suffix);
            let actual = suffix_crc(prefix, target.len(), target_crc);
            if expected != actual {
                return Err(format!("{i}: {expected:#08x} != {actual:#08x}"));
            }
        }

        Ok(())
    }

    #[test]
    fn small_test() {
        test_target(b"Hello, world!").unwrap();
    }

    #[test]
    fn small_zero_test() {
        test_target(&[0; 29]).unwrap();
        test_target(&[0; 30]).unwrap();
        test_target(&[0; 31]).unwrap();
        test_target(&[0; 128]).unwrap();
    }

    #[test]
    fn small_rand_test() {
        let mut target = vec![0; 128];
        Xoshiro256StarStar::seed_from_u64(1).fill_bytes(target.as_mut_slice());
        test_target(&target).unwrap();
    }

    #[test]
    #[ignore]
    fn large_rand_test() {
        let mut target = vec![0; 1024 * 1024 * 300];
        Xoshiro256StarStar::seed_from_u64(2).fill_bytes(target.as_mut_slice());

        let target = target.as_slice();
        let target_crc = crc32fast::hash(target);

        for i in target.len() / 2 - 8..target.len() / 2 + 8 {
            let (prefix, suffix) = target.split_at(i);
            let expected = crc32fast::hash(suffix);
            let actual = suffix_crc(prefix, target.len(), target_crc);
            assert_eq!(expected, actual);
        }
    }
}
