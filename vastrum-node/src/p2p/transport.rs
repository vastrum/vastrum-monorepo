use crate::utils::limits::MAX_FRAME_SIZE;

pub struct FramedReader {
    reader: ReadHalf<TcpStream>,
}
impl FramedReader {
    pub fn new(reader: ReadHalf<TcpStream>) -> Self {
        FramedReader { reader }
    }

    pub async fn read_frame(&mut self) -> eyre::Result<Vec<u8>> {
        let mut len_bytes = [0u8; 4];
        self.reader.read_exact(&mut len_bytes).await?;

        let len = u32::from_be_bytes(len_bytes) as usize;
        if len > MAX_FRAME_SIZE {
            return Err(eyre::eyre!("Message larger then message limit"));
        }

        let mut buffer = vec![0u8; len];
        self.reader.read_exact(&mut buffer).await?;
        Ok(buffer)
    }
}
pub struct FramedWriter {
    writer: WriteHalf<TcpStream>,
}
impl FramedWriter {
    pub fn new(writer: WriteHalf<TcpStream>) -> Self {
        FramedWriter { writer }
    }

    pub async fn write_frame(&mut self, data: &[u8]) -> tokio::io::Result<()> {
        let len = (data.len() as u32).to_be_bytes();
        let mut buf = Vec::with_capacity(4 + data.len());
        buf.extend_from_slice(&len);
        buf.extend_from_slice(data);
        self.writer.write_all(&buf).await?;
        self.writer.flush().await?;
        Ok(())
    }
}
pub struct EncryptedReader {
    inner: FramedReader,
    cipher: TransportCipher,
}
impl EncryptedReader {
    pub fn new(inner: FramedReader, cipher: TransportCipher) -> Self {
        EncryptedReader { inner, cipher }
    }

    pub async fn read_frame(&mut self) -> eyre::Result<Vec<u8>> {
        let ciphertext = self.inner.read_frame().await?;
        let plaintext = self.cipher.decrypt(&ciphertext)?;
        Ok(plaintext)
    }
}

pub struct EncryptedWriter {
    inner: FramedWriter,
    cipher: TransportCipher,
}
impl EncryptedWriter {
    pub fn new(inner: FramedWriter, cipher: TransportCipher) -> Self {
        EncryptedWriter { inner, cipher }
    }

    pub async fn write_frame(&mut self, data: &[u8]) -> eyre::Result<()> {
        let ciphertext = self.cipher.encrypt(data);
        self.inner.write_frame(&ciphertext).await?;
        Ok(())
    }
}

use crate::p2p::transport_cipher::TransportCipher;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
