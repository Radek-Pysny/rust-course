use serde::{Serialize, Deserialize};
use serde_cbor;


#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /// Simple text message.
    Text(String),

    /// Content of image file.
    Image(Vec<u8>),

    /// General file to be transferred.
    File{
        filename: String,
        payload: Vec<u8>,
    }
}

impl Message {
    pub fn serialize(&self) -> serde_cbor::Result<Vec<u8>> {
        serde_cbor::to_vec(&self)
    }

    pub fn deserialize(payload: &[u8]) -> serde_cbor::Result<Message> {
        serde_cbor::from_slice(&payload)
    }
}

#[cfg(tests)]
mod tests {
    use super::Message;

    #[test]
    fn test_serialize_text() {
        let text = Message::Text("ahojky");
        let result = text.serialize();

        assert_eq!();
    }
}
