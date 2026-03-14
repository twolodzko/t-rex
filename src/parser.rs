use super::types::{Boundary, Character, is_special};
use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone)]
pub(super) enum Token {
    /// $, ^
    Special(Boundary),
    /// ., \w, a, b, [a-z], ...
    Matcher(Character),
    /// abc[a-z]+
    Branch(Vec<Token>),
    /// abc|def
    Alternative(Vec<Token>),
    /// a+, a?, a*, a{n}, a{n,m}
    Repeat(Box<Token>, usize, Option<usize>),
}

pub(super) fn parse(regex: &str) -> Result<Token, ParsingError> {
    Parser::from(regex).regex()
}

struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> From<&'a str> for Parser<'a> {
    fn from(regex: &'a str) -> Self {
        Parser {
            chars: regex.chars().peekable(),
        }
    }
}

impl<'a> Parser<'a> {
    fn regex(&mut self) -> Result<Token, ParsingError> {
        self.alternative()
    }

    fn alternative(&mut self) -> Result<Token, ParsingError> {
        use Token::*;
        let mut acc = Vec::new();
        while self.chars.peek().is_some() {
            let branch = self.branch()?;
            acc.push(branch);
            if let Some('|') = self.chars.peek() {
                self.chars.next();
            } else {
                break;
            }
        }
        let token = match acc.len() {
            // empty branch makes more sense than empty alternative
            // also, less edge cases
            0 => Branch(acc),
            1 => acc.pop().unwrap(),
            _ => Alternative(acc),
        };
        Ok(token)
    }

    fn branch(&mut self) -> Result<Token, ParsingError> {
        use Token::*;
        let mut acc = Vec::new();
        while let Some(c) = self.chars.peek() {
            if *c == '|' || *c == ')' {
                break;
            }
            if let Some(res) = self.atom() {
                let atom = res?;
                acc.push(atom);
            }
        }
        let token = if acc.len() == 1 {
            acc.pop().unwrap()
        } else {
            Branch(acc)
        };
        Ok(token)
    }

    fn atom(&mut self) -> Option<Result<Token, ParsingError>> {
        use Character::*;
        use Token::*;
        let mut atom = match self.chars.next()? {
            '.' => Matcher(Anything),
            '^' => Special(Boundary::Start),
            '$' => Special(Boundary::End),
            '\\' => {
                if let Some(c) = self.chars.peek() {
                    // in POSIX only those can be escaped: ^.[$()|*+?{\
                    // plus the special characters like \n or \b
                    match c {
                        'b' => {
                            self.chars.next();
                            Special(Boundary::Word(true))
                        }
                        'B' => {
                            self.chars.next();
                            Special(Boundary::Word(false))
                        }
                        _ => {
                            // the first character after \ cannot be consumed at this point
                            let c = match self.escaped_character() {
                                Ok(c) => c,
                                Err(err) => return Some(Err(err)),
                            };
                            Matcher(c)
                        }
                    }
                } else {
                    // \ can't be the last character
                    return Some(Err(ParsingError::EndOfInput));
                }
            }
            '(' => {
                if let Some('?') = self.chars.peek() {
                    self.chars.next();
                    let Some(':') = self.chars.next() else {
                        return Some(Err(ParsingError::Unexpected('?')));
                    };
                }
                let branch = match self.alternative() {
                    Ok(b) => b,
                    Err(err) => return Some(Err(err)),
                };
                let Some(')') = self.chars.next() else {
                    return Some(Err(ParsingError::Missing(')')));
                };
                branch
            }
            '[' => match self.set() {
                Ok(s) => Matcher(s),
                Err(err) => return Some(Err(err)),
            },
            c @ ('|' | '?' | '*' | '+' | '{') => {
                return Some(Err(ParsingError::Unexpected(c)));
            }
            c => Matcher(Literal(c)),
        };
        if let Some(res) = self.reps() {
            match res {
                Ok((n, m)) => {
                    if m == Some(0) {
                        // a{0} or a{0,0} means skip
                        return None;
                    }
                    atom = Repeat(Box::new(atom), n, m);
                }
                Err(err) => return Some(Err(err)),
            };
        };
        Some(Ok(atom))
    }

    fn reps(&mut self) -> Option<Result<(usize, Option<usize>), ParsingError>> {
        let reps = match self.chars.peek()? {
            '?' => (0, Some(1)),
            '*' => (0, None),
            '+' => (1, None),
            '{' => {
                self.chars.next();
                let Some(n) = self.number() else {
                    return Some(Err(ParsingError::InvalidRepetitions));
                };
                let Some(c) = self.chars.peek() else {
                    return Some(Err(ParsingError::EndOfInput));
                };
                // TODO: handle whitespaces
                let m = match c {
                    ',' => {
                        self.chars.next();
                        if let Some('}') = self.chars.peek() {
                            None
                        } else {
                            let m = self.number();
                            let Some('}') = self.chars.peek() else {
                                return Some(Err(ParsingError::Missing('}')));
                            };
                            m
                        }
                    }
                    '}' => Some(n),
                    _ => return Some(Err(ParsingError::InvalidRepetitions)),
                };
                if let Some(m) = m
                    && (n > m || m == 0 || m > 100_000)
                {
                    return Some(Err(ParsingError::InvalidRepetitions));
                }
                (n, m)
            }
            _ => return None,
        };
        self.chars.next();
        Some(Ok(reps))
    }

    fn set(&mut self) -> Result<Character, ParsingError> {
        use Character::*;
        let mut acc = Vec::new();
        let negate = if let Some('^') = self.chars.peek() {
            self.chars.next();
            true
        } else {
            false
        };
        if let Some(c @ ('-' | ']')) = self.chars.peek() {
            acc.push(Literal(*c));
            self.chars.next();
        }
        loop {
            let Some(c) = self.chars.next() else {
                return Err(ParsingError::EndOfInput);
            };
            match c {
                '\\' => {
                    let c = self.escaped_character()?;
                    acc.push(c);
                }
                ']' => break,
                '-' => {
                    let Some(end) = self.chars.next() else {
                        return Err(ParsingError::EndOfInput);
                    };
                    match end {
                        ']' => {
                            acc.push(Literal(c));
                            break;
                        }
                        _ => {
                            if let Some(t) = acc.pop() {
                                let Literal(start) = t else { unreachable!() };
                                if start >= end {
                                    return Err(ParsingError::InvalidRange(start, end));
                                }
                                acc.push(Range(start, end));
                            } else {
                                acc.push(Literal('-'));
                                acc.push(Literal(c));
                            }
                        }
                    }
                }
                _ => acc.push(Literal(c)),
            }
        }
        let token = if negate { !Set(acc) } else { Set(acc) };
        Ok(token)
    }

    fn escaped_character(&mut self) -> Result<Character, ParsingError> {
        use Character::*;
        let Some(c) = self.chars.next() else {
            return Err(ParsingError::EndOfInput);
        };
        let c = match c {
            'w' => return Ok(Word),
            'W' => return Ok(!Word),
            'd' => return Ok(Digit),
            'D' => return Ok(!Digit),
            's' => return Ok(Space),
            'S' => return Ok(!Space),
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '0' => '\0',
            'b' => '\u{000C}',
            // \xNN
            'x' => {
                let mut s = String::new();
                for _ in 0..2 {
                    if let Some(c) = self.chars.next() {
                        s.push(c);
                    } else {
                        return Err(ParsingError::EndOfInput);
                    };
                }
                let Ok(u) = u32::from_str_radix(&s, 16) else {
                    return Err(ParsingError::NotANumber(s));
                };
                char::from_u32(u).ok_or(ParsingError::NotANumber(s))?
            }
            // \uNNNN
            'u' => {
                let mut s = String::new();
                for _ in 0..4 {
                    if let Some(c) = self.chars.next() {
                        s.push(c);
                    } else {
                        return Err(ParsingError::EndOfInput);
                    };
                }
                let Ok(u) = u32::from_str_radix(&s, 16) else {
                    return Err(ParsingError::NotANumber(s));
                };
                char::from_u32(u).ok_or(ParsingError::NotANumber(s))?
            }
            // in particular: \'"
            '\'' | '"' => c,
            c if is_special(c) => c,
            c => return Err(ParsingError::BadEscape(c)),
        };
        Ok(Literal(c))
    }

    fn number(&mut self) -> Option<usize> {
        let mut n = self.chars.next()?.to_digit(10)?;
        while let Some(c) = self.chars.peek() {
            let Some(d) = c.to_digit(10) else {
                break;
            };
            self.chars.next();
            n = (n * 10) + d;
        }
        Some(n as usize)
    }
}

#[derive(Debug)]
pub enum ParsingError {
    EndOfInput,
    Unexpected(char),
    Missing(char),
    InvalidRange(char, char),
    InvalidRepetitions,
    NotANumber(String),
    BadEscape(char),
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParsingError::*;
        match self {
            EndOfInput => write!(f, "unexpected end of input"),
            Unexpected(c) => write!(f, "unexpected character {}", c),
            Missing(c) => write!(f, "missing {}", c),
            InvalidRange(a, b) => write!(f, "{}-{} is not a valid range", a, b),
            InvalidRepetitions => {
                write!(f, "invalid repetitions counts")
            }
            NotANumber(s) => write!(f, "{} is not a number", s),
            BadEscape(c) => write!(f, "unsupported escape character \\{}", c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use test_case::test_case;

    #[test_case(
            r"a";
            "single character"
        )]
    #[test_case(
            r"abc";
            "many characters"
        )]
    #[test_case(
            r"a+";
            "more than one character"
        )]
    #[test_case(
            r"(\n)+";
            "more than one groups"
        )]
    #[test_case(
            r"\S{7,9}";
            "repeat non space"
        )]
    #[test_case(
            r"\d{7,}";
            "repeat digit"
        )]
    #[test_case(
            r"(){42}";
            "repeat empty group"
        )]
    #[test_case(
            r"[]]?";
            "zero or one set with bracket"
        )]
    #[test_case(
            r"[--]";
            "minus minus set"
        )]
    #[test_case(
            r"[--4]";
            "minus range set"
        )]
    #[test_case(
            r"[-a-z-]";
            "range with minuses on both sides"
        )]
    #[test_case(
            r"[]-}]";
            "set of strange chars"
        )]
    #[test_case(
            r"[^][]]";
            "set of non brackets and bracket"
        )]
    #[test_case(
            r"()|()";
            "either empty set"
        )]
    #[test_case(
            r"a|b";
            "either a or b"
        )]
    #[test_case(
            r"(a|b)";
            "group of either a or b"
        )]
    fn no_errors(input: &str) {
        let res = parse(input);
        println!("{} => {:?}", input, res);
        assert!(res.is_ok());
        // assert!(false)
    }
}
