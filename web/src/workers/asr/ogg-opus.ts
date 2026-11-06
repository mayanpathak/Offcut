// Opus packets into an Ogg file (RFC 3533, RFC 7845), for cloud
// transcription (cloudimplementation §8.3). The browser's encoder gives
// packets; this writes the pages around them: the identification header
// (`OpusHead`), the comment header (`OpusTags`), then the audio, the last
// page's granule position trimming the stream to the exact length of the
// clip. Nothing here decodes or changes the audio.

/** The Ogg page CRC: polynomial 0x04C11DB7, initial value 0, not reflected. */
const CRC_TABLE: Uint32Array = (() => {
  const table = new Uint32Array(256);
  for (let i = 0; i < 256; i += 1) {
    let r = i << 24;
    for (let bit = 0; bit < 8; bit += 1) {
      r = r & 0x80000000 ? (r << 1) ^ 0x04c11db7 : r << 1;
    }
    table[i] = r >>> 0;
  }
  return table;
})();

export function oggCrc(bytes: Uint8Array): number {
  let crc = 0;
  for (const byte of bytes) {
    crc = ((crc << 8) ^ (CRC_TABLE[((crc >>> 24) ^ byte) & 0xff] ?? 0)) >>> 0;
  }
  return crc;
}

const BOS = 0x02;
const EOS = 0x04;
const HEADER_LEN = 27;
/** At most this many packets on one audio page: about one second of 20 ms packets. */
export const PACKETS_PER_PAGE = 50;
const MAX_LACING = 255;
/** What the browser's encoder is assumed to delay by when it does not say (TE-C1 read 104). */
export const DEFAULT_PRE_SKIP = 312;

export type OggOpusInput = {
  /** The encoder's packets, in order. */
  packets: readonly Uint8Array[];
  /** How many 48 kHz samples each packet decodes to. */
  packetSamples: readonly number[];
  /** 48 kHz samples the decoder drops at the start. */
  preSkip: number;
  /** The rate the audio was recorded at, for `OpusHead` (information only). */
  inputSampleRate: number;
  /** The clip's length in 48 kHz samples: the end is trimmed to it. */
  totalSamples: number;
  /** The stream's serial number. */
  serial: number;
};

/** The lacing values of a packet: 255s, then the rest (0 when the length is a multiple of 255). */
function lacing(length: number): number[] {
  const values: number[] = [];
  let left = length;
  while (left >= MAX_LACING) {
    values.push(MAX_LACING);
    left -= MAX_LACING;
  }
  values.push(left);
  return values;
}

function page(flags: number, granule: number, serial: number, sequence: number, packets: readonly Uint8Array[]): Uint8Array {
  const table = packets.flatMap((p) => lacing(p.length));
  if (table.length > MAX_LACING) {
    throw new Error("an Ogg page holds at most 255 lacing values");
  }
  const dataLength = packets.reduce((sum, p) => sum + p.length, 0);
  const out = new Uint8Array(HEADER_LEN + table.length + dataLength);
  const view = new DataView(out.buffer);
  out.set([0x4f, 0x67, 0x67, 0x53], 0); // "OggS"
  view.setUint8(4, 0);
  view.setUint8(5, flags);
  view.setBigInt64(6, BigInt(granule), true);
  view.setUint32(14, serial >>> 0, true);
  view.setUint32(18, sequence >>> 0, true);
  view.setUint32(22, 0, true);
  view.setUint8(26, table.length);
  out.set(table, HEADER_LEN);
  let at = HEADER_LEN + table.length;
  for (const p of packets) {
    out.set(p, at);
    at += p.length;
  }
  view.setUint32(22, oggCrc(out), true);
  return out;
}

function ascii(text: string): number[] {
  return Array.from(text, (c) => c.charCodeAt(0));
}

/** RFC 7845 §5.1: version 1, one channel, mapping family 0. */
export function opusHead(preSkip: number, inputSampleRate: number): Uint8Array {
  const out = new Uint8Array(19);
  const view = new DataView(out.buffer);
  out.set(ascii("OpusHead"), 0);
  view.setUint8(8, 1);
  view.setUint8(9, 1);
  view.setUint16(10, preSkip, true);
  view.setUint32(12, inputSampleRate, true);
  view.setInt16(16, 0, true);
  view.setUint8(18, 0);
  return out;
}

/** RFC 7845 §5.2: our vendor string and no comments. */
function opusTags(): Uint8Array {
  const vendor = ascii("offcut");
  const out = new Uint8Array(8 + 4 + vendor.length + 4);
  const view = new DataView(out.buffer);
  out.set(ascii("OpusTags"), 0);
  view.setUint32(8, vendor.length, true);
  out.set(vendor, 12);
  view.setUint32(12 + vendor.length, 0, true);
  return out;
}

/** The whole Ogg Opus file. */
export function writeOggOpus(input: OggOpusInput): Uint8Array {
  const { packets, packetSamples, preSkip, inputSampleRate, totalSamples, serial } = input;
  if (packets.length !== packetSamples.length) {
    throw new Error("every packet needs its sample count");
  }
  const pages: Uint8Array[] = [];
  let sequence = 0;
  pages.push(page(BOS, 0, serial, sequence++, [opusHead(preSkip, inputSampleRate)]));
  pages.push(page(0, 0, serial, sequence++, [opusTags()]));

  const end = preSkip + totalSamples;
  let decoded = preSkip;
  let first = 0;
  do {
    const group: Uint8Array[] = [];
    let laced = 0;
    let index = first;
    for (; index < packets.length && group.length < PACKETS_PER_PAGE; index += 1) {
      const packet = packets[index];
      if (packet === undefined) {
        break;
      }
      const needs = lacing(packet.length).length;
      if (laced + needs > MAX_LACING) {
        break;
      }
      group.push(packet);
      laced += needs;
      decoded += packetSamples[index] ?? 0;
    }
    const last = index >= packets.length;
    // The last page's granule is the clip's own end, which trims the padding of the last packet.
    pages.push(page(last ? EOS : 0, last ? end : Math.min(decoded, end), serial, sequence++, group));
    first = index;
  } while (first < packets.length);

  const length = pages.reduce((sum, p) => sum + p.length, 0);
  const out = new Uint8Array(length);
  let at = 0;
  for (const p of pages) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}
