const STR_INTERLEAVE_SIZE: usize = 512;
const STR_CHANNELS: usize = 2;

pub fn deinterleave_str(data: &mut [u8]) {
    let mut buffer = vec![0u8; STR_INTERLEAVE_SIZE * STR_CHANNELS];

    for chunk in data.chunks_mut(STR_INTERLEAVE_SIZE * STR_CHANNELS) {
        let half_size = chunk.len() / 2;

        // Left chunk
        for (i, d) in chunk.iter().take(half_size).enumerate() {
            buffer[((i >> 1) * 4) + (i & 1)] = *d;
        }

        // Right chunk
        for (i, d) in chunk.iter().skip(half_size).enumerate() {
            buffer[((i >> 1) * 4) + (i & 1) + 2] = *d;
        }

        // Copy buffer
        chunk.copy_from_slice(&buffer[..chunk.len()]);
    }
}