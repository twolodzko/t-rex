use super::types::State;
use std::collections::HashSet;

#[inline]
pub(super) fn is_special(c: char) -> bool {
    matches!(
        c,
        '^' | '.' | '[' | '$' | '(' | ')' | '|' | '*' | '+' | '?' | '{' | '\\'
    )
}

pub fn regex_to_lower(regex: &str) -> String {
    let mut acc = String::new();
    let mut chars = regex.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            acc.push(c);
            if let Some(c) = chars.next() {
                acc.push(c);
            }
        } else {
            acc.push(to_lower(c));
        }
    }
    acc
}

pub fn to_lower(c: char) -> char {
    // This character cannot be losslessly transformed to lowercase
    // https://stackoverflow.com/questions/35716159/what-is-the-motivation-of-rusts-tolowercase
    if c == '\u{130}' {
        return c;
    }
    c.to_lowercase().next().unwrap()
}

#[derive(Default)]
pub(super) struct Queue {
    seen: HashSet<usize>,
    queue: Vec<State>,
}

impl Queue {
    pub(super) fn push(&mut self, value: State) -> bool {
        if self.seen.insert(value.addr()) {
            self.queue.push(value);
            return true;
        }
        false
    }

    pub(super) fn pop(&mut self) -> Option<State> {
        self.queue.pop()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn regex_to_lower() {
        let input = r"abc+DEF*ĄΞ\W[\D]\n";
        let expected = r"abc+def*ąξ\W[\D]\n";
        let result = super::regex_to_lower(input);
        assert_eq!(result, expected)
    }
}
