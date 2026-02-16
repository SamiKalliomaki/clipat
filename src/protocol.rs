use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};

use crate::clipboard::{Content, Image};

pub(crate) struct Connection<R: Read, W: Write> {
    reader: BufReader<R>,
    writer: BufWriter<W>,
}

impl<R: Read, W: Write> Connection<R, W> {
    pub(crate) fn new(reader: R, writer: W) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
        }
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf = [0];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_newline(&mut self) -> io::Result<()> {
        if self.read_byte()? != b'\n' {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Expected newline",
            ));
        }
        Ok(())
    }

    fn read_line(&mut self) -> io::Result<String> {
        let mut line = String::new();
        loop {
            let data = self.reader.fill_buf()?;
            if data.is_empty() {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected EOF"));
            }

            match data.iter().position(|b| *b == b'\n') {
                Some(pos) => {
                    line += std::str::from_utf8(&data[..pos])
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    self.reader.consume(pos + 1);
                    return Ok(line);
                }
                None => {
                    line += std::str::from_utf8(data)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    let len = data.len();
                    self.reader.consume(len);
                }
            }
        }
    }

    fn read_int<T: std::str::FromStr>(&mut self) -> io::Result<T>
    where
        T::Err: std::error::Error + Send + Sync + 'static,
    {
        self.read_line()?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn send_int<T: std::fmt::Display>(&mut self, int: T) -> io::Result<()> {
        self.writer.write_all(int.to_string().as_bytes())?;
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    pub(crate) fn read_string(&mut self) -> io::Result<String> {
        let len = self.read_int()?;
        let mut data = vec![0; len];
        self.reader.read_exact(&mut data)?;
        self.read_newline()?;

        String::from_utf8(data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn send_string(&mut self, string: &str) -> io::Result<()> {
        self.writer.write_all(string.len().to_string().as_bytes())?;
        self.writer.write_all(b"\n")?;
        self.writer.write_all(string.as_bytes())?;
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    pub(crate) fn read_command(&mut self) -> io::Result<String> {
        self.read_line()
    }

    pub(crate) fn send_command(&mut self, command: &str) -> io::Result<()> {
        self.writer.write_all(command.as_bytes())?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()
    }

    pub(crate) fn read_image(&mut self) -> io::Result<Image<'static>> {
        let width = self.read_int()?;
        let height = self.read_int()?;
        let mut bytes = vec![0; width * height * 4];
        self.reader.read_exact(&mut bytes)?;
        self.read_newline()?;

        Ok(Image {
            width,
            height,
            bytes: bytes.into(),
        })
    }

    pub(crate) fn send_clipboard(&mut self, content: &Option<Content<'_>>) -> io::Result<()> {
        let content = match content {
            Some(content) => content,
            None => {
                self.writer.write_all(b"COPY NONE\n")?;
                return self.writer.flush();
            }
        };

        match content {
            Content::Text(text) => {
                self.writer.write_all(b"COPY TEXT\n")?;
                self.send_string(text)?;
            }
            Content::Image(image) => {
                self.writer.write_all(b"COPY IMAGE\n")?;
                self.send_int(image.width)?;
                self.send_int(image.height)?;
                self.writer.write_all(&image.bytes)?;
                self.writer.write_all(b"\n")?;
            }
        }

        self.writer.flush()
    }

    pub(crate) fn send_error(&mut self, error: &str) -> io::Result<()> {
        self.writer.write_all(b"ERROR\n")?;
        self.send_string(error)?;
        self.writer.flush()
    }
}
