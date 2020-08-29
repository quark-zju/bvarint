//! # A Better Varint
//!
//! Based on D. Richard Hipp.'s "A Better Varint" idea.
//! See https://youtu.be/gpxnbly9bz4?t=2386.
//!
//! Changed so leading 0xff is reserved for larger integers
//! and some branches are removed.

use std::io;

/// Encode `v` and write it to `w`.
pub fn write_bvarint(v: u64, mut w: impl io::Write) -> io::Result<()> {
    match v {
        0..=0xf6 => {
            w.write_all(&[v as u8])?;
        }
        0xf7..=u64::MAX => {
            let width = ((64 + 8 - 1 - v.leading_zeros()) / 8) as usize;
            debug_assert!(width >= 1);
            let a: [u8; 8] = v.to_be_bytes();
            w.write_all(&[(0xf7 - 1 + width) as u8])?;
            w.write_all(&a[(8 - width)..])?;
        }
    }
    Ok(())
}

/// Read from `r` and return the decoded integer.
pub fn read_bvarint(mut r: impl io::Read) -> io::Result<u64> {
    let mut a = [0; 8];
    r.read_exact(&mut a[7..8])?;
    match a[7] {
        0..=0xf6 => Ok(a[7] as _),
        0xf7..=0xfe => {
            let width = (a[7] - 0xf7 + 1) as usize;
            r.read_exact(&mut a[(8 - width)..8])?;
            Ok(u64::from_be_bytes(a))
        }
        // 0xff is reserved for larger integers (ex. u128).
        0xff => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "exceeds u64::MAX",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    fn check_round_trip_u64(x: u64) {
        let mut buf = Vec::new();
        write_bvarint(x, &mut buf).unwrap();
        let y = read_bvarint(&buf[..]).unwrap();
        assert_eq!(x, y, "check_round_trip(0x{:x})", x);
    }

    fn check_order_u64(x: u64, y: u64) {
        let mut bufx = Vec::new();
        write_bvarint(x, &mut bufx).unwrap();

        let mut bufy = Vec::new();
        write_bvarint(y, &mut bufy).unwrap();
        assert_eq!(
            x.cmp(&y),
            bufx.cmp(&bufy),
            "check_order_u64(0x{:x}, 0x{:x}) {:?} {:?}",
            x,
            y,
            bufx,
            bufy,
        );
    }

    fn interesting_values() -> Vec<u64> {
        vec![0, 0xef, 0x7ee, 0x8ee, 0x107ee, 0x108ee, u64::MAX - 3]
            .into_iter()
            .chain((5..=63).map(|b| (1u64 << b) - 2))
            .flat_map(|v| vec![v, v + 1, v + 2, v + 3])
            .collect()
    }

    #[test]
    fn test_round_trip_u64_manual() {
        #[cfg(not(debug_assertions))]
        for x in 0..0x1000003 {
            check_round_trip_u64(x);
        }
        for x in interesting_values() {
            check_round_trip_u64(x);
        }
    }

    #[test]
    fn test_order_manual() {
        #[cfg(not(debug_assertions))]
        for x in 0..0x1000003 {
            check_order_u64(x, x + 1);
        }
        let values = interesting_values();
        for x in &values {
            for y in &values {
                check_order_u64(*x, *y);
            }
        }
    }

    #[test]
    fn test_round_trip_u64_quickcheck() {
        quickcheck(check_round_trip_u64 as fn(u64));
    }

    #[test]
    fn test_order_u64_quickcheck() {
        quickcheck(check_order_u64 as fn(u64, u64));
    }
}
