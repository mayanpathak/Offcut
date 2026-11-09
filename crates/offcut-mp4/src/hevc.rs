//! HEVC's decoder configuration record (`hvcC`, ISO/IEC 14496-15 §8.3.3) and
//! the WebCodecs codec string built from it (Annex E), so the browser can be
//! asked whether it decodes the file (D-396, D-398). Only the fixed header is
//! read; the parameter-set arrays after it go to the decoder as they are.

use crate::ContainerError;

/// The bytes of `hvcC` before its arrays of parameter sets.
pub const HVCC_HEADER_LEN: usize = 23;

/// The fields of `hvcC` that make up the codec string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HvcC {
    pub profile_space: u8,
    pub high_tier: bool,
    pub profile_idc: u8,
    pub compatibility_flags: u32,
    pub constraint_flags: [u8; 6],
    pub level_idc: u8,
}

/// Reads the fixed header of an `hvcC` payload (the box body).
pub fn parse_hvcc(payload: &[u8]) -> Result<HvcC, ContainerError> {
    let malformed = ContainerError::Malformed("hvcC");
    let header = payload.get(..HVCC_HEADER_LEN).ok_or(malformed)?;
    let &[
        version,
        first,
        c0,
        c1,
        c2,
        c3,
        k0,
        k1,
        k2,
        k3,
        k4,
        k5,
        level,
        ..,
    ] = header
    else {
        return Err(malformed);
    };
    if version != 1 {
        return Err(malformed);
    }
    Ok(HvcC {
        profile_space: first >> 6,
        high_tier: (first >> 5) & 1 == 1,
        profile_idc: first & 0x1F,
        compatibility_flags: u32::from_be_bytes([c0, c1, c2, c3]),
        constraint_flags: [k0, k1, k2, k3, k4, k5],
        level_idc: level,
    })
}

/// `hvc1.<space><profile>.<compat>.<tier><level>[.<constraint>…]`: the
/// compatibility flags in reverse bit order, in hexadecimal; the constraint
/// bytes in hexadecimal, the zero bytes at the end left out. The prefix is
/// the sample entry's (`hev1` stays `hev1`); any other entry gets `hvc1`.
pub fn codec_string(h: &HvcC, sample_entry: [u8; 4]) -> String {
    let prefix = match &sample_entry {
        b"hev1" => "hev1",
        _ => "hvc1",
    };
    let space = match h.profile_space {
        1 => "A",
        2 => "B",
        3 => "C",
        _ => "",
    };
    let tier = if h.high_tier { 'H' } else { 'L' };
    let mut out = format!(
        "{prefix}.{space}{}.{:X}.{tier}{}",
        h.profile_idc,
        h.compatibility_flags.reverse_bits(),
        h.level_idc
    );
    let used = h
        .constraint_flags
        .iter()
        .rposition(|&byte| byte != 0)
        .map_or(0, |last| last + 1);
    for byte in h.constraint_flags.iter().take(used) {
        out.push_str(&format!(".{byte:X}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // The fixed headers of three real `hvcC` boxes, read in P171 (experiments.md).
    /// `rej_hevc.mov`: Main, level 3.1.
    const REJ_HEVC: [u8; 23] = [
        0x01, 0x01, 0x60, 0x00, 0x00, 0x00, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5d, 0xf0, 0x00,
        0xfc, 0xfd, 0xf8, 0xf8, 0x00, 0x00, 0x0f, 0x04,
    ];
    /// `long_45min_4k_hevc.mov`: Main, level 5.
    const LONG_4K: [u8; 23] = [
        0x01, 0x01, 0x60, 0x00, 0x00, 0x00, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x96, 0xf0, 0x00,
        0xfc, 0xfd, 0xf8, 0xf8, 0x00, 0x00, 0x0f, 0x04,
    ];
    /// `hevc_main10_2s.mov`: Main 10, level 4.
    const MAIN10: [u8; 23] = [
        0x01, 0x02, 0x20, 0x00, 0x00, 0x00, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x78, 0xf0, 0x00,
        0xfc, 0xfd, 0xfa, 0xfa, 0x00, 0x00, 0x0f, 0x04,
    ];

    fn string(payload: &[u8], entry: &[u8; 4]) -> String {
        codec_string(&parse_hvcc(payload).unwrap(), *entry)
    }

    #[test]
    fn real_boxes_give_the_strings_d1_chrome_was_asked_about() {
        assert_eq!(string(&REJ_HEVC, b"hvc1"), "hvc1.1.6.L93.90");
        assert_eq!(string(&LONG_4K, b"hvc1"), "hvc1.1.6.L150.90");
        assert_eq!(string(&MAIN10, b"hvc1"), "hvc1.2.4.L120.90");
        assert_eq!(string(&REJ_HEVC, b"hev1"), "hev1.1.6.L93.90");
    }

    #[test]
    fn the_fields_are_read_from_their_bits() {
        let h = parse_hvcc(&MAIN10).unwrap();
        assert_eq!(
            h,
            HvcC {
                profile_space: 0,
                high_tier: false,
                profile_idc: 2,
                compatibility_flags: 0x2000_0000,
                constraint_flags: [0x90, 0, 0, 0, 0, 0],
                level_idc: 120,
            }
        );
    }

    #[test]
    fn space_tier_and_inner_zero_bytes_are_written_as_annex_e_says() {
        let h = HvcC {
            profile_space: 1,
            high_tier: true,
            profile_idc: 1,
            compatibility_flags: 0x6000_0000,
            constraint_flags: [0xB0, 0x00, 0x10, 0x00, 0x00, 0x00],
            level_idc: 153,
        };
        assert_eq!(codec_string(&h, *b"hvc1"), "hvc1.A1.6.H153.B0.0.10");
        let none = HvcC {
            constraint_flags: [0; 6],
            ..h
        };
        assert_eq!(codec_string(&none, *b"hvc1"), "hvc1.A1.6.H153");
    }

    #[test]
    fn short_or_wrong_version_payloads_are_malformed() {
        assert_eq!(
            parse_hvcc(&REJ_HEVC[..22]),
            Err(ContainerError::Malformed("hvcC"))
        );
        assert_eq!(parse_hvcc(&[]), Err(ContainerError::Malformed("hvcC")));
        let mut v0 = REJ_HEVC;
        v0[0] = 0;
        assert_eq!(parse_hvcc(&v0), Err(ContainerError::Malformed("hvcC")));
    }
}
