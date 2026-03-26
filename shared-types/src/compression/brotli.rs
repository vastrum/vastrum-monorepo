use std::io::Write;

#[derive(thiserror::Error, Debug)]
pub enum BrotliError {
    #[error("brotli decompress failed: {0}")]
    Decompress(#[from] std::io::Error),
    #[error("decompressed page is not valid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub fn brotli_compress(data: &[u8]) -> Vec<u8> {
    let quality = if cfg!(debug_assertions) { 5 } else { 8 };
    let mut output = Vec::new();
    let mut writer = brotli::CompressorWriter::new(&mut output, 4096, quality, 22);
    writer.write_all(data).unwrap();
    drop(writer);
    output
}

pub fn brotli_compress_html(html: &str) -> Vec<u8> {
    brotli_compress(html.as_bytes())
}

pub fn brotli_decompress_html(data: &[u8]) -> Result<String, BrotliError> {
    if data.is_empty() {
        return Ok(String::new());
    }
    let mut output = Vec::new();
    brotli::BrotliDecompress(&mut std::io::Cursor::new(data), &mut output)?;
    Ok(String::from_utf8(output)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let html = "<html><body>hello world</body></html>";
        let compressed = brotli_compress_html(html);
        let decompressed = brotli_decompress_html(&compressed).unwrap();
        assert_eq!(decompressed, html);
    }

    #[test]
    fn empty_input() {
        assert_eq!(brotli_compress_html("").len(), 1);
        assert_eq!(brotli_decompress_html(&[]).unwrap(), "");
    }
}
