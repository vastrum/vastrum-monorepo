pub fn decompress_if_gzip(data: Vec<u8>) -> Vec<u8> {
    if data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b {
        use std::io::Read;
        let mut decoder = flate2::read::GzDecoder::new(&data[..]);
        let mut out = Vec::new();
        if decoder.read_to_end(&mut out).is_ok() {
            return out;
        }
    }
    return data;
}
