//! The colour box (`colr`, ISO/IEC 14496-12 §12.1.5; QuickTime's `nclc`), and
//! whether it says the video is HDR (D-397).
//!
//! As read (P172): D-397 names `nclx` only, but ffmpeg writes QuickTime's
//! `nclc` in a `.mov`, with the same code points. A `.mov` saying PQ or HLG
//! in `nclc` is HDR too: reading it as SDR would accept an HDR file, and V6.1
//! refuses HDR (`hdr_ok = false`).

use crate::ContainerError;

/// SMPTE ST 2084 (PQ), ITU-T H.273 transfer characteristics.
pub const TRANSFER_PQ: u16 = 16;
/// ARIB STD-B67 (HLG).
pub const TRANSFER_HLG: u16 = 18;

/// What a `colr` box holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colr {
    /// `nclx` (ISO) or `nclc` (QuickTime): the H.273 code points.
    Coded {
        primaries: u16,
        transfer: u16,
        matrix: u16,
        /// `nclx` only; `nclc` has no range flag.
        full_range: bool,
    },
    /// `rICC`, `prof` or another kind: an ICC profile, not read.
    Other,
}

/// Reads a `colr` body.
pub fn parse_colr(body: &[u8]) -> Result<Colr, ContainerError> {
    let malformed = ContainerError::Malformed("colr");
    let kind = body.get(..4).ok_or(malformed)?;
    let coded = |len: usize| -> Result<[u8; 7], ContainerError> {
        let fields = body.get(4..4 + len).ok_or(malformed)?;
        let mut out = [0u8; 7];
        for (slot, byte) in out.iter_mut().zip(fields) {
            *slot = *byte;
        }
        Ok(out)
    };
    match kind {
        b"nclx" | b"nclc" => {
            let is_x = kind == b"nclx";
            let [p0, p1, t0, t1, m0, m1, range] = coded(if is_x { 7 } else { 6 })?;
            Ok(Colr::Coded {
                primaries: u16::from_be_bytes([p0, p1]),
                transfer: u16::from_be_bytes([t0, t1]),
                matrix: u16::from_be_bytes([m0, m1]),
                full_range: is_x && range & 0x80 != 0,
            })
        }
        _ => Ok(Colr::Other),
    }
}

/// HDR: a coded `colr` whose transfer is PQ or HLG. No `colr`, or an ICC one, is SDR.
pub fn is_hdr(colr: Option<&Colr>) -> bool {
    matches!(
        colr,
        Some(Colr::Coded { transfer, .. }) if *transfer == TRANSFER_PQ || *transfer == TRANSFER_HLG
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nclx(primaries: u16, transfer: u16, matrix: u16, full: bool) -> Vec<u8> {
        let mut b = b"nclx".to_vec();
        for v in [primaries, transfer, matrix] {
            b.extend_from_slice(&v.to_be_bytes());
        }
        b.push(if full { 0x80 } else { 0 });
        b
    }

    #[test]
    fn hlg_and_pq_are_hdr_bt709_is_not() {
        let hlg = parse_colr(&nclx(9, 18, 9, false)).unwrap();
        let pq = parse_colr(&nclx(9, 16, 9, false)).unwrap();
        let sdr = parse_colr(&nclx(1, 1, 1, false)).unwrap();
        assert!(is_hdr(Some(&hlg)));
        assert!(is_hdr(Some(&pq)));
        assert!(!is_hdr(Some(&sdr)));
        assert!(!is_hdr(None));
    }

    #[test]
    fn nclx_fields_and_range_are_read() {
        assert_eq!(
            parse_colr(&nclx(9, 18, 9, true)).unwrap(),
            Colr::Coded {
                primaries: 9,
                transfer: 18,
                matrix: 9,
                full_range: true
            }
        );
    }

    #[test]
    fn nclc_carries_the_same_code_points() {
        let mut b = b"nclc".to_vec();
        for v in [9u16, 18, 9] {
            b.extend_from_slice(&v.to_be_bytes());
        }
        let c = parse_colr(&b).unwrap();
        assert_eq!(
            c,
            Colr::Coded {
                primaries: 9,
                transfer: 18,
                matrix: 9,
                full_range: false
            }
        );
        assert!(is_hdr(Some(&c)));
    }

    #[test]
    fn an_icc_profile_is_other_and_sdr() {
        let c = parse_colr(b"prof\x00\x00\x00\x10").unwrap();
        assert_eq!(c, Colr::Other);
        assert!(!is_hdr(Some(&c)));
    }

    #[test]
    fn a_short_box_is_malformed() {
        assert_eq!(parse_colr(b"ncl"), Err(ContainerError::Malformed("colr")));
        assert_eq!(
            parse_colr(b"nclx\x00\x09\x00\x12\x00"),
            Err(ContainerError::Malformed("colr"))
        );
        assert_eq!(
            parse_colr(b"nclc\x00\x09\x00"),
            Err(ContainerError::Malformed("colr"))
        );
    }
}
