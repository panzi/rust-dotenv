use std::{ffi::OsStr, io::{BufRead, ErrorKind}};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Encoding {
    ASCII,
    /// aka ISO-8859-1
    Latin1,
    UTF8,
    UTF16BE,
    UTF16LE,
    UTF32BE,
    UTF32LE,
}

impl Encoding {
    pub(crate) fn read_line(&self, reader: &mut dyn BufRead, line: &mut String) -> std::io::Result<usize> {
        match self {
            Encoding::UTF8    => reader.read_line(line),
            Encoding::ASCII   => read_line_ascii(reader, line),
            Encoding::Latin1  => read_line_latin1(reader, line),
            Encoding::UTF16LE => read_line_utf16le(reader, line),
            Encoding::UTF16BE => read_line_utf16be(reader, line),
            Encoding::UTF32LE => read_line_utf32le(reader, line),
            Encoding::UTF32BE => read_line_utf32be(reader, line),
        }
    }

    pub(crate) fn read_to_string(&self, reader: &mut dyn BufRead, buf: &mut String) -> std::io::Result<usize> {
        match self {
            Encoding::UTF8 => reader.read_to_string(buf),
            Encoding::ASCII => {
                let mut bytes = Vec::new();
                let byte_count = reader.read_to_end(&mut bytes)?;
                if bytes.iter().cloned().any(|byte| byte > 127) {
                    return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
                }

                buf.push_str(&unsafe { String::from_utf8_unchecked(bytes) });

                Ok(byte_count)
            },
            Encoding::Latin1 => {
                let mut bytes = Vec::new();
                let byte_count = reader.read_to_end(&mut bytes)?;

                buf.reserve(byte_count);
                buf.extend(bytes.iter().cloned().map(|byte| byte as char));

                Ok(byte_count)
            },
            Encoding::UTF16BE => read_utf16(reader, buf, u16::from_be_bytes),
            Encoding::UTF16LE => read_utf16(reader, buf, u16::from_le_bytes),
            Encoding::UTF32BE => read_utf32(reader, buf, u32::from_be_bytes),
            Encoding::UTF32LE => read_utf32(reader, buf, u32::from_le_bytes),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IllegalEncoding();

impl std::fmt::Display for IllegalEncoding {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "IllegalEncoding".fmt(f)
    }
}

impl std::error::Error for IllegalEncoding {}

impl TryFrom<&OsStr> for Encoding {
    type Error = IllegalEncoding;

    fn try_from(value: &OsStr) -> std::result::Result<Self, Self::Error> {
        if value.is_empty() ||
           value.eq_ignore_ascii_case("utf-8") ||
           value.eq_ignore_ascii_case("utf8") ||
           value.eq_ignore_ascii_case("windows-65001") {
            Ok(Encoding::UTF8)
        } else if value.eq_ignore_ascii_case("ascii") ||
                  value.eq_ignore_ascii_case("us-ascii") ||
                  value.eq_ignore_ascii_case("windows-20127") {
            Ok(Encoding::ASCII)
        } else if value.eq_ignore_ascii_case("latin1") ||
                  value.eq_ignore_ascii_case("iso-8859-1") ||
                  value.eq_ignore_ascii_case("iso8859-1") ||
                  value.eq_ignore_ascii_case("iso8859_1") ||
                  value.eq_ignore_ascii_case("windows-28591") ||
                  value.eq_ignore_ascii_case("cp819") {
            Ok(Encoding::Latin1)
        } else if value.eq_ignore_ascii_case("utf-16le") ||
                  value.eq_ignore_ascii_case("utf16le") ||
                  value.eq_ignore_ascii_case("windows-1200") {
            Ok(Encoding::UTF16LE)
        } else if value.eq_ignore_ascii_case("utf-16be") ||
                  value.eq_ignore_ascii_case("utf16be") ||
                  value.eq_ignore_ascii_case("windows-1201") {
            Ok(Encoding::UTF16BE)
        } else if value.eq_ignore_ascii_case("utf-32le") ||
                  value.eq_ignore_ascii_case("utf32le") ||
                  value.eq_ignore_ascii_case("windows-12000") {
            Ok(Encoding::UTF32LE)
        } else if value.eq_ignore_ascii_case("utf-32be") ||
                  value.eq_ignore_ascii_case("utf32be") ||
                  value.eq_ignore_ascii_case("windows-12001") {
            Ok(Encoding::UTF32BE)
        } else {
            Err(IllegalEncoding())
        }
    }
}

impl std::str::FromStr for Encoding {
    type Err = IllegalEncoding;

    #[inline]
    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Encoding::try_from(value.as_ref())
    }
}

impl Default for Encoding {
    #[inline]
    fn default() -> Self {
        Encoding::UTF8
    }
}

fn read_line_ascii(reader: &mut dyn BufRead, line: &mut String) -> std::io::Result<usize> {
    let mut byte_count = 0;

    loop {
        let mut buf = [0u8; 1];
        if let Err(error) = reader.read_exact(&mut buf) {
            if error.kind() == ErrorKind::UnexpectedEof {
                break;
            }
            return Err(error);
        }
        byte_count += 1;

        let ch = buf[0];
        if ch > 127 {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }

        let ch = ch as char;

        line.push(ch);
        if ch == '\n' {
            break;
        }
    }

    Ok(byte_count)
}

fn read_line_latin1(reader: &mut dyn BufRead, line: &mut String) -> std::io::Result<usize> {
    let mut byte_count = 0;

    loop {
        let mut buf = [0u8; 1];
        if let Err(error) = reader.read_exact(&mut buf) {
            if error.kind() == ErrorKind::UnexpectedEof {
                break;
            }
            return Err(error);
        }
        byte_count += 1;

        let ch = buf[0] as char;

        line.push(ch);
        if ch == '\n' {
            break;
        }
    }

    Ok(byte_count)
}

#[inline]
fn read_line_utf16be(reader: &mut dyn BufRead, line: &mut String) -> std::io::Result<usize> {
    read_line_utf16(reader, line, u16::from_be_bytes)
}

#[inline]
fn read_line_utf16le(reader: &mut dyn BufRead, line: &mut String) -> std::io::Result<usize> {
    read_line_utf16(reader, line, u16::from_le_bytes)
}

fn read_line_utf16(reader: &mut dyn BufRead, line: &mut String, decode: fn([u8; 2]) -> u16) -> std::io::Result<usize> {
    let mut byte_count = 0;

    loop {
        let mut buf = [0u8; 2];

        if let Err(error) = reader.read_exact(&mut buf) {
            if error.kind() == ErrorKind::UnexpectedEof {
                break;
            }
            return Err(error);
        }
        byte_count += buf.len();
        let hi = decode(buf);

        if hi == ('\n' as u16) {
            line.push('\n');
            break;
        }

        if hi >= 0xDC00 /* && hi <= 0xDFFF */ {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }

        if hi >= 0xD800 && hi <= 0xDBFF {
            if let Err(error) = reader.read_exact(&mut buf) {
                if error.kind() == ErrorKind::UnexpectedEof {
                    return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
                }
                return Err(error);
            }
            byte_count += buf.len();
            let lo = decode(buf);

            if lo < 0xDC00 || lo > 0xDFFF {
                return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
            }

            let ch = (((hi & 0x3ff) as u32) << 10 | (lo & 0x3ff) as u32) + 0x1_0000;
            line.push(unsafe { char::from_u32_unchecked(ch) });
        } else {
            line.push(unsafe { char::from_u32_unchecked(hi as u32) });
        }
    }

    Ok(byte_count)
}

#[inline]
fn read_line_utf32be(reader: &mut dyn BufRead, line: &mut String) -> std::io::Result<usize> {
    read_line_utf32(reader, line, u32::from_be_bytes)
}

#[inline]
fn read_line_utf32le(reader: &mut dyn BufRead, line: &mut String) -> std::io::Result<usize> {
    read_line_utf32(reader, line, u32::from_le_bytes)
}

fn read_line_utf32(reader: &mut dyn BufRead, line: &mut String, decode: fn([u8; 4]) -> u32) -> std::io::Result<usize> {
    let mut byte_count = 0;

    loop {
        let mut buf = [0u8; 4];

        if let Err(error) = reader.read_exact(&mut buf) {
            if error.kind() == ErrorKind::UnexpectedEof {
                break;
            }
            return Err(error);
        }
        byte_count += buf.len();
        let ch = decode(buf);

        let Some(ch) = char::from_u32(ch) else {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        };
        line.push(ch);

        if ch == '\n' {
            break;
        }
    }

    Ok(byte_count)
}

fn read_utf16(reader: &mut dyn BufRead, out: &mut String, decode: fn([u8; 2]) -> u16) -> std::io::Result<usize> {
    let mut byte_count = 0;

    loop {
        let mut buf = [0u8; 2];

        if let Err(error) = reader.read_exact(&mut buf) {
            if error.kind() == ErrorKind::UnexpectedEof {
                break;
            }
            return Err(error);
        }
        byte_count += buf.len();
        let hi = decode(buf);

        if hi >= 0xDC00 /* && hi <= 0xDFFF */ {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }

        if hi >= 0xD800 && hi <= 0xDBFF {
            if let Err(error) = reader.read_exact(&mut buf) {
                if error.kind() == ErrorKind::UnexpectedEof {
                    return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
                }
                return Err(error);
            }
            byte_count += buf.len();
            let lo = decode(buf);

            if lo < 0xDC00 || lo > 0xDFFF {
                return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
            }

            let ch = (((hi & 0x3ff) as u32) << 10 | (lo & 0x3ff) as u32) + 0x1_0000;
            out.push(unsafe { char::from_u32_unchecked(ch) });
        } else {
            out.push(unsafe { char::from_u32_unchecked(hi as u32) });
        }
    }

    Ok(byte_count)
}

fn read_utf32(reader: &mut dyn BufRead, line: &mut String, decode: fn([u8; 4]) -> u32) -> std::io::Result<usize> {
    let mut byte_count = 0;

    loop {
        let mut buf = [0u8; 4];

        if let Err(error) = reader.read_exact(&mut buf) {
            if error.kind() == ErrorKind::UnexpectedEof {
                break;
            }
            return Err(error);
        }
        byte_count += buf.len();
        let ch = decode(buf);

        let Some(ch) = char::from_u32(ch) else {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        };
        line.push(ch);
    }

    Ok(byte_count)
}
