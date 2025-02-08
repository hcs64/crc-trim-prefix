use std::env;
use std::fs::File;
use std::io::Read;

use crc32fast::Hasher;

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

fn suffix_crc(prefix: &[u8], target_size: usize, target_crc: u32) -> u32 {
    assert!(prefix.len() < target_size);
    let mut hasher = Hasher::new();
    hasher.update(prefix);
    hasher.combine(&Hasher::new_with_initial_len(
        target_crc,
        (target_size - prefix.len()).try_into().unwrap(),
    ));
    hasher.finalize()
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
