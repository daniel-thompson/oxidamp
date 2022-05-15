const KICK_DRY_B: [i8; 8192] = [
    0, 0, 1, 0, 0, -1, -1, -2, -2, -3, -3, -3, -5, -5, -6, -9, -10, -11, -12, -14, -16, -19, -21,
    -23, -26, -28, -30, -34, -36, -38, -43, -45, -48, -50, -54, -58, -60, -61, -64, -70, -73, -73,
    -73, -80, -86, -87, -85, -81, -95, -107, -103, -90, -89, -99, -109, -116, -114, -111, -108,
    -100, -99, -107, -119, -120, -120, -120, -118, -103, -99, -115, -120, -119, -120, -120, -119,
    -120, -117, -112, -111, -118, -119, -99, -80, -74, -70, -59, -52, -57, -64, -58, -44, -34, -19,
    -9, -8, -12, -8, 3, 7, 9, 17, 32, 45, 47, 48, 54, 61, 67, 70, 77, 89, 98, 105, 110, 115, 117,
    122, 125, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127,
    127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127, 127,
    127, 125, 123, 125, 125, 124, 124, 124, 123, 124, 124, 124, 123, 124, 124, 124, 123, 124, 124,
    124, 123, 123, 124, 123, 124, 124, 124, 124, 124, 124, 124, 123, 123, 124, 124, 123, 124, 124,
    124, 123, 124, 121, 119, 113, 104, 101, 99, 97, 91, 85, 80, 75, 70, 69, 59, 51, 44, 37, 37, 33,
    26, 26, 23, 14, 10, 8, 2, -2, -4, -7, -9, -19, -26, -25, -27, -27, -32, -36, -40, -44, -48,
    -50, -54, -59, -63, -66, -69, -73, -70, -68, -73, -81, -82, -82, -86, -85, -90, -95, -99, -93,
    -90, -91, -99, -103, -103, -103, -103, -107, -108, -107, -108, -106, -102, -106, -109, -112,
    -114, -112, -109, -108, -112, -111, -112, -110, -112, -108, -109, -111, -112, -109, -107, -107,
    -106, -107, -107, -108, -103, -102, -100, -98, -97, -93, -96, -93, -95, -93, -90, -86, -90,
    -88, -83, -80, -77, -77, -73, -74, -71, -70, -68, -66, -64, -61, -62, -62, -59, -57, -55, -52,
    -51, -52, -53, -51, -52, -51, -53, -53, -53, -55, -54, -53, -52, -51, -51, -49, -50, -53, -53,
    -54, -53, -53, -53, -53, -54, -55, -56, -56, -58, -59, -60, -60, -60, -59, -59, -60, -62, -64,
    -68, -70, -70, -69, -70, -68, -71, -73, -74, -73, -75, -78, -79, -82, -81, -82, -82, -82, -82,
    -82, -81, -82, -85, -86, -84, -82, -86, -85, -87, -86, -86, -86, -82, -82, -85, -86, -85, -86,
    -86, -86, -86, -86, -86, -86, -87, -84, -83, -86, -82, -82, -82, -82, -82, -79, -77, -78, -78,
    -78, -76, -73, -73, -74, -74, -74, -72, -70, -68, -65, -66, -66, -67, -65, -67, -65, -64, -61,
    -59, -61, -59, -57, -58, -58, -56, -55, -54, -53, -50, -50, -47, -47, -45, -46, -45, -46, -46,
    -46, -45, -41, -39, -39, -39, -39, -39, -38, -36, -37, -36, -35, -34, -35, -34, -33, -31, -31,
    -31, -30, -30, -30, -30, -29, -28, -29, -29, -28, -27, -27, -27, -28, -25, -25, -26, -27, -28,
    -26, -26, -25, -24, -24, -24, -26, -24, -24, -24, -26, -26, -25, -24, -23, -23, -23, -23, -22,
    -22, -24, -23, -22, -23, -23, -22, -24, -23, -23, -22, -21, -21, -21, -22, -24, -22, -22, -22,
    -24, -22, -21, -19, -18, -19, -20, -20, -19, -18, -19, -17, -15, -17, -17, -16, -18, -18, -14,
    -14, -12, -10, -10, -11, -11, -9, -8, -7, -7, -6, -5, -4, -4, -4, -4, -3, -2, -2, 0, 0, 0, 3,
    2, 3, 3, 4, 5, 4, 7, 9, 9, 10, 10, 12, 13, 12, 14, 14, 14, 15, 15, 17, 19, 19, 20, 21, 22, 21,
    23, 23, 23, 23, 23, 23, 22, 21, 21, 20, 21, 19, 20, 20, 20, 21, 21, 22, 22, 21, 22, 21, 22, 22,
    23, 24, 25, 25, 26, 28, 29, 29, 30, 31, 30, 31, 32, 35, 34, 36, 37, 38, 38, 38, 42, 41, 42, 41,
    42, 44, 45, 46, 47, 49, 51, 51, 51, 52, 53, 53, 54, 57, 57, 59, 60, 62, 62, 64, 63, 64, 67, 65,
    68, 70, 70, 69, 70, 70, 68, 72, 73, 73, 74, 73, 73, 73, 74, 74, 74, 78, 74, 73, 76, 76, 72, 76,
    76, 75, 78, 77, 77, 78, 78, 78, 78, 78, 77, 78, 78, 77, 78, 77, 77, 78, 78, 77, 77, 77, 77, 77,
    78, 73, 73, 74, 73, 74, 74, 73, 74, 74, 71, 69, 69, 69, 69, 69, 70, 69, 69, 69, 69, 69, 67, 68,
    67, 65, 66, 65, 65, 65, 65, 63, 65, 63, 62, 61, 60, 59, 57, 58, 56, 54, 53, 53, 54, 51, 51, 51,
    49, 48, 47, 47, 45, 44, 42, 44, 41, 40, 41, 39, 39, 37, 37, 35, 35, 35, 33, 33, 31, 30, 30, 28,
    29, 27, 26, 24, 22, 23, 21, 20, 20, 18, 17, 17, 16, 16, 14, 13, 12, 11, 10, 10, 10, 9, 8, 8, 7,
    5, 5, 6, 5, 5, 4, 5, 3, 2, 2, 2, 0, 0, 0, -1, -2, -2, -3, -5, -4, -5, -4, -4, -6, -6, -6, -7,
    -7, -7, -7, -7, -8, -8, -7, -7, -6, -6, -6, -6, -4, -6, -6, -5, -5, -4, -4, -4, -3, -3, -5, -3,
    -5, -6, -5, -6, -4, -5, -5, -5, -5, -5, -5, -4, -4, -4, -3, -3, -4, -4, -3, -3, -3, -3, -1, -2,
    -2, -2, -2, -1, -1, -2, -1, -2, -1, -1, -1, -1, -1, -1, -1, 1, -1, 0, 0, 1, 1, 1, 1, 1, 2, 1,
    3, 4, 3, 3, 4, 3, 2, 2, 3, 2, 1, 2, 2, 2, 3, 1, 1, 2, 3, 2, 1, 0, 1, 0, -1, 1, -1, -1, -2, -3,
    -4, -4, -5, -5, -5, -7, -7, -7, -7, -9, -9, -11, -12, -12, -13, -14, -16, -16, -15, -16, -18,
    -17, -19, -18, -20, -20, -20, -22, -20, -21, -22, -22, -23, -23, -24, -23, -23, -24, -25, -24,
    -23, -24, -23, -23, -24, -24, -25, -24, -25, -25, -27, -25, -25, -26, -25, -25, -24, -26, -26,
    -25, -25, -25, -25, -23, -24, -24, -23, -24, -23, -23, -23, -22, -22, -23, -21, -22, -21, -22,
    -20, -20, -21, -21, -20, -19, -19, -20, -19, -20, -21, -20, -20, -20, -19, -21, -20, -19, -20,
    -19, -21, -19, -18, -19, -19, -17, -19, -18, -16, -18, -18, -18, -17, -17, -17, -17, -17, -17,
    -17, -17, -18, -17, -18, -18, -18, -18, -17, -18, -17, -18, -17, -17, -17, -18, -17, -16, -17,
    -17, -18, -19, -18, -17, -18, -20, -19, -17, -18, -19, -18, -18, -18, -18, -18, -17, -17, -16,
    -16, -17, -16, -15, -16, -15, -16, -14, -14, -14, -13, -14, -14, -12, -13, -11, -12, -12, -13,
    -13, -12, -11, -13, -12, -14, -14, -13, -13, -13, -14, -14, -13, -14, -15, -14, -15, -15, -15,
    -16, -15, -15, -15, -15, -17, -16, -17, -17, -17, -17, -17, -18, -17, -17, -18, -17, -17, -17,
    -18, -16, -18, -17, -16, -17, -17, -17, -18, -17, -17, -16, -16, -18, -19, -18, -20, -20, -19,
    -19, -19, -21, -20, -21, -20, -20, -21, -19, -20, -20, -20, -20, -20, -20, -19, -20, -20, -19,
    -19, -19, -18, -17, -18, -18, -16, -15, -16, -16, -15, -14, -14, -14, -13, -13, -13, -13, -13,
    -11, -11, -11, -10, -10, -9, -9, -10, -9, -9, -7, -7, -7, -7, -7, -6, -6, -6, -5, -4, -4, -4,
    -3, -3, -2, -1, -2, -1, 0, 0, 1, 1, 1, 1, 1, 2, 3, 3, 3, 3, 4, 3, 4, 3, 4, 5, 4, 4, 4, 4, 4, 5,
    5, 5, 5, 4, 5, 6, 5, 5, 4, 5, 5, 4, 6, 5, 5, 5, 6, 6, 5, 5, 5, 5, 5, 5, 5, 3, 4, 3, 4, 3, 3, 3,
    3, 3, 4, 3, 2, 2, 3, 2, 2, 3, 2, 2, 1, 1, 1, 0, 1, -1, -1, -1, -1, -1, -1, -2, -2, -2, -2, -2,
    -3, -2, -3, -3, -3, -3, -4, -4, -4, -3, -4, -3, -4, -3, -4, -4, -3, -4, -3, -3, -4, -3, -4, -4,
    -3, -3, -3, -3, -3, -3, -3, -3, -2, -2, -3, -3, -2, -2, -2, -2, -3, -3, -2, -1, -2, -1, -2, -2,
    -1, -1, -1, -1, -1, 0, -2, 0, -1, -1, 0, 0, 0, -1, 0, 0, -1, -2, -1, -1, -1, -2, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -2, -1, -2, -1, -1, -1, -2, 0, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -2, -2, -1, -1, -1, -2, -2, -1, 0, -1, 0, -1, 0, 0, -1, -1, 0, -1, -2, 0, -1, -1, 0,
    -1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1,
    0, 1, 1, 2, 1, 2, 1, 1, 2, 2, 2, 2, 3, 3, 2, 2, 2, 2, 3, 3, 3, 4, 3, 3, 3, 2, 3, 4, 4, 4, 5, 4,
    4, 3, 4, 4, 4, 5, 5, 4, 4, 4, 6, 5, 4, 6, 6, 6, 6, 6, 7, 7, 7, 8, 8, 8, 7, 7, 7, 8, 8, 7, 7, 8,
    7, 7, 7, 7, 7, 6, 6, 7, 6, 6, 7, 6, 6, 6, 6, 6, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 4, 4, 5, 4, 4,
    4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 4, 5, 6, 5, 6, 6, 6, 5, 6, 6, 6, 6, 6, 7, 6, 6, 6, 7, 7, 6, 7, 7,
    7, 7, 7, 6, 8, 7, 8, 9, 8, 7, 9, 7, 9, 8, 9, 9, 10, 9, 9, 10, 10, 10, 10, 10, 11, 11, 11, 11,
    12, 12, 12, 12, 12, 13, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14, 14, 15, 15, 15, 16, 15, 15, 15,
    15, 16, 17, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 15, 15, 15, 16, 15, 15, 15, 15,
    15, 14, 14, 15, 15, 16, 15, 15, 15, 14, 14, 14, 14, 14, 14, 14, 14, 14, 13, 13, 14, 13, 13, 13,
    13, 12, 13, 12, 13, 13, 13, 13, 12, 12, 12, 13, 12, 12, 13, 12, 12, 12, 12, 12, 12, 13, 13, 13,
    12, 13, 13, 13, 13, 13, 14, 13, 13, 13, 14, 14, 14, 14, 14, 13, 14, 14, 14, 14, 14, 15, 15, 15,
    15, 15, 15, 15, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 17, 18, 18, 17, 17, 17, 18, 17, 17, 17,
    17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 16, 17, 16, 16, 17, 16, 16, 15, 15, 15,
    15, 15, 14, 14, 15, 14, 15, 14, 14, 14, 14, 14, 13, 14, 14, 13, 14, 14, 13, 13, 13, 13, 13, 12,
    12, 13, 12, 12, 12, 12, 12, 12, 12, 12, 12, 11, 11, 11, 11, 10, 11, 10, 11, 11, 10, 10, 10, 11,
    9, 9, 9, 10, 10, 10, 9, 10, 9, 9, 9, 9, 9, 9, 9, 8, 9, 9, 8, 8, 10, 9, 8, 8, 8, 8, 8, 7, 8, 8,
    7, 7, 7, 8, 8, 7, 8, 8, 7, 7, 7, 7, 6, 6, 7, 6, 6, 6, 6, 6, 6, 6, 5, 5, 4, 5, 5, 4, 5, 4, 3, 3,
    4, 3, 4, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 0, 1, 1, 0, 0, -1, 1, 0, 0, 0, 0, 0, -1, -1, 0,
    0, -1, -1, -1, -1, -1, -2, -1, -2, -2, -2, -2, -2, -2, -2, -3, -2, -2, -3, -3, -4, -4, -3, -4,
    -4, -4, -4, -4, -5, -6, -5, -5, -5, -5, -7, -6, -6, -7, -6, -7, -7, -8, -7, -7, -7, -8, -8, -7,
    -8, -8, -8, -8, -9, -8, -9, -9, -8, -9, -9, -9, -9, -9, -9, -10, -9, -10, -10, -9, -9, -10,
    -10, -11, -10, -9, -10, -10, -10, -10, -10, -10, -9, -9, -10, -10, -10, -10, -9, -10, -9, -9,
    -10, -9, -9, -10, -10, -9, -10, -10, -10, -9, -10, -9, -10, -9, -9, -9, -10, -9, -10, -10, -10,
    -9, -9, -10, -9, -9, -10, -9, -9, -9, -9, -8, -9, -9, -9, -9, -8, -9, -8, -8, -9, -8, -8, -8,
    -9, -8, -8, -9, -8, -8, -8, -9, -9, -8, -9, -8, -9, -9, -9, -9, -8, -9, -10, -8, -9, -9, -9,
    -9, -9, -10, -10, -9, -9, -9, -10, -9, -9, -9, -9, -9, -9, -9, -9, -9, -8, -10, -9, -10, -9,
    -9, -9, -9, -9, -9, -9, -9, -9, -9, -9, -8, -9, -9, -10, -9, -9, -9, -9, -9, -9, -10, -9, -9,
    -9, -10, -9, -9, -9, -9, -8, -9, -9, -9, -9, -9, -9, -8, -9, -9, -8, -9, -7, -8, -8, -8, -8,
    -6, -7, -7, -7, -6, -6, -6, -6, -6, -6, -7, -6, -6, -6, -6, -5, -6, -5, -5, -5, -5, -5, -5, -4,
    -5, -5, -5, -5, -4, -4, -5, -4, -4, -4, -4, -3, -3, -3, -3, -3, -3, -3, -3, -2, -2, -2, -2, -1,
    -2, -2, -2, -1, -1, -1, -1, -2, -1, 0, -1, 0, -1, -1, -1, 0, -1, -1, -1, -1, -1, -1, -1, -2,
    -2, -2, -2, -2, -2, -3, -2, -3, -3, -3, -3, -4, -3, -4, -3, -4, -4, -4, -5, -3, -4, -5, -4, -4,
    -4, -5, -5, -4, -5, -4, -6, -5, -5, -5, -6, -5, -5, -6, -5, -6, -5, -6, -5, -6, -6, -5, -6, -5,
    -5, -6, -6, -5, -6, -6, -6, -5, -6, -6, -6, -6, -6, -6, -5, -7, -6, -6, -6, -5, -6, -5, -5, -6,
    -6, -5, -6, -6, -5, -5, -6, -5, -5, -5, -4, -5, -5, -4, -5, -4, -5, -5, -4, -4, -3, -3, -4, -4,
    -4, -3, -3, -3, -3, -3, -3, -3, -2, -2, -2, -2, -2, -2, -2, -3, -2, -3, -2, -2, -2, -2, -2, -1,
    -1, -1, -1, -2, -1, -1, -1, 0, 0, 0, -1, 0, -1, 0, 0, -1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1,
    0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1,
    1, 0, 2, 1, 1, 1, 1, 2, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 2, 3, 2, 1, 3, 2, 2, 2, 2, 2, 2, 2, 3, 3,
    1, 3, 3, 2, 2, 3, 2, 2, 3, 2, 3, 3, 3, 2, 2, 2, 2, 3, 2, 2, 3, 2, 3, 2, 2, 3, 3, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 1, 1, 2, 2, 2,
    2, 2, 1, 2, 3, 2, 2, 2, 3, 2, 2, 3, 3, 2, 3, 2, 3, 2, 2, 3, 3, 2, 3, 3, 3, 2, 3, 2, 3, 3, 3, 3,
    3, 2, 3, 3, 4, 3, 3, 3, 4, 4, 4, 3, 3, 4, 4, 3, 4, 4, 4, 5, 4, 3, 4, 4, 4, 4, 5, 5, 4, 5, 5, 5,
    5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 7, 6, 7, 6, 7, 6, 7, 7, 7, 7, 7, 7, 7, 8, 7, 7,
    6, 7, 7, 7, 7, 7, 7, 7, 8, 7, 8, 8, 8, 7, 8, 7, 8, 8, 7, 7, 8, 7, 7, 8, 7, 8, 7, 8, 7, 8, 7, 8,
    6, 7, 7, 8, 7, 7, 6, 7, 6, 7, 6, 7, 6, 6, 7, 6, 6, 6, 6, 7, 6, 6, 5, 5, 6, 5, 5, 5, 5, 5, 4, 5,
    5, 5, 4, 5, 5, 5, 4, 4, 4, 4, 5, 4, 5, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 3, 4, 3,
    4, 3, 3, 4, 4, 3, 4, 4, 3, 3, 4, 4, 4, 4, 4, 4, 3, 4, 3, 4, 4, 5, 4, 4, 4, 4, 5, 5, 4, 5, 6, 5,
    4, 5, 5, 5, 6, 6, 5, 6, 5, 6, 6, 6, 6, 6, 6, 6, 6, 7, 6, 7, 6, 6, 7, 7, 7, 7, 7, 7, 7, 6, 7, 7,
    7, 8, 7, 7, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 7, 6, 6, 6, 6, 6, 6, 6, 5, 6,
    6, 5, 6, 6, 5, 6, 5, 5, 5, 4, 5, 5, 5, 4, 4, 3, 4, 4, 3, 3, 4, 3, 3, 3, 4, 4, 3, 3, 3, 4, 3, 3,
    3, 2, 2, 3, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 1, 2,
    2, 3, 2, 2, 2, 3, 2, 2, 2, 3, 2, 2, 2, 2, 3, 2, 3, 3, 3, 2, 2, 3, 3, 3, 3, 4, 3, 3, 4, 3, 2, 3,
    3, 3, 3, 3, 4, 3, 4, 3, 3, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 2, 3, 4, 3, 4, 4, 3, 3, 4, 3, 3, 3, 3,
    3, 3, 3, 4, 3, 3, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 3, 3, 2, 2, 2, 2, 3, 1, 2, 2, 1, 2,
    2, 1, 1, 2, 2, 1, 2, 0, 1, 2, 1, 0, 0, 1, 0, 0, 1, 1, 0, 1, -1, 1, 0, 1, -1, -1, -1, -1, 0, -1,
    0, -1, -1, -1, -1, -1, -1, -1, -2, -1, -1, -2, -1, -1, -2, -2, -2, -2, -1, -1, -2, -2, -2, -2,
    -2, -3, -2, -2, -2, -2, -3, -2, -3, -2, -3, -2, -3, -2, -2, -2, -2, -3, -2, -3, -2, -2, -2, -3,
    -2, -2, -3, -2, -3, -3, -2, -3, -3, -3, -2, -2, -2, -3, -3, -2, -3, -3, -2, -3, -2, -3, -2, -2,
    -2, -2, -2, -2, -2, -3, -2, -2, -2, -3, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2,
    -1, -2, -2, -1, -2, -2, -1, -1, -1, -2, -2, -2, -2, -1, -1, -2, -1, -2, -2, -2, -2, -2, -2, -2,
    -2, -2, -2, -2, -2, -2, -2, -1, -3, -1, -2, -2, -1, -1, -2, -2, -2, -2, -1, -3, -2, -2, -3, -2,
    -2, -2, -3, -2, -2, -2, -2, -2, -2, -2, -2, -3, -2, -3, -3, -3, -2, -2, -2, -2, -3, -3, -2, -2,
    -2, -2, -2, -2, -2, -3, -3, -3, -3, -2, -3, -3, -2, -3, -3, -3, -2, -2, -2, -3, -2, -2, -2, -2,
    -3, -3, -3, -2, -3, -2, -2, -3, -2, -2, -2, -2, -2, -3, -2, -3, -3, -2, -2, -2, -3, -2, -2, -2,
    -2, -3, -2, -2, -3, -2, -2, -2, -1, -2, -1, -2, -1, -1, -2, -2, -1, -1, -1, -1, -2, -2, -1, -2,
    -1, -2, -1, -1, -1, 0, -2, -2, -1, -1, -2, -2, -2, -1, -2, -1, -1, -2, -1, -1, -1, -1, -1, -1,
    -1, -1, 0, -1, -1, -2, 0, -1, -1, -1, 0, -1, -1, -1, -1, -2, -1, -1, -1, -1, -1, -1, -2, -1,
    -1, -2, -2, -2, -1, -2, -1, -2, -2, -1, -1, -1, -1, -1, -2, -2, -2, -1, -1, -2, -1, -3, -1, -2,
    -2, -1, -1, -2, -2, -2, -2, -3, -1, -3, -2, -2, -2, -2, -2, -1, -2, -2, -2, -3, -2, -3, -2, -3,
    -3, -2, -2, -3, -2, -2, -2, -2, -2, -3, -3, -3, -3, -1, -2, -2, -3, -2, -2, -2, -2, -2, -2, -2,
    -2, -2, -2, -1, -2, -2, -2, -1, -2, -2, -1, -2, -2, -2, -2, -1, -2, -2, -2, -2, -2, -1, -1, -2,
    -1, -1, 0, -1, -1, -1, -1, -1, 0, -1, -1, -1, -1, 0, 0, 0, -1, -1, -1, 0, 0, 0, -1, -1, 0, -1,
    0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0,
    0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 1, 0, 1, 0, 1, -1, 0, 0, 0, 1,
    0, 0, 1, 1, 0, -1, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, -1, 0, 0, 0, -1, 0, -1, 0, 0, 0, -1, 0,
    -1, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, -1, 1, 0, -1, 0, 0, 1, 1, 0, 0, -1, 0, 0, 0, 0, 0, -1, 0,
    0, -1, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 2, 1, 2, 2, 2, 2, 2, 2, 1, 1, 1, 2, 2, 2, 1, 2, 1, 2, 2, 3, 2, 3, 1, 1, 2, 2, 2, 3, 1,
    2, 2, 2, 2, 3, 3, 2, 2, 2, 2, 3, 2, 2, 2, 2, 3, 2, 1, 3, 3, 3, 2, 2, 2, 2, 2, 2, 3, 3, 3, 2, 3,
    3, 2, 3, 3, 3, 2, 3, 3, 3, 3, 3, 2, 3, 2, 2, 3, 3, 3, 4, 3, 4, 3, 4, 3, 3, 4, 2, 4, 2, 3, 3, 3,
    4, 3, 3, 3, 4, 3, 4, 3, 3, 3, 4, 3, 4, 3, 4, 3, 4, 4, 4, 3, 4, 4, 4, 3, 3, 4, 3, 4, 4, 4, 4, 3,
    4, 4, 4, 4, 5, 4, 4, 5, 4, 5, 4, 4, 4, 4, 5, 4, 5, 4, 5, 4, 4, 5, 4, 3, 4, 4, 5, 5, 4, 4, 4, 4,
    4, 4, 4, 4, 4, 3, 5, 4, 4, 4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 3, 3, 3, 4, 4, 4, 3, 3, 3, 4, 3, 3, 3,
    4, 3, 3, 3, 3, 3, 2, 3, 3, 2, 3, 3, 3, 4, 3, 3, 3, 3, 2, 4, 3, 2, 3, 3, 3, 3, 2, 4, 3, 2, 3, 3,
    3, 3, 3, 3, 3, 3, 2, 2, 3, 2, 3, 3, 4, 3, 2, 3, 3, 3, 3, 4, 3, 3, 3, 3, 3, 3, 3, 2, 3, 3, 2, 3,
    2, 2, 2, 3, 2, 3, 2, 3, 2, 3, 2, 2, 3, 2, 2, 2, 3, 3, 3, 2, 2, 2, 3, 2, 1, 2, 2, 2, 2, 2, 2, 1,
    2, 2, 2, 2, 2, 1, 1, 2, 1, 2, 1, 1, 3, 2, 1, 2, 2, 1, 2, 1, 1, 2, 1, 2, 1, 1, 1, 1, 2, 2, 1, 1,
    2, 1, 1, 1, 1, 1, 1, 2, 1, 2, 2, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 2, 2, 1, 1, 1, 2, 1, 1, 2, 2, 1,
    1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 2, 1, 2, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 2, 2, 2, 1,
    2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 1,
    2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 2, 1, 2, 2, 2, 1, 2, 1, 1, 1,
    1, 1, 2, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0,
    0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, -1, -1, 0, 0, 0, 0, -1, -1, 0, 0,
    0, -1, -1, 0, -1, 0, 0, -1, 0, 0, -1, 0, -1, 0, 0, 0, 0, 0, -1, -1, 0, -1, -1, -1, 0, 0, -1,
    -1, -1, -1, -1, -1, -1, 0, 0, -1, 0, -1, -1, -1, -1, -1, 0, 0, -1, -1, -1, -1, -1, 0, -1, 0, 0,
    -1, 0, 0, 0, 0, -1, -1, -1, 0, -1, 0, 0, 0, -1, -1, 0, 0, 0, 0, -1, 0, 0, 0, -1, 0, 0, -1, -1,
    -1, -1, 0, -1, 0, 0, -1, 0, 0, 0, -1, 0, 0, 0, -1, 0, 0, 0, 0, -1, -1, 0, 0, 0, -1, 0, 1, 0, 0,
    1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 0, 1, 1, 1, 2, 1, 0, 0, 1, 2, 1, 2, 1, 1, 0, 1, 2,
    1, 1, 2, 1, 2, 2, 1, 1, 2, 1, 1, 1, 1, 1, 2, 1, 1, 1, 2, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1,
    0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1,
    1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, -1, 0, 1, 0, -1, 0, 0, 0, -1, 0, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, -1, 0, 0, 0, -1, 0, 0, 0, 1, 0, 0, -1, -1, 0, -1, 0, 0, -1,
    0, 0, 0, 0, 0, -1, -1, -1, 0, -1, 1, 0, 0, -1, 0, 0, 0, 1, -1, 1, 0, -1, 0, 1, 0, 0, 0, -1, 0,
    1, 1, -1, -1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 2,
    1, 1, 1, 1, 1, 1, 1, 0, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2, 1, 1, 1, 2, 1, 2, 1,
    2, 1, 1, 2, 2, 1, 2, 1, 1, 2, 1, 1, 1, 1, 1, 2, 1, 1, 2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2,
    2, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 2, 2, 1, 2, 1, 1, 0, 1, 1, 2, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1,
    0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0,
    1, 1, 1, 0, 0, 1, 1, 2, 1, 0, 1, 1, 2, 0, 1, 1, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1,
    2, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 2, 1, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 1,
    1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 2, 1, 2, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1,
    1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1,
    1, 0, 0, 1, 1, 0, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 2, 1, 2, 1,
    2, 1, 2, 1, 1, 1, 1, 2, 1, 2, 2, 1, 1, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 1, 1, 1, 2, 1, 2, 2, 2, 2,
    1, 2, 2, 2, 2, 2, 1, 2, 2, 2, 1, 2, 2, 2, 1, 2, 1, 2, 2, 1, 2, 1, 1, 2, 2, 2, 2, 1, 2, 1, 2, 2,
    1, 1, 2, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 2, 0, 1, 1, 1, 1, 1, 2, 1, 1, 2, 1, 1, 1,
    1, 0, 1, 1, 0, 1, 2, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 2, 0, 1, 1, 1,
    1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 1,
    0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, 1,
    0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0,
    1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0,
    0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1,
    0, 1, 1, 1, 1, 1, 1, 1, 2, 0, 1, 1, 1, 1, 1, 1, 2, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 2, 1, 1, 2, 0, 1, 1, 1, 0, 1, 1, 1, 2, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1,
    1, 2, 1, 1, 2, 2, 1, 1, 2, 1, 1, 1, 2, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0,
    1, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, -1, 0, 0, 0, 0, 1, 0, -1, 0,
    0, 0, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 1, 0, -1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0,
    1, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 2, 1, 1, 1, 1, 1, 2, 1, 1,
    0, 2, 1, 1, 1, 2, 2, 0, 0, 1, 1, 1, 2, 1, 1, 2, 2, 1, 1, 1, 1, 2, 2, 2, 1, 1, 1, 1, 1, 2, 1, 0,
    1, 2, 0, 1, 1, 1, 1, 1, 1, 2, 0, 1, 1, 1, 2, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1,
    1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0,
    1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 2, 1, 1, 1, 0, 1, 1, 1, 0, 0,
    0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 2, 0, 1, 1, 0, 1, 0, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 2, 1, 0, 1, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0,
    0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 1,
    1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 2, 0, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 2, 1, 1, 0, 1, 0, 1, 1, 1, 2, 1, 1, 0, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 1, 2, 1, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 2, 1, 1, 1, 1, 1, 1, 0, 1, 0,
    1, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1,
    1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 2, 1, 1, 1, 2, 1, 1,
    0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 0, 0, 1, 2, 0, 1, 1, 0, 0, 2, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 0, 1,
    0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0,
    1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1,
    1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 2, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1,
    1, 1, 1, 1, 2, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2,
    1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1,
    0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 1, 0,
    0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0,
    1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1, 0,
    1, 0, 0, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 1, 1,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1,
    1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0,
    0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1,
    0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0,
    0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 0, 1,
    0, 0, 0, 0, 1, 0, 0, 0, -1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
    0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 0,
    1, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1,
    1, 1, 1, 1, 1, 0, 0, 1, 0, 1, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1,
    0, 0, 1, 0, 0, 1, 1, 0, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
    1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
    0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0,
    0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 0,
    0, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0,
    0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1,
    -1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
    0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0, -1, -1, 1, 1, -1, 0,
    1, 0, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, -1, 0, 0, 1, 0, 0, 0, 0,
    0, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0,
    0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 1, -1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1,
    0, 0, 0, 1, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 1, 1,
    -1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, -1, 0, 0, 0, 0, 1, 0, 0, 0,
    0, 0, 0, 0, 0, 1, -1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, -1, 1, 0, 0, 1,
    1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0,
    1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 1, -1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1,
    0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 1,
    1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0, -1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0,
    0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, -1, 1, 0, 0, 0, -1, -1, 0, 1, 0, 0, -1, 1, 0, 0, 1, 0, 0, 0,
    -1, 0, 0, 0, 0, 1, 0, 0, 0, -1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 0, 1, 0, 1, 0, 0,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, -1, 0, 0, 0, 1,
    0, 1, 0, 1, -1, 1, 0, -1, 0, -1, 0, 0, 1, 0, 0, 1, 0, 0, -1, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0,
    0, -1, -1, 0, 1, 0, 0, 0, -1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, -1, 1, 0, 0, 0, 0,
    1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, -1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
    0, 0, 1, 0, 0, 0, -1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 1, 0, 0,
    1, 0, -1, 0, 0, 0, 0, 0, -1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 0, -1, 0, -1, 0, 0, -1, -1, 0, 0, 0, 0,
    0, 1, 0, 1, 0, 0, -1, 0, 1, -1, 0, -1, 0, 0, 1, 0, 0, 0, 0, -1, 0, 0, 0, 0, -1, 0, 0, 0, 0, -1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];