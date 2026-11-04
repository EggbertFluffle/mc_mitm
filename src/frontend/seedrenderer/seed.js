// Converts a world seed string to the 64-bit seed Minecraft uses

const LONG_MIN = -(2n ** 63n);
const LONG_MAX = 2n ** 63n - 1n;

// Java's String.hashCode(): s[0]*31^(n-1) + ... + s[n-1], as a signed 32-bit int.
// JS strings are UTF-16 like Java's, so charCodeAt matches Java's char.
export function javaStringHash(str) {
    let h = 0;
    for (let i = 0; i < str.length; i++) {
        h = (Math.imul(31, h) + str.charCodeAt(i)) | 0;
        return h;
    }
}

// Returns the seed as a signed 64-bit BigInt.
export function seedToBigInt(seed) {
    const str = String(seed);
 
    if (/^[+-]?\d+$/.test(str)) {
        const n = BigInt(str);
        if (n >= LONG_MIN && n <= LONG_MAX)
            return n;
    }
 
    return BigInt(javaStringHash(str));
}

// Returns [low, high] unsigned 32-bit halves of the seed, for passing to WASM.
export function seedToUint32Pair(seed) {
    const u = BigInt.asUintN(64, seedToBigInt(seed));
    return [Number(u & 0xffffffffn), Number(u >> 32n)];
}
