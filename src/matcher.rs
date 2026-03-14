use super::{
    types::{Arrow, Boundary, Character, Regex},
    utils::{Queue, to_lower},
};

impl Regex {
    pub fn is_match(&self, string: &str) -> bool {
        use Arrow::*;
        if self.graph.reaches_final() {
            // regex that always matches
            return true;
        }

        let chars = if self.ignore_case {
            string.chars().map(to_lower).collect::<Vec<char>>()
        } else {
            string.chars().collect::<Vec<char>>()
        };

        for i in 0..=chars.len() {
            // use queue of the states visit, it makes sure not to re-visit the already seen ones
            let mut queue = Queue::default();
            queue.push(self.graph.clone());

            // go beyond the string length to make sure it reaches the end state
            for j in i..=chars.len() {
                // match character against multiple states simultaneously
                let mut next = Queue::default();
                while let Some(state) = queue.pop() {
                    if state.is_final() {
                        return true;
                    }
                    for arrow in &state.arrows() {
                        match arrow {
                            Epsilon(s) => {
                                if s.is_final() {
                                    // early exit
                                    return true;
                                }
                                // instantly jump to next state
                                // because it did not match on current character
                                queue.push(s.clone());
                            }
                            Match(v, s) => {
                                // conditionally jump to next state
                                if let Some(c) = chars.get(j)
                                    && v.is_match(c)
                                {
                                    if s.is_final() {
                                        // early exit
                                        return true;
                                    }
                                    // add states to be used for matching the next character
                                    next.push(s.clone());
                                }
                            }
                            Guard(g, s) => {
                                use Boundary::*;
                                if match g {
                                    Start if j == 0 => true,
                                    End if j >= chars.len() => true,
                                    Word(yes) if is_boundary(&chars, j) == *yes => true,
                                    _ => false,
                                } {
                                    if s.is_final() {
                                        // early exit
                                        return true;
                                    }
                                    // instantly jump to next state
                                    // guards are non-consuming
                                    queue.push(s.clone());
                                }
                            }
                        }
                    }
                }
                // using the queue for the next character
                queue = next;
            }
        }
        false
    }
}

impl Character {
    fn is_match(&self, c: &char) -> bool {
        use Character::*;
        match self {
            Anything => true,
            Literal(s) => s == c,
            Space => c.is_whitespace(),
            Digit => c.is_ascii_digit(),
            Word => c.is_alphanumeric() || *c == '_',
            Range(l, u) => l <= c && c <= u,
            Set(s) => s.iter().any(|v| v.is_match(c)),
            Not(s) => !s.is_match(c),
        }
    }
}

fn is_boundary(string: &[char], index: usize) -> bool {
    if index > 0
        && let Some(a) = string.get(index - 1)
        && let Some(b) = string.get(index)
    {
        a.is_whitespace() != b.is_whitespace()
    } else {
        !string.get(index).is_some_and(|c| c.is_whitespace())
    }
}
