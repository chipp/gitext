// rewriten to Rust from https://github.com/kballard/go-shellquote

use std::fmt;
use std::str::Chars;

pub fn split<'i>(string: &'i str) -> Split<'i> {
    Split {
        inner: string.chars(),
    }
}

pub struct Split<'i> {
    inner: Chars<'i>,
}

impl<'i> Iterator for Split<'i> {
    type Item = Result<String, SplitError>;

    fn next(&mut self) -> Option<Result<String, SplitError>> {
        while let Some(chr) = self.inner.next() {
            if SPLIT_CHARS.contains(&chr) {
                continue;
            } else if chr == '\\' {
                if let Some('\n') = self.inner.next() {
                    continue;
                } else {
                    return Some(Err(SplitError::UnterminatedEscape));
                }
            }

            return Some(split_word(chr, &mut self.inner));
        }

        None
    }
}

#[derive(Debug, PartialEq)]
pub enum SplitError {
    UnterminatedEscape,
    UnterminatedSingleQuote,
    UnterminatedDoubleQuote,
}

impl fmt::Display for SplitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SplitError::UnterminatedEscape => f.write_str("unterminated escape"),
            SplitError::UnterminatedSingleQuote => f.write_str("unterminated single quote"),
            SplitError::UnterminatedDoubleQuote => f.write_str("unterminated double quote"),
        }
    }
}

impl std::error::Error for SplitError {}

const SPLIT_CHARS: &[char] = &[' ', '\n', '\t'];
const DOUBLE_ESCAPE_CHARS: &[char] = &['$', '`', '\"', '\n', '\\'];

fn split_word<'i>(chr: char, chars: &'i mut Chars) -> Result<String, SplitError> {
    let mut buffer = String::new();
    handle_chr(chr, chars, &mut buffer)?;

    while let Some(chr) = chars.next() {
        let stop = handle_chr(chr, chars, &mut buffer)?;

        if stop {
            break;
        }
    }

    Ok(buffer)
}

fn handle_chr(chr: char, chars: &mut Chars, buffer: &mut String) -> Result<bool, SplitError> {
    if chr == '\\' {
        if let Some(chr) = handle_escape(chars)? {
            buffer.push(chr);
        }
        return Ok(false);
    }

    if chr == '\'' {
        handle_single_quote(chars, buffer)?;
        return Ok(false);
    }

    if chr == '\"' {
        handle_double_quote(chars, buffer)?;
        return Ok(false);
    }

    if SPLIT_CHARS.contains(&chr) {
        return Ok(true);
    }

    buffer.push(chr);

    Ok(false)
}

fn handle_escape(chars: &mut Chars) -> Result<Option<char>, SplitError> {
    let chr = chars.next().ok_or(SplitError::UnterminatedEscape)?;

    if chr == '\n' {
        Ok(None)
    } else {
        Ok(Some(chr))
    }
}

fn handle_single_quote(chars: &mut Chars, buffer: &mut String) -> Result<(), SplitError> {
    while let Some(chr) = chars.next() {
        if chr == '\'' {
            return Ok(());
        }

        buffer.push(chr);
    }

    Err(SplitError::UnterminatedSingleQuote)
}

fn handle_double_quote(chars: &mut Chars, buffer: &mut String) -> Result<(), SplitError> {
    while let Some(chr) = chars.next() {
        if chr == '\"' {
            return Ok(());
        } else if chr == '\\' {
            if let Some(chr) = chars.next() {
                if DOUBLE_ESCAPE_CHARS.contains(&chr) {
                    if chr != '\n' {
                        buffer.push(chr);
                    } else {
                        continue;
                    }
                } else {
                    buffer.push('\\');
                    buffer.push(chr);
                }

                continue;
            }
        }

        buffer.push(chr);
    }

    Err(SplitError::UnterminatedDoubleQuote)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUCCESS_TESTS: &[(&str, &[&str])] = &[
        ("hello", &["hello"]),
        ("hello goodbye", &["hello", "goodbye"]),
        ("hello   goodbye", &["hello", "goodbye"]),
        ("glob* test?", &["glob*", "test?"]),
        (
            "don\\'t you know the dewey decimal system\\?",
            &["don't", "you", "know", "the", "dewey", "decimal", "system?"],
        ),
        (
            "'don'\\''t you know the dewey decimal system?'",
            &["don't you know the dewey decimal system?"],
        ),
        ("one '' two", &["one", "", "two"]),
        (
            "text with\\\na backslash-escaped newline",
            &["text", "witha", "backslash-escaped", "newline"],
        ),
        (
            "text \"with\na\" quoted newline",
            &["text", "with\na", "quoted", "newline"],
        ),
        (
            "\"quoted\\d\\\\\\\" text with\\\na backslash-escaped newline\"",
            &["quoted\\d\\\" text witha backslash-escaped newline"],
        ),
        (
            "text with an escaped \\\n newline in the middle",
            &[
                "text", "with", "an", "escaped", "newline", "in", "the", "middle",
            ],
        ),
        ("foo\"bar\"baz", &["foobarbaz"]),
    ];

    fn try_collect<I, V, E>(iterator: I) -> Result<Vec<V>, E>
    where
        I: Iterator<Item = Result<V, E>>,
    {
        let mut output = vec![];
        for result in iterator {
            output.push(result?);
        }
        Ok(output)
    }

    #[test]
    fn test_success() {
        for (input, expected) in SUCCESS_TESTS {
            let output = try_collect(split(input)).unwrap();
            assert_eq!(&output, expected);
        }
    }

    const FAILURE_TESTS: &[(&str, SplitError)] = &[
        ("don't worry", SplitError::UnterminatedSingleQuote),
        ("'test'\\''ing", SplitError::UnterminatedSingleQuote),
        ("\"foo'bar", SplitError::UnterminatedDoubleQuote),
        ("foo\\", SplitError::UnterminatedEscape),
        ("   \\", SplitError::UnterminatedEscape),
    ];

    #[test]
    fn test_failure() {
        for (input, expected) in FAILURE_TESTS {
            let error = try_collect(split(input)).unwrap_err();
            assert_eq!(&error, expected);
        }
    }
}
