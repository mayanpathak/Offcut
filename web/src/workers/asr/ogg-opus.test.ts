import { describe, expect, it } from "vitest";

import { DEFAULT_PRE_SKIP, oggCrc, opusHead, PACKETS_PER_PAGE, writeOggOpus } from "./ogg-opus";

/** The CRC bit by bit, as RFC 3533 describes it: the table version must agree. */
function slowCrc(bytes: Uint8Array): number {
  let crc = 0;
  for (const byte of bytes) {
    crc ^= byte << 24;
    for (let bit = 0; bit < 8; bit += 1) {
      crc = crc & 0x80000000 ? (crc << 1) ^ 0x04c11db7 : crc << 1;
    }
    crc >>>= 0;
  }
  return crc;
}

type ReadPage = { flags: number; granule: bigint; sequence: number; serial: number; crcOk: boolean; table: number[]; packets: Uint8Array[] };

/** A reader of what the writer wrote, written apart from it. */
function readPages(file: Uint8Array): ReadPage[] {
  const pages: ReadPage[] = [];
  const view = new DataView(file.buffer, file.byteOffset, file.byteLength);
  let at = 0;
  let pending: number[] = [];
  while (at < file.length) {
    expect(String.fromCharCode(...file.subarray(at, at + 4))).toBe("OggS");
    const segments = view.getUint8(at + 26);
    const table = Array.from(file.subarray(at + 27, at + 27 + segments));
    const dataLength = table.reduce((a, b) => a + b, 0);
    const length = 27 + segments + dataLength;
    const copy = file.slice(at, at + length);
    new DataView(copy.buffer).setUint32(22, 0, true);
    const packets: Uint8Array[] = [];
    let offset = at + 27 + segments;
    for (const value of table) {
      for (let i = 0; i < value; i += 1) {
        pending.push(file[offset + i] ?? -1);
      }
      offset += value;
      if (value < 255) {
        packets.push(Uint8Array.from(pending));
        pending = [];
      }
    }
    pages.push({
      flags: view.getUint8(at + 5),
      granule: view.getBigInt64(at + 6, true),
      serial: view.getUint32(at + 14, true),
      sequence: view.getUint32(at + 18, true),
      crcOk: view.getUint32(at + 22, true) === slowCrc(copy),
      table,
      packets,
    });
    at += length;
  }
  expect(pending).toEqual([]);
  return pages;
}

function packetsOf(count: number, size = 80): Uint8Array[] {
  return Array.from({ length: count }, (_, i) => Uint8Array.from({ length: size }, (_, j) => (i * 7 + j) & 0xff));
}

const SECOND = 48_000;

function file(packets: Uint8Array[], totalSamples: number, preSkip = 104): Uint8Array {
  return writeOggOpus({
    packets,
    packetSamples: packets.map(() => 960),
    preSkip,
    inputSampleRate: 16_000,
    totalSamples,
    serial: 0xabcdef01,
  });
}

describe("oggCrc", () => {
  it("agrees with the bit-by-bit definition", () => {
    for (const bytes of [new Uint8Array(0), Uint8Array.from([0]), Uint8Array.from([0x4f, 0x67, 0x67, 0x53]), packetsOf(1, 1000)[0] ?? new Uint8Array()]) {
      expect(oggCrc(bytes)).toBe(slowCrc(bytes));
    }
  });
});

describe("writeOggOpus", () => {
  it("writes OpusHead, OpusTags, then the packets byte for byte, every page with a good CRC", () => {
    const packets = packetsOf(120);
    const pages = readPages(file(packets, 120 * 960));
    expect(pages.every((p) => p.crcOk)).toBe(true);
    expect(pages.map((p) => p.sequence)).toEqual(pages.map((_, i) => i));
    expect(new Set(pages.map((p) => p.serial))).toEqual(new Set([0xabcdef01]));

    const head = pages[0]?.packets[0] ?? new Uint8Array();
    expect(String.fromCharCode(...head.subarray(0, 8))).toBe("OpusHead");
    expect(head.length).toBe(19);
    expect([head[8], head[9]]).toEqual([1, 1]);
    expect(new DataView(head.buffer).getUint16(10, true)).toBe(104);
    expect(new DataView(head.buffer).getUint32(12, true)).toBe(16_000);
    const tags = pages[1]?.packets[0] ?? new Uint8Array();
    expect(String.fromCharCode(...tags.subarray(0, 8))).toBe("OpusTags");

    const audio = pages.slice(2).flatMap((p) => p.packets);
    expect(audio).toEqual(packets);
  });

  it("marks the first page BOS and the last EOS, and only those", () => {
    const pages = readPages(file(packetsOf(120), 120 * 960));
    expect(pages.map((p) => p.flags)).toEqual([0x02, 0, 0, 0, 0x04]);
  });

  it("puts at most 50 packets on a page, with rising granules and the clip's end on the last", () => {
    const total = 120 * 960 - 500;
    const pages = readPages(file(packetsOf(120), total, 104));
    const audio = pages.slice(2);
    expect(audio.map((p) => p.packets.length)).toEqual([PACKETS_PER_PAGE, PACKETS_PER_PAGE, 20]);
    expect(audio.map((p) => p.granule)).toEqual([BigInt(104 + 50 * 960), BigInt(104 + 100 * 960), BigInt(104 + total)]);
    expect(pages.slice(0, 2).map((p) => p.granule)).toEqual([0n, 0n]);
  });

  it("laces a packet of 600 bytes as 255, 255, 90, and one of 510 as 255, 255, 0", () => {
    const big = packetsOf(2, 600);
    const exact = packetsOf(1, 510);
    const pages = readPages(file([...big, ...exact], 3 * 960));
    expect(pages.slice(2).flatMap((p) => p.packets)).toEqual([...big, ...exact]);
    expect(pages[2]?.table).toEqual([255, 255, 90, 255, 255, 90, 255, 255, 0]);
  });

  it("starts a new page before a packet whose lacing would pass 255 values", () => {
    // A packet of 1,500 bytes takes 6 lacing values: 42 of them fit in 255, the 43rd starts a page.
    const packets = packetsOf(45, 1_500);
    const pages = readPages(file(packets, 45 * 960));
    expect(pages.slice(2).map((p) => p.packets.length)).toEqual([42, 3]);
  });

  it("writes an empty last page for no packets, still with the clip's end", () => {
    const pages = readPages(file([], 960));
    expect(pages.map((p) => p.flags)).toEqual([0x02, 0, 0x04]);
    expect(pages[2]?.granule).toBe(BigInt(104 + 960));
  });

  it("needs one sample count per packet", () => {
    expect(() => writeOggOpus({ packets: packetsOf(2), packetSamples: [960], preSkip: 0, inputSampleRate: 16_000, totalSamples: 960, serial: 1 })).toThrow();
  });

  it("keeps the assumed pre-skip and a one-second page in its named constants", () => {
    expect(DEFAULT_PRE_SKIP).toBe(312);
    expect(PACKETS_PER_PAGE * 960).toBe(SECOND);
    expect(opusHead(104, 16_000).length).toBe(19);
  });
});
