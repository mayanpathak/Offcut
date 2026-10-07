// The smallest valid WebAssembly module that uses SIMD: one function that
// returns a v128 value built with two SIMD instructions. `WebAssembly.validate`
// accepts it only where fixed-width SIMD is supported.
export const SIMD_PROBE_BYTES = new Uint8Array([
  0, 97, 115, 109, 1, 0, 0, 0, 1, 5, 1, 96, 0, 1, 123, 3, 2, 1, 0,
  10, 10, 1, 8, 0, 65, 0, 253, 15, 253, 98, 11,
]);
