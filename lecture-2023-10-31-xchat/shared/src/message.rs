use std::io;

use color_eyre::eyre::{bail, Result};
use serde::{Serialize, Deserialize};
use serde_cbor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream};
use tokio::time::{Duration, timeout};



/// `Message` is a type representing all messages that might be transferred between server and
/// client via TCP stream.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Message {
    /// Login message (client -> server).
    Login{
        login: String,
        pass: String,
    },

    /// Welcome message (server -> client).
    Welcome{
        motd: String,
    },

    /// Simple text message (server <-> client).
    Text(String),

    /// Content of image file (server <-> client).
    Image(Vec<u8>),

    /// General file to be transferred (server <-> client).
    File{
        filename: String,
        payload: Vec<u8>,
    }
}


impl Message {
    /// `serialize` take care of serialization process into [CBOR](https://cbor.io/) binary format
    /// using [serde](https://serde.rs/).
    pub fn serialize(&self) -> serde_cbor::Result<Vec<u8>> {
        serde_cbor::to_vec(&self)
    }

    /// `deserialize` is counterpart to the [Message::serialize] function.
    pub fn deserialize(payload: &[u8]) -> serde_cbor::Result<Message> {
        serde_cbor::from_slice(&payload)
    }

    /// `send` the message (_self_) via the given TCP `stream`.
    pub async fn send(&self, stream: &mut TcpStream) -> Result<()> {
        let serialized = self.serialize()?;
        let length = serialized.len() as u32;

        stream.write(&length.to_be_bytes()).await?;
        stream.write_all(&serialized).await?;

        Ok(())
    }

    /// `receive` try to receive a message from the given TCP `stream` in a non-blocking manner.
    ///
    /// Receiving is based on accepting first 4-byte unsigned integer (in Big Endian coding)
    /// that denotes number of bytes used by the follow-up [CBOR](https://cbor.io/) encoded message.
    ///
    /// Detection of [std::io::ErrorKind::WouldBlock] error kind is used to detect no message yet
    /// to be received. In this case is returned `Ok(None)` to denote it clearly.
    ///
    /// If zero bytes are received, it means that the peer was disconnected, therefore it is being
    /// translated into [std::io::ErrorKind::UnexpectedEof] error to have correctly handled
    /// client disconnection on the server side.
    ///
    /// Message is implicitly deserialized from CBOR representation using [Message::deserialize].
    pub async fn receive(stream: &mut TcpStream) -> Result<Option<Message>> {
        let mut length_bytes = [0u8; 4];
        match stream.try_read(&mut length_bytes) {
            Ok(0) => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "received 0 bytes"))?,
            Ok(_) => {},
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                return Ok(None);
            }
            Err(err) => bail!(err),
        }
        let length = u32::from_be_bytes(length_bytes) as usize;

        let mut message_bytes = vec![0u8; length];
        stream.read_exact(&mut message_bytes).await?;

        let message = Message::deserialize(&message_bytes)?;
        Ok(Some(message))
    }

    /// `receive_with_timeout` try to receive a response blocking for the given `duration`
    /// from the given TCP `stream`.
    ///
    /// The timeout is realized using `tokio::time::timeout` function awaiting for receiving.
    /// To detect that timeout happened check return value for `Ok(None)`.
    ///
    /// See [Message::receive] for details as they are very similar.
    pub async fn receive_with_timeout(
        stream: &mut TcpStream,
        duration: Duration,
    ) -> Result<Option<Message>> {
        let mut length_bytes = [0u8; 4];

        match timeout(duration, stream.read_exact(&mut length_bytes)).await? {
            Ok(0) => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "received 0 bytes"))?,
            Ok(_) => {},
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                return Ok(None);
            }
            Err(err) => bail!(err),
        };
        let length = u32::from_be_bytes(length_bytes) as usize;

        let mut message_bytes = vec![0u8; length];
        let result = timeout(Duration::from_secs(1), stream.read_exact(&mut message_bytes)).await?;
        if let Err(err) = result {
            bail!(err.to_string());
        };

        let message = Message::deserialize(&message_bytes)?;
        Ok(Some(message))
    }
}


#[cfg(test)]
mod tests {
    use super::Message;


    #[test]
    fn test_serialization_of_text() {
        let sample_text: Message = Message::Text("ahojky".to_string());
        let expected: Vec<u8> = vec![
            0xa1,                           // map(1)
            0x64,                             // text(4) (key)
            0x54, 0x65, 0x78, 0x74,             // "Text"
            0x66,                             // text(6) (value)
            0x61, 0x68, 0x6f, 0x6a, 0x6b, 0x79  // "ahojky"
        ];

        let encoded = sample_text.serialize();
        assert!(encoded.as_ref().is_ok());
        assert_eq!(encoded.as_ref().unwrap(), &expected);

        let decoded = Message::deserialize(&encoded.unwrap()[..]);
        assert!(decoded.as_ref().is_ok());
        assert_eq!(decoded.unwrap(), sample_text);
    }


    #[test]
    fn test_serialization_of_image() {
        let sample_image: Message = Message::Image(vec![0, 1]);
        let expected: Vec<u8> = vec![
            0xa1,                           // map(1)
            0x65,                             // text(5) (key)
            0x49, 0x6d, 0x61, 0x67, 0x65,       // "Image"
            0x82,                             // array(2) (value)
            0x0,                                // unsigned(0)
            0x1,                                // unsigned(1)
        ];

        let encoded = sample_image.serialize();
        assert!(encoded.as_ref().is_ok());
        assert_eq!(encoded.as_ref().unwrap(), &expected);

        let decoded = Message::deserialize(&encoded.unwrap()[..]);
        assert!(decoded.as_ref().is_ok());
        assert_eq!(decoded.unwrap(), sample_image);
    }


    #[test]
    fn test_serialization_of_file() {
        let sample_file: Message = Message::File{
            filename: "file.txt".to_string(),
            payload: "file content".as_bytes().to_vec(),
        };
        let expected: Vec<u8> = vec![
            0xa1,                                           // map(1)
            0x64,                                             // text(4)
            0x46, 0x69, 0x6c, 0x65,                             // "File"
            0xa2,                                             // map(2)
            0x68,                                               // text(8)
            0x66, 0x69, 0x6c, 0x65, 0x6e, 0x61, 0x6d, 0x65,       // "filename" (key)
            0x68,                                               // text(8)
            0x66, 0x69, 0x6c, 0x65, 0x2e, 0x74, 0x78, 0x74,       // "file.txt" (value)
            0x67,                                               // text(8)
            0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64,             // "payload"
            0x8c,                                               // array(12)
            0x18, 0x66,                                           // unsigned(102) ('f')
            0x18, 0x69,                                           // unsigned(105) ('i')
            0x18, 0x6c,                                           // unsigned(108) ('l')
            0x18, 0x65,                                           // unsigned(101) ('e')
            0x18, 0x20,                                           // unsigned(32)  (' ')
            0x18, 0x63,                                           // unsigned(99)  ('c')
            0x18, 0x6f,                                           // unsigned(111) ('o')
            0x18, 0x6e,                                           // unsigned(110) ('n')
            0x18, 0x74,                                           // unsigned(116) ('t')
            0x18, 0x65,                                           // unsigned(101) ('e')
            0x18, 0x6e,                                           // unsigned(110) ('n')
            0x18, 0x74,                                           // unsigned(116) ('t')
        ];

        let encoded = sample_file.serialize();
        assert!(encoded.as_ref().is_ok());
        assert_eq!(encoded.as_ref().unwrap(), &expected);

        let decoded = Message::deserialize(&encoded.unwrap()[..]);
        assert!(decoded.as_ref().is_ok());
        assert_eq!(decoded.unwrap(), sample_file);
    }
}

