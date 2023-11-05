use std::fmt::{self, Formatter};
use std::str::FromStr;
use std::fs::File;
use std::io::{Cursor, Read};


#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    File,
    Image,
    Text,
}


/// `Command` represent all the available commands over known by the client.
/// For better developer experience is included empty command, so empty lines might be simply
/// ignored.
#[derive(PartialEq, Eq)]
pub enum Command {
    Empty,
    Quit,
    Text{text: String},
    File{path: String, content: Vec<u8>},
    Image{path: String, content: Vec<u8>},
}


impl fmt::Display for Command {
    /// `fmt` enable conversion from `Comment` to `&str` via `to_string` method.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let key = match self {
            Command::Empty => "",
            Command::Quit => "",
            Command::Text {..} => "Text",
            Command::Image {..} => "Image",
            Command::File {..} => "File",
        };

        write!(f, "{}", key)
    }
}


impl FromStr for Command {
    type Err = String;

    /// `from_str` implement conversion from `&str` to `Command` that might fail on invalid text.
    fn from_str(line: &str) -> std::result::Result<Self, Self::Err> {
        let mut parts = line
            .trim()
            .splitn(2, char::is_whitespace)
            .map(str::to_string);

        let first = match parts.next() {
            None => return Ok(Command::Empty),
            Some(text) if text.is_empty() => return Ok(Command::Empty),
            Some(text) => text,
        };

        let mut command = match first.as_str() {
            ".file" => Command::File {path: String::new(), content: vec![]},
            ".image" => Command::Image {path: String::new(), content: vec![]},
            ".quit" => return Ok(Command::Quit),
            _ => return Ok(Command::Text {text: line.trim().to_owned()}),
        };

        let filepath = match parts.next() {
            None => return Err(format!("missing path argument")),
            Some(path) => path,
        };

        let mut content = match &mut command {
            Command::File {content, path} => {
                *path = filepath.clone();
                content
            },
            Command::Image {content, path} => {
                *path = filepath.clone();
                content
            },
            _ => return Err(format!("internal error: content of invalid command")),
        };

        match File::open(filepath) {
            Ok(mut f) => match f.read_to_end(&mut content) {
                Ok(_) => {},
                Err(err) => return Err(err.to_string()),
            },
            Err(err) => return Err(err.to_string()),
        };

        if let Command::Image{content, ..} = &mut command {
            check_image(content)?;
        }

        return Ok(command)
    }
}


/// `check_image` implement transparent conversion of any possible (tested just with jpeg format)
/// image file format into the PNG file format.
fn check_image(content: &mut Vec<u8>) -> Result<(), String> {
    use image::io::Reader;

    let reader = match Reader::new(Cursor::new(&content))
        .with_guessed_format()
        .expect("Cursor I/O never fails")
        .decode() {
        Ok(reader) => reader,
        Err(err) => Err(format!("failed to decode image format: {}", err.to_string()))?,
    };

    match reader.write_to(&mut Cursor::new(content), image::ImageOutputFormat::Png) {
        Ok(_) => {},
        Err(err) => Err(format!("failed to convert image into PNG format: {}", err.to_string()))?,
    }

    Ok(())
}


impl Command {
    /// `extract` is used for extraction of data of command into 3-tuple to be passed via channel.
    pub fn extract(self) -> (MessageType, Option<String>, Option<Vec<u8>>) {
        match self {
            Command::Text {text} =>
                (MessageType::Text, Some(text), None),

            Command::File {path, content} =>
                (MessageType::File, Some(path), Some(content)),

            Command::Image {path, content} =>
                (MessageType::Image, Some(path), Some(content)),

            Command::Quit | Command::Empty =>
                (MessageType::Text, None, None),
        }
    }
}
