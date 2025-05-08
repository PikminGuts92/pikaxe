// Reference: https://github.com/eurotools/es-ima-adpcm-encoder-decoder/tree/main

const ADPCM_INDEX_TABLE: [i32; 16] = [
    -1, -1, -1, -1, 2, 4, 6, 8,
    -1, -1, -1, -1, 2, 4, 6, 8,
];

const ADPCM_STEP_SIZE_TABLE: [i32; 89] = [
        7,     8,     9,    10,    11,    12,    13,    14,    16,    17,
       19,    21,    23,    25,    28,    31,    34,    37,    41,    45,
       50,    55,    60,    66,    73,    80,    88,    97,   107,   118,
      130,   143,   157,   173,   190,   209,   230,   253,   279,   307,
      337,   371,   408,   449,   494,   544,   598,   658,   724,   796,
      876,   963,  1060,  1166,  1282,  1411,  1552,  1707,  1878,  2066,
     2272,  2499,  2749,  3024,  3327,  3660,  4026,  4428,  4871,  5358,
     5894,  6484,  7132,  7845,  8630,  9493, 10442, 11487, 12635, 13899,
    15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];

pub struct ADPCMDecoder {
    state: (i32, i32)
}

impl ADPCMDecoder {
    pub fn new() -> Self {
        Self {
            state: (0, 0)
        }
    }

    pub fn reset(&mut self) {
        self.state = (0, 0);
    }

    pub fn decode(&mut self, data: &[u8]) -> Vec<i16> {
        let mut buffer = vec![0i16; data.len() * 2];
        self.decode_with_buffer(data, &mut buffer);
        buffer
    }

    pub fn decode_with_buffer(&mut self, data: &[u8], buffer: &mut [i16]) {
        let (ref mut prev_val, ref mut index) = self.state;
        let mut step = ADPCM_STEP_SIZE_TABLE[*index as usize];

        for i in 0..(data.len() * 2) {
            // Get delta value
            let delta = if (i & 1) == 0 {
                data[i >> 1] & 0xf
            } else {
                (data[i >> 1] & 0xf0) >> 4
            };

            // Get new index value (keep within step table range)
            *index += ADPCM_INDEX_TABLE[delta as usize];
            if *index < 0 {
                *index = 0;
            } else if *index >= ADPCM_STEP_SIZE_TABLE.len() as i32 {
                *index = (ADPCM_STEP_SIZE_TABLE.len() - 1) as i32;
            }

            // Separate sign + magnitude
            let sign = delta & 8;
            let magnitude = delta & 7;

            // Compute difference + new predicted value
            let mut vp_diff = step >> 3;
            if (magnitude & 4) != 0 { vp_diff += step }
            if (magnitude & 2) != 0 { vp_diff += step >> 1 }
            if (magnitude & 1) != 0 { vp_diff += step >> 2 }

            *prev_val = if sign != 0 {
                *prev_val - vp_diff
            } else {
                *prev_val + vp_diff
            };

            // Clamp output value
            if *prev_val > std::i16::MAX as i32 {
                *prev_val = std::i16::MAX as i32;
            } else if *prev_val < std::i16::MIN as i32 {
                *prev_val = std::i16::MIN as i32;
            }

            // Update step value
            step = ADPCM_STEP_SIZE_TABLE[*index as usize];

            // Update sample value
            buffer[i] = *prev_val as i16;
        }
    }
}