// trying to emulate: https://github.com/venthur/dotenv-cli/blob/master/dotenv_cli/core.py
use std::{io::BufRead, path::Path};

use crate::{Env, Error, Options, Result, DEBUG_PREFIX};

pub fn config_python_dotenv_cli(reader: &mut dyn BufRead, env: &mut dyn Env, options: &Options<&Path>) -> Result<()> {
    let path_str = options.path.to_string_lossy();

    let mut lines = String::new();
    options.encoding.read_to_string(reader, &mut lines)?;

    let mut lines = &lines[..];
    let mut value_buf = String::new();
    let mut lineno = 0usize;
    loop {
        lineno += 1;
        let mut line;
        // split on "\n", "\r\n", and "\r"
        if let Some(index) = lines.find(|p| p == '\r' || p == '\n') {
            line = &lines[..index];
            lines = &lines[index..];
            if lines.starts_with("\r\n") {
                lines = &lines[2..];
            } else {
                lines = &lines[1..];
            }
        } else {
            line = lines;
            if line.is_empty() {
                break;
            }
        }

        line = line.trim();

        // ignore comments
        if line.starts_with('#') {
            continue;
        }

        let Some(equals) = line.find('=') else {
            // ignore empty lines or lines w/o '='
            continue;
        };

        let mut key = &line[..equals];
        let mut value = &line[equals + 1..];

        // allow export
        if key.starts_with("export ") {
            key = &key[7..];
        }

        key = key.trim();
        value = value.trim();

        // remove quotes (not sure if this is standard behaviour)
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            value = &value[1..value.len() - 1];

            // decode escape characters
            value_buf.clear();
            while !value.is_empty() {
                let Some(index) = value.find('\\') else {
                    value_buf.push_str(value);
                    break;
                };

                value_buf.push_str(&value[..index]);
                value = &value[index + 1..];
                let Some(ch) = value.chars().next() else {
                    if options.debug {
                        eprintln!("{DEBUG_PREFIX}{path_str}:{lineno}: truncated escape sequence");
                    }
                    if options.strict {
                        return Err(Error::syntax_error(lineno, 1));
                    }
                    break;
                };

                match ch {
                    '\n' => {
                        value = &value[1..];
                    }
                    '\\' | '\'' | '"' => {
                        value_buf.push(ch);
                        value = &value[1..];
                    }
                    'a' => {
                        value_buf.push('\x07');
                        value = &value[1..];
                    }
                    'b' => {
                        value_buf.push('\x08');
                        value = &value[1..];
                    }
                    'f' => {
                        value_buf.push('\x0c');
                        value = &value[1..];
                    }
                    'n' => {
                        value_buf.push('\n');
                        value = &value[1..];
                    }
                    'r' => {
                        value_buf.push('\r');
                        value = &value[1..];
                    }
                    't' => {
                        value_buf.push('\t');
                        value = &value[1..];
                    }
                    'v' => {
                        value_buf.push('\x0b');
                        value = &value[1..];
                    }
                    '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7' => {
                        let mut end_index = 1;
                        if value.len() > 1 && value[1..].starts_with(|ch| matches!(ch, '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7')) {
                            end_index += 1;
                            if value.len() > 2 && value[2..].starts_with(|ch| matches!(ch, '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7')) {
                                end_index += 1;
                            }
                        }

                        let arg = &value[..end_index];
                        let Ok(ch) = u8::from_str_radix(arg, 8) else {
                            if options.debug {
                                eprintln!("{DEBUG_PREFIX}{path_str}:{lineno}: invalid octal escape sequence: \\{}", arg);
                            }
                            if options.strict {
                                return Err(Error::syntax_error(lineno, 1));
                            }
                            value_buf.push('\\');
                            continue;
                        };
                        value_buf.push(ch as char);
                        value = &value[end_index..];
                    }
                    'x' => {
                        if value.len() < 3 {
                            if options.debug {
                                eprintln!("{DEBUG_PREFIX}{path_str}:{lineno}: invalid hex escape sequence: \\{}", value);
                            }
                            if options.strict {
                                return Err(Error::syntax_error(lineno, 1));
                            }
                            value_buf.push('\\');
                            continue;
                        }

                        let arg = &value[1..3];
                        let Ok(ch) = u8::from_str_radix(arg, 16) else {
                            if options.debug {
                                eprintln!("{DEBUG_PREFIX}{path_str}:{lineno}: invalid hex escape sequence: \\x{}", arg);
                            }
                            if options.strict {
                                return Err(Error::syntax_error(lineno, 1));
                            }
                            value_buf.push('\\');
                            continue;
                        };
                        value_buf.push(ch as char);
                        value = &value[3..];
                    }
                    'u' => {
                        if value.len() < 5 {
                            if options.debug {
                                eprintln!("{DEBUG_PREFIX}{path_str}:{lineno}: invalid unicode escape sequence: \\{}", value);
                            }
                            if options.strict {
                                return Err(Error::syntax_error(lineno, 1));
                            }
                            value_buf.push('\\');
                            continue;
                        }

                        let arg = &value[1..5];
                        let Ok(ch) = u16::from_str_radix(arg, 16) else {
                            if options.debug {
                                eprintln!("{DEBUG_PREFIX}{path_str}:{lineno}: invalid unicode escape sequence: \\u{}", arg);
                            }
                            if options.strict {
                                return Err(Error::syntax_error(lineno, 1));
                            }
                            value_buf.push('\\');
                            continue;
                        };
                        let Some(ch) = char::from_u32(ch.into()) else {
                            if options.debug {
                                eprintln!("{DEBUG_PREFIX}{path_str}:{lineno}: invalid unicode escape sequence: \\u{}", arg);
                            }
                            if options.strict {
                                return Err(Error::syntax_error(lineno, 1));
                            }
                            value_buf.push('\\');
                            continue;
                        };
                        value_buf.push(ch);
                        value = &value[5..];
                    }
                    'U' => {
                        if value.len() < 9 {
                            if options.debug {
                                eprintln!("{DEBUG_PREFIX}{path_str}:{lineno}: invalid unicode escape sequence: \\{}", value);
                            }
                            if options.strict {
                                return Err(Error::syntax_error(lineno, 1));
                            }
                            value_buf.push('\\');
                            continue;
                        }

                        let arg = &value[1..9];
                        let Ok(ch) = u32::from_str_radix(arg, 16) else {
                            if options.debug {
                                eprintln!("{DEBUG_PREFIX}{path_str}:{lineno}: invalid unicode escape sequence: \\U{}", arg);
                            }
                            if options.strict {
                                return Err(Error::syntax_error(lineno, 1));
                            }
                            value_buf.push('\\');
                            continue;
                        };
                        let Some(ch) = char::from_u32(ch) else {
                            if options.debug {
                                eprintln!("{DEBUG_PREFIX}{path_str}:{lineno}: invalid unicode escape sequence: \\U{}", arg);
                            }
                            if options.strict {
                                return Err(Error::syntax_error(lineno, 1));
                            }
                            value_buf.push('\\');
                            continue;
                        };
                        value_buf.push(ch);
                        value = &value[9..];
                    }
                    _ => {
                        if options.debug {
                            eprintln!("{DEBUG_PREFIX}{path_str}:{lineno}: invalid escape sequence: \\{}", ch);
                        }
                        if options.strict {
                            return Err(Error::syntax_error(lineno, 1));
                        }
                        value_buf.push('\\');
                        continue;
                    }
                }
            }
            value = &value_buf[..];
        } else if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
            value = &value[1..value.len() - 1];
        }

        options.set_var_cut_null(env, key, value);
    }

    Ok(())
}
