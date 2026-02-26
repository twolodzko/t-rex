use super::types::{Character, Instruction, Regex};
use std::{iter::Peekable, str::Chars};

pub fn parse(regex: &str) -> Result<Regex, ()> {
    let instructions = Parser::from(regex).branch(false)?;
    let regex = Regex { instructions };
    Ok(regex)
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
    fn branch(&mut self, inside_brackets: bool) -> Result<Vec<Instruction>, ()> {
        let mut acc = Vec::new();
        loop {
            let mut atom = self.atom()?;
            acc.append(&mut atom);
            if let Some(c) = self.chars.peek() {
                match c {
                    '|' => {
                        self.chars.next();
                        acc.insert(0, Instruction::Split(acc.len() + 2));
                        // recurse
                        let mut tail = self.branch(inside_brackets)?;
                        acc.push(Instruction::Skip(tail.len() + 1));
                        acc.append(&mut tail);
                        break;
                    }
                    ')' if inside_brackets => {
                        break;
                    }
                    _ => (),
                }
            } else {
                break;
            }
        }
        Ok(acc)
    }

    fn atom(&mut self) -> Result<Vec<Instruction>, ()> {
        use Character::*;
        use Instruction::*;
        let mut acc = Vec::new();
        if let Some(c) = self.chars.next() {
            match c {
                '.' => acc.push(Is(Anything)),
                '^' => acc.push(Start),
                '$' => acc.push(End),
                '\\' => {
                    if let Some(c) = self.chars.peek() {
                        // in POSIX only those can be escaped: ^.[$()|*+?{\
                        // plus the special characters like \n or \b
                        let instruction = match c {
                            'b' => Boundary(true),
                            'B' => Boundary(false),
                            's' => Is(Space(true)),
                            'S' => Is(Space(false)),
                            'd' => Is(Digit(true)),
                            'D' => Is(Digit(false)),
                            'w' => Is(Word(true)),
                            'W' => Is(Word(false)),
                            // matching groups
                            '1'..'9' => {
                                let id = self.number().unwrap();
                                acc.push(Group(id));
                                return Ok(acc);
                            }
                            _ => {
                                // the first character after \ cannot be consumed at this point
                                let c = self.escaped_character()?;
                                acc.push(Is(Literal(c)));
                                return Ok(acc);
                            }
                        };
                        self.chars.next();
                        acc.push(instruction)
                    } else {
                        // \ can't be the last character
                        return Err(());
                    }
                }
                '(' => {
                    let mut capturing = true;
                    if let Some('?') = self.chars.peek() {
                        self.chars.next();
                        match self.chars.next() {
                            Some(':') => capturing = true,
                            Some('R') => {
                                let Some(')') = self.chars.next() else {
                                    return Err(());
                                };
                                acc.push(Instruction::Recurse);
                                if let Some(res) = self.reps() {
                                    let (n, m) = res?;
                                    repeat(&mut acc, n, m);
                                }
                                return Ok(acc);
                            }
                            _ => {
                                // could also be a modifier in many regex flavors
                                return Err(());
                            }
                        };
                    }
                    if capturing {
                        acc.push(Instruction::GroupStart);
                    }
                    let mut branch = self.branch(true)?;
                    acc.append(&mut branch);
                    if let Some(')') = self.chars.next() {
                        if capturing {
                            acc.push(Instruction::GroupEnd);
                        }
                    } else {
                        return Err(());
                    };
                }
                '[' => acc.push(Is(self.set()?)),
                '|' | '?' | '*' | '+' | '{' => unreachable!(),
                c => acc.push(Is(Literal(c))),
            };
        }
        if let Some(res) = self.reps() {
            let (n, m) = res?;
            repeat(&mut acc, n, m);
        }
        Ok(acc)
    }

    fn reps(&mut self) -> Option<Result<(usize, usize), ()>> {
        let reps = match self.chars.peek()? {
            '?' => (0, 1),
            '*' => (0, usize::MAX),
            '+' => (1, usize::MAX),
            '{' => {
                self.chars.next();
                let n = self.number()?;
                let Some(c) = self.chars.peek() else {
                    return Some(Err(()));
                };
                // TODO: handle whitespaces
                let m = match c {
                    ',' => {
                        self.chars.next();
                        if let Some('}') = self.chars.peek() {
                            usize::MAX
                        } else {
                            let m = self.number().unwrap_or(usize::MAX);
                            let Some('}') = self.chars.peek() else {
                                return Some(Err(()));
                            };
                            m
                        }
                    }
                    '}' => n,
                    _ => return Some(Err(())),
                };
                if n > m {
                    return Some(Err(()));
                }
                (n, m)
            }
            _ => return None,
        };
        self.chars.next();
        Some(Ok(reps))
    }

    fn set(&mut self) -> Result<Character, ()> {
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
                return Err(());
            };
            match c {
                '\\' => {
                    let c = self.escaped_character()?;
                    acc.push(Literal(c));
                }
                ']' => break,
                '-' => {
                    let Some(end) = self.chars.next() else {
                        return Err(());
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
                                    return Err(());
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
        Ok(Set(acc, !negate))
    }

    fn escaped_character(&mut self) -> Result<char, ()> {
        let Some(c) = self.chars.next() else {
            return Err(());
        };
        let c = match c {
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
                        return Err(());
                    };
                }
                let Ok(u) = u32::from_str_radix(&s, 16) else {
                    return Err(());
                };
                return char::from_u32(u).ok_or(());
            }
            // \uNNNN
            'u' => {
                let mut s = String::new();
                for _ in 0..4 {
                    if let Some(c) = self.chars.next() {
                        s.push(c);
                    } else {
                        return Err(());
                    };
                }
                let Ok(u) = u32::from_str_radix(&s, 16) else {
                    return Err(());
                };
                return char::from_u32(u).ok_or(());
            }
            // in particular: \'"
            c => c,
        };
        Ok(c)
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

fn repeat(acc: &mut Vec<Instruction>, n: usize, m: usize) {
    match (n, m) {
        // {n}
        (n, m) if n == m => {
            if n == 0 {
                acc.clear();
                return;
            }
            let atom = acc.clone();
            for _ in 0..(n - 1) {
                for v in &atom {
                    acc.push(v.clone());
                }
            }
        }
        // ?
        (0, 1) => {
            acc.insert(0, Instruction::Split(acc.len() + 1));
        }
        // *
        (0, usize::MAX) => {
            acc.insert(0, Instruction::Split(acc.len() + 2));
            acc.push(Instruction::Repeat(acc.len()));
        }
        // +
        (1, usize::MAX) => {
            let atom = acc.clone();
            acc.push(Instruction::Split(atom.len() + 2));
            for v in &atom {
                acc.push(v.clone());
            }
            acc.push(Instruction::Repeat(atom.len() + 1));
        }
        // {n,}
        (n, usize::MAX) => {
            let atom = acc.clone();
            for _ in 0..n.saturating_sub(1) {
                for v in &atom {
                    acc.push(v.clone());
                }
            }
            acc.push(Instruction::Split(atom.len() + 2));
            for v in &atom {
                acc.push(v.clone());
            }
            acc.push(Instruction::Repeat(atom.len() + 1));
        }
        // {n,m}
        (n, m) => {
            let atom = acc.clone();
            for _ in 0..n.saturating_sub(1) {
                for v in &atom {
                    acc.push(v.clone());
                }
            }
            // TODO: this is inefficient, e.g. say that m-n is huge
            let end = (atom.len() + 1) * m;
            for _ in 0..(m - n) {
                // on fail, jump over the repetitions
                acc.push(Instruction::Split(end));
                for v in &atom {
                    acc.push(v.clone());
                }
            }
        }
    }
}
