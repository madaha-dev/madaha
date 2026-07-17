const Y: bool = true;
const N: bool = !Y;

const XGLITE_SUPPORT: [[bool; 128]; 128] = [
    /* LSB=  0 */ [Y; 128],
    /* LSB=  1 */ [Y; 128],
    /* LSB=  2 */ [Y; 128],
    /* LSB=  3 */ [Y; 128],
    /* LSB=  4 */ [Y; 128],
    /* LSB=  5 */ [Y; 128],
    /* LSB=  6 */ [Y; 128],
    /* LSB=  7 */ [Y; 128],
    /* LSB=  8,   1 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB=  9 */ [Y; 128],
    /* LSB= 10 */ [Y; 128],
    /* LSB= 11 */ [Y; 128],
    /* LSB= 12,   1 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 13 */ [Y; 128],
    /* LSB= 14,   1 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 15 */ [Y; 128],
    /* LSB= 16,  32 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, N, N, N, N, N,
        N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, N, Y, Y, Y, N, Y, N, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 17,   2 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 18,   5 fallbacks */
    [
        N, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 19 */ [Y; 128],
    /* LSB= 20,   2 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 21 */ [Y; 128],
    /* LSB= 22 */ [Y; 128],
    /* LSB= 23 */ [Y; 128],
    /* LSB= 24,   5 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, N, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 25,   3 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 26 */ [Y; 128],
    /* LSB= 27,   6 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, N, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 28 */ [Y; 128],
    /* LSB= 29 */ [Y; 128],
    /* LSB= 30 */ [Y; 128],
    /* LSB= 31 */ [Y; 128],
    /* LSB= 32,   4 fallbacks */
    [
        Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 33,   1 fallbacks */
    [
        Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 34,   1 fallbacks */
    [
        Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 35,   1 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 36,   1 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 37 */ [Y; 128],
    /* LSB= 38,   1 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 39 */ [Y; 128],
    /* LSB= 40,   8 fallbacks */
    [
        Y, Y, N, N, N, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, N, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 41,   4 fallbacks */
    [
        Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, N, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 42,   2 fallbacks */
    [
        Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 43,   2 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 44 */ [Y; 128],
    /* LSB= 45,   7 fallbacks */
    [
        Y, Y, Y, Y, N, N, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, N, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 46 */ [Y; 128],
    /* LSB= 47 */ [Y; 128],
    /* LSB= 48 */ [Y; 128],
    /* LSB= 49 */ [Y; 128],
    /* LSB= 50 */ [Y; 128],
    /* LSB= 51 */ [Y; 128],
    /* LSB= 52 */ [Y; 128],
    /* LSB= 53 */ [Y; 128],
    /* LSB= 54 */ [Y; 128],
    /* LSB= 55 */ [Y; 128],
    /* LSB= 56 */ [Y; 128],
    /* LSB= 57 */ [Y; 128],
    /* LSB= 58 */ [Y; 128],
    /* LSB= 59 */ [Y; 128],
    /* LSB= 60 */ [Y; 128],
    /* LSB= 61 */ [Y; 128],
    /* LSB= 62 */ [Y; 128],
    /* LSB= 63 */ [Y; 128],
    /* LSB= 64,  15 fallbacks */
    [
        Y, Y, Y, Y, N, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, N, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, N, N, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, N, N, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, N, Y, Y, Y, Y,
        Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 65,   9 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, N, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, N,
        N, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 66,   6 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        N, Y, Y, Y, Y, N, Y, Y, N, Y, Y, N, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 67,   9 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        N, N, N, N, N, N, N, N, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 68,   2 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 69,   2 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 70 */ [Y; 128],
    /* LSB= 71 */ [Y; 128],
    /* LSB= 72 */ [Y; 128],
    /* LSB= 73 */ [Y; 128],
    /* LSB= 74 */ [Y; 128],
    /* LSB= 75 */ [Y; 128],
    /* LSB= 76 */ [Y; 128],
    /* LSB= 77 */ [Y; 128],
    /* LSB= 78 */ [Y; 128],
    /* LSB= 79 */ [Y; 128],
    /* LSB= 80 */ [Y; 128],
    /* LSB= 81 */ [Y; 128],
    /* LSB= 82 */ [Y; 128],
    /* LSB= 83 */ [Y; 128],
    /* LSB= 84 */ [Y; 128],
    /* LSB= 85 */ [Y; 128],
    /* LSB= 86 */ [Y; 128],
    /* LSB= 87 */ [Y; 128],
    /* LSB= 88 */ [Y; 128],
    /* LSB= 89 */ [Y; 128],
    /* LSB= 90 */ [Y; 128],
    /* LSB= 91 */ [Y; 128],
    /* LSB= 92 */ [Y; 128],
    /* LSB= 93 */ [Y; 128],
    /* LSB= 94 */ [Y; 128],
    /* LSB= 95 */ [Y; 128],
    /* LSB= 96,   5 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, N, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 97,   2 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB= 98 */ [Y; 128],
    /* LSB= 99 */ [Y; 128],
    /* LSB=100 */ [Y; 128],
    /* LSB=101,   1 fallbacks */
    [
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, Y, N, Y, Y, Y, Y, Y, Y, Y,
        Y, Y, Y, Y, Y, Y, Y, Y,
    ],
    /* LSB=102 */ [Y; 128],
    /* LSB=103 */ [Y; 128],
    /* LSB=104 */ [Y; 128],
    /* LSB=105 */ [Y; 128],
    /* LSB=106 */ [Y; 128],
    /* LSB=107 */ [Y; 128],
    /* LSB=108 */ [Y; 128],
    /* LSB=109 */ [Y; 128],
    /* LSB=110 */ [Y; 128],
    /* LSB=111 */ [Y; 128],
    /* LSB=112 */ [Y; 128],
    /* LSB=113 */ [Y; 128],
    /* LSB=114 */ [Y; 128],
    /* LSB=115 */ [Y; 128],
    /* LSB=116 */ [Y; 128],
    /* LSB=117 */ [Y; 128],
    /* LSB=118 */ [Y; 128],
    /* LSB=119 */ [Y; 128],
    /* LSB=120 */ [Y; 128],
    /* LSB=121 */ [Y; 128],
    /* LSB=122 */ [Y; 128],
    /* LSB=123 */ [Y; 128],
    /* LSB=124 */ [Y; 128],
    /* LSB=125 */ [Y; 128],
    /* LSB=126 */ [Y; 128],
    /* LSB=127 */ [Y; 128],
];

pub fn map_xg_lite(lsb: u8, prog: u8) -> usize {
    if XGLITE_SUPPORT[lsb as usize][prog as usize] {
        lsb as usize
    } else {
        0usize
    }
}
