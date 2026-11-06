//! What the route reads of an Ogg Opus file, and nothing more
//! (cloudimplementation §7.6, D-361): that it begins as one mono Opus
//! stream, and how long it is. No decoding, no CRC: the provider reads the
//! audio, and the server stays off media crates (TS §7).

/// What the route needs of a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OggInfo {
    /// Samples at 48 kHz that a decoder drops at the start.
    pub pre_skip: u16,
    /// The audio's length, from the last page's granule position.
    pub duration_ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum OggError {
    #[error("not an Ogg stream")]
    NotOgg,
    #[error("not an Opus stream")]
    NotOpus,
    #[error("not one channel")]
    NotMono,
    #[error("no last page")]
    NoEnd,
    #[error("no audio")]
    Empty,
}

const CAPTURE: &[u8; 4] = b"OggS";
const HEADER_LEN: usize = 27;
const BOS: u8 = 0x02;
const EOS: u8 = 0x04;
const OPUS_HEAD: &[u8; 8] = b"OpusHead";
const OPUS_HEAD_LEN: usize = 19;
/// How far from the end the last page is looked for. A page holds at most
/// 255 × 255 bytes of data.
const LAST_PAGE_SEARCH: usize = 64 * 1024;
const SAMPLES_PER_MS: i64 = 48;

/// One page's header, read at `offset`.
struct Page {
    header_type: u8,
    granule: i64,
    /// Header, lacing table and data.
    len: usize,
    data_start: usize,
    /// The lacing values of the page.
    lacing: Vec<u8>,
}

fn page_at(bytes: &[u8], offset: usize) -> Option<Page> {
    let header = bytes.get(offset..offset.checked_add(HEADER_LEN)?)?;
    if header.get(..4)? != CAPTURE || *header.get(4)? != 0 {
        return None;
    }
    let header_type = *header.get(5)?;
    let granule = i64::from_le_bytes(header.get(6..14)?.try_into().ok()?);
    let segments = usize::from(*header.get(26)?);
    let table_start = offset + HEADER_LEN;
    let lacing = bytes.get(table_start..table_start + segments)?.to_vec();
    let data_len: usize = lacing.iter().map(|&v| usize::from(v)).sum();
    let data_start = table_start + segments;
    Some(Page {
        header_type,
        granule,
        len: HEADER_LEN + segments + data_len,
        data_start,
        lacing,
    })
}

/// Reads the first page's `OpusHead` and the last page's granule position.
pub fn read(bytes: &[u8]) -> Result<OggInfo, OggError> {
    let first = page_at(bytes, 0).ok_or(OggError::NotOgg)?;
    if first.header_type & BOS == 0 {
        return Err(OggError::NotOgg);
    }
    // The first packet is the identification header, and it is alone on the page.
    let first_packet_len: usize = first
        .lacing
        .iter()
        .take_while(|&&v| v == 255)
        .map(|_| 255)
        .sum::<usize>()
        + first
            .lacing
            .iter()
            .find(|&&v| v < 255)
            .map_or(0, |&v| usize::from(v));
    let head = bytes
        .get(first.data_start..first.data_start + first_packet_len)
        .ok_or(OggError::NotOgg)?;
    if head.len() < OPUS_HEAD_LEN || head.get(..8) != Some(OPUS_HEAD.as_slice()) {
        return Err(OggError::NotOpus);
    }
    // Version: the upper four bits are the major version, which must be 0.
    if head.get(8).is_none_or(|v| v & 0xF0 != 0) {
        return Err(OggError::NotOpus);
    }
    if head.get(9) != Some(&1) {
        return Err(OggError::NotMono);
    }
    let pre_skip = head
        .get(10..12)
        .and_then(|b| b.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or(OggError::NotOpus)?;

    let last = last_page(bytes).ok_or(OggError::NoEnd)?;
    let samples = last.granule - i64::from(pre_skip);
    let duration_ms = u32::try_from(samples / SAMPLES_PER_MS).map_err(|_| OggError::Empty)?;
    if duration_ms == 0 {
        return Err(OggError::Empty);
    }
    Ok(OggInfo {
        pre_skip,
        duration_ms,
    })
}

/// The page that ends exactly at the end of the file and closes the stream.
fn last_page(bytes: &[u8]) -> Option<Page> {
    let from = bytes.len().saturating_sub(LAST_PAGE_SEARCH);
    let window = bytes.get(from..)?;
    let mut end = window.len();
    while let Some(at) = window
        .get(..end)?
        .windows(CAPTURE.len())
        .rposition(|w| w == CAPTURE)
    {
        let offset = from + at;
        if let Some(page) = page_at(bytes, offset)
            && page.header_type & EOS != 0
            && offset + page.len == bytes.len()
        {
            return Some(page);
        }
        end = at;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A page with the given flags, granule and packets (each under 255 bytes).
    fn page(header_type: u8, granule: i64, sequence: u32, packets: &[&[u8]]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(CAPTURE);
        out.push(0);
        out.push(header_type);
        out.extend_from_slice(&granule.to_le_bytes());
        out.extend_from_slice(&7u32.to_le_bytes());
        out.extend_from_slice(&sequence.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.push(packets.len() as u8);
        for p in packets {
            out.push(p.len() as u8);
        }
        for p in packets {
            out.extend_from_slice(p);
        }
        out
    }

    fn opus_head(channels: u8, pre_skip: u16) -> Vec<u8> {
        let mut head = OPUS_HEAD.to_vec();
        head.push(1);
        head.push(channels);
        head.extend_from_slice(&pre_skip.to_le_bytes());
        head.extend_from_slice(&16_000u32.to_le_bytes());
        head.extend_from_slice(&0i16.to_le_bytes());
        head.push(0);
        head
    }

    fn file(channels: u8, pre_skip: u16, last_granule: i64) -> Vec<u8> {
        let mut out = page(BOS, 0, 0, &[&opus_head(channels, pre_skip)]);
        out.extend(page(0, 0, 1, &[b"OpusTags\x06\0\0\0offcut\0\0\0\0"]));
        let packet: &[u8] = &[0x78, 1, 2, 3];
        out.extend(page(0, 48_000, 2, &[packet, packet, packet]));
        // A packet holding the capture pattern must not be taken for a page.
        out.extend(page(EOS, last_granule, 3, &[b"xxOggSxx"]));
        out
    }

    #[test]
    fn a_mono_file_gives_its_duration_from_the_last_granule() {
        let info = read(&file(1, 104, 104 + 48 * 3_000)).unwrap();
        assert_eq!(
            info,
            OggInfo {
                pre_skip: 104,
                duration_ms: 3_000
            }
        );
    }

    #[test]
    fn each_bad_file_has_its_error() {
        assert_eq!(read(&file(2, 104, 48_104)), Err(OggError::NotMono));
        assert_eq!(read(&file(1, 104, 104)), Err(OggError::Empty));
        assert_eq!(read(&file(1, 104, 50)), Err(OggError::Empty));
        assert_eq!(read(b""), Err(OggError::NotOgg));
        assert_eq!(
            read(b"\x89PNG\r\n\x1a\n0000000000000000000000000"),
            Err(OggError::NotOgg)
        );

        let mut no_bos = file(1, 104, 48_104);
        no_bos[5] = 0;
        assert_eq!(read(&no_bos), Err(OggError::NotOgg));

        let mut not_opus = file(1, 104, 48_104);
        not_opus[28] = b'X';
        assert_eq!(read(&not_opus), Err(OggError::NotOpus));

        let whole = file(1, 104, 48_104);
        assert_eq!(read(&whole[..whole.len() - 3]), Err(OggError::NoEnd));

        let mut no_eos = whole.clone();
        let last = no_eos.len() - (HEADER_LEN + 1 + 8);
        no_eos[last + 5] = 0;
        assert_eq!(read(&no_eos), Err(OggError::NoEnd));
    }

    #[test]
    fn the_ffmpeg_tone_reads_three_seconds() {
        let bytes = include_bytes!("../../tests/fixtures/ogg/ffmpeg_tone_3s.ogg");
        let info = read(bytes).unwrap();
        assert!(info.duration_ms.abs_diff(3_000) <= 20, "{info:?}");
    }
}
