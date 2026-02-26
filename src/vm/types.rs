use super::parser;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Regex {
    /// The instructions that are executed
    pub(crate) instructions: Vec<Instruction>,
}

impl FromStr for Regex {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parser::parse(s) {
            Ok(ok) => Ok(ok),
            Err(_) => Err("failed to parse".to_string()),
        }
    }
}

pub struct Input {
    chars: Vec<char>,
}

#[derive(Debug, Clone)]
pub(crate) enum Instruction {
    /// match beginning of the string
    Start,
    /// match end of the string
    End,
    /// match word boundary
    Boundary(bool),
    /// match a character
    Is(Character),
    /// create a backtrack path
    Split(usize),
    /// repeat last N instructions
    Repeat(usize),
    /// skip N instructions
    Skip(usize),
    /// match the current position of the string
    /// vs the already matched group with given;
    /// if the group does not exist, it is considered as empty string
    Group(usize),
    /// start matching a group
    GroupStart,
    /// finalize matching a group
    GroupEnd,
    /// recurse the whole pattern
    Recurse,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Character {
    /// .
    Anything,
    /// literal character: a, b, 8, \n, \r, etc
    Literal(char),
    /// \s \S
    Space(bool),
    /// \w \W
    Word(bool),
    /// \d \D
    Digit(bool),
    /// a-z
    Range(char, char),
    /// [ab-z-]
    Set(Vec<Character>, bool),
}

impl Regex {
    #[cfg(test)]
    pub(crate) fn show_code(&self) {
        if !self.instructions.is_empty() {
            let digits = self.instructions.len().ilog10() as usize + 1;
            for (pos, instruction) in self.instructions.iter().enumerate() {
                print!("{:digits$}  {}", pos, instruction);
                match instruction {
                    Instruction::Split(n) => println!(" (else backtrack to {})", pos + n),
                    Instruction::Repeat(n) => println!(" (else jump to {})", pos - n),
                    Instruction::Skip(n) => println!(" (jump to {})", pos + n),
                    _ => println!(),
                }
            }
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Instruction::*;
        match self {
            Start => write!(f, r"match ^"),
            End => write!(f, r"match $"),
            Boundary(true) => write!(f, r"match \b"),
            Boundary(false) => write!(f, r"match \B"),
            Is(c) => write!(f, "match {}", c),
            Split(n) => write!(f, "split {}", n),
            Repeat(n) => write!(f, "repeat {}", n),
            Skip(n) => write!(f, "skip {}", n),
            Group(id) => write!(f, "starts with \\{}", id),
            GroupStart => write!(f, "group start"),
            GroupEnd => write!(f, "group end"),
            Recurse => write!(f, "recurse"),
        }
    }
}

impl std::fmt::Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Character::*;
        match self {
            Anything => write!(f, "."),
            Literal(c) if is_special(*c) => write!(f, "\\{}", c),
            Literal(c) => write!(f, "{}", c),
            Space(true) => write!(f, r"\s"),
            Space(false) => write!(f, r"\S"),
            Digit(true) => write!(f, r"\d"),
            Digit(false) => write!(f, r"\D"),
            Word(true) => write!(f, r"\w"),
            Word(false) => write!(f, r"\W"),
            Range(l, u) => write!(f, "{}-{}", l, u),
            Set(set, b) => {
                let s = set
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join("");
                if *b {
                    write!(f, "[{}]", s)
                } else {
                    write!(f, "[^{}]", s)
                }
            }
        }
    }
}

fn is_special(c: char) -> bool {
    matches!(
        c,
        '^' | '.' | '[' | '$' | '(' | ')' | '|' | '*' | '+' | '?' | '{' | '\\'
    )
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in &self.chars {
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}

impl From<&str> for Input {
    fn from(value: &str) -> Self {
        Input {
            chars: value.chars().collect(),
        }
    }
}

impl Input {
    pub(crate) fn get(&self, index: usize) -> Option<&char> {
        self.chars.get(index)
    }

    pub(crate) fn len(&self) -> usize {
        self.chars.len()
    }

    #[cfg(debug_assertions)]
    pub(crate) fn string_slice(&self, a: usize, b: usize) -> String {
        self.chars[a..b].iter().collect()
    }

    pub(crate) fn char_slice(&self, a: usize, b: usize) -> &[char] {
        &self.chars[a..b]
    }

    pub(crate) fn contains_at(&self, index: usize, pattern: &[char]) -> bool {
        self.chars[index..].starts_with(pattern)
    }
}
