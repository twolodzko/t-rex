use super::utils::{Queue, is_special};
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    hash::Hash,
    rc::Rc,
};

#[derive(Debug)]
pub struct Regex(pub(super) State);

#[derive(Debug, Clone, Default)]
pub(super) struct State(Rc<RefCell<Node>>);

#[derive(Debug, Clone, Default)]
struct Node {
    arrows: Vec<Arrow>,
    end: bool,
}

#[derive(Debug, Clone)]
pub(super) enum Arrow {
    Epsilon(State),
    Match(Character, State),
    Guard(Boundary, State),
}

#[derive(Debug, Clone)]
pub(super) enum Character {
    /// .
    Anything,
    /// literal character: a, b, 8, \n, \r, etc
    Literal(char),
    /// \s
    Space,
    /// \w
    Word,
    /// \d
    Digit,
    /// a-z
    Range(char, char),
    /// [ab-z-]
    Set(Vec<Character>),
    /// negates the match, e.g. Not(\s) = \S, Not([abc]) = [^abc]
    Not(Box<Character>),
}

impl std::fmt::Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Character::*;
        match self {
            Anything => write!(f, "."),
            Literal(' ') => write!(f, r"\s"),
            Literal('\n') => write!(f, r"\n"),
            Literal('\t') => write!(f, r"\t"),
            Literal('\0') => write!(f, r"\0"),
            Literal(c) if is_special(*c) => write!(f, "\\{}", c),
            Literal(c) => write!(f, "{}", c),
            Space => write!(f, r"\s"),
            Digit => write!(f, r"\d"),
            Word => write!(f, r"\w"),
            Range(l, u) => write!(f, "{}-{}", l, u),
            Set(s) => {
                write!(
                    f,
                    "[{}]",
                    s.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join("")
                )
            }
            Not(s) => match s.as_ref() {
                Space => write!(f, r"\S"),
                Digit => write!(f, r"\D"),
                Word => write!(f, r"\W"),
                Set(s) => {
                    write!(
                        f,
                        "[^{}]",
                        s.iter()
                            .map(ToString::to_string)
                            .collect::<Vec<String>>()
                            .join("")
                    )
                }
                _ => unreachable!(),
            },
        }
    }
}

impl std::ops::Not for Character {
    type Output = Character;

    fn not(self) -> Self::Output {
        Character::Not(Box::new(self))
    }
}

#[derive(Debug, Clone)]
pub(super) enum Boundary {
    /// ^
    Start,
    /// $
    End,
    /// \b, \B
    Word(bool),
}

impl std::fmt::Display for Boundary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Boundary::*;
        match self {
            Start => write!(f, "^"),
            End => write!(f, "$"),
            Word(true) => write!(f, r"\b"),
            Word(false) => write!(f, r"\B"),
        }
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl Eq for State {}

impl Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0.as_ptr(), state);
    }
}

impl Regex {
    pub fn graph(&self) -> String {
        use Arrow::*;
        let nodes = self.nodes();
        #[allow(clippy::mutable_key_type)]
        let mut ids = HashMap::new();
        for (i, n) in nodes.iter().enumerate() {
            ids.insert(n, i);
        }
        let edges = nodes
            .iter()
            .flat_map(|s| {
                s.0.borrow()
                    .arrows
                    .iter()
                    .map(|a| match a {
                        Epsilon(n) => {
                            format!(
                                "  {} -> {} [label = \"\u{03B5}\"]",
                                ids.get(s).unwrap(),
                                ids.get(n).unwrap(),
                            )
                        }
                        Match(c, n) => {
                            format!(
                                "  {} -> {} [label = \"{}\"]",
                                ids.get(s).unwrap(),
                                ids.get(n).unwrap(),
                                c
                            )
                        }
                        Guard(c, n) => {
                            format!(
                                "  {} -> {} [label = \"{}\"]",
                                ids.get(s).unwrap(),
                                ids.get(n).unwrap(),
                                c
                            )
                        }
                    })
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<String>>()
            .join("\n");
        format!("digraph G {{\n  rankdir=LR;\n{}\n}}", edges).replace(r"\", r"\\")
    }

    pub(super) fn nodes(&self) -> Vec<State> {
        let mut queue = Queue::default();
        let mut nodes = vec![self.0.clone()];
        queue.push(self.0.clone());
        while let Some(s) = queue.pop() {
            for a in &s.arrows() {
                use Arrow::*;
                let (Epsilon(n) | Match(_, n) | Guard(_, n)) = a;
                if queue.push(n.clone()) {
                    nodes.push(n.clone());
                }
            }
        }
        nodes
    }
}

impl State {
    /// Iterate over arrows pointing out of the state
    pub(super) fn arrows(&self) -> Iter<'_> {
        // see: https://stackoverflow.com/a/33542412/
        Iter { r: self.0.borrow() }
    }

    pub(super) fn is_final(&self) -> bool {
        self.0.borrow().end
    }

    pub(super) fn reaches_final(&self) -> bool {
        self.arrows().into_iter().any(|a| match a {
            Arrow::Epsilon(s) => s.is_final() || s.reaches_final(),
            _ => false,
        })
    }

    pub(super) fn addr(&self) -> usize {
        self.0.as_ptr().addr()
    }

    pub(super) fn insert_arrow(&self, arrow: Arrow) {
        self.0.borrow_mut().arrows.push(arrow);
    }

    pub(super) fn end() -> Self {
        let state = State::default();
        state.0.borrow_mut().end = true;
        state
    }
}

pub(super) struct Iter<'a> {
    r: Ref<'a, Node>,
}

impl<'a, 'b: 'a> IntoIterator for &'b Iter<'a> {
    type IntoIter = std::slice::Iter<'a, Arrow>;
    type Item = &'a Arrow;

    fn into_iter(self) -> Self::IntoIter {
        self.r.arrows.iter()
    }
}
