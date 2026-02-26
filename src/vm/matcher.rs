// https://swtch.com/~rsc/regexp/regexp2.html

use super::types::{Character, Input, Instruction, Regex};

impl Regex {
    pub fn is_match(&self, string: &str) -> bool {
        let mut state = MatchState::default();
        let input = Input::from(string);
        state.is_match(&self.instructions, &input)
    }

    #[cfg(test)]
    pub(crate) fn captures(&mut self, string: &str) -> Vec<String> {
        let mut state = MatchState::default();
        let input = Input::from(string);
        let mut acc = Vec::new();
        if state.is_match(&self.instructions, &input) {
            acc.push(input.string_slice(state.start, state.sp));
            for (a, b) in &state.groups {
                let s = input.string_slice(*a, *b);
                acc.push(s);
            }
        }
        acc
    }
}

#[derive(Debug, Default, Clone)]
struct MatchState {
    /// cursor pointing at the current instruction
    pc: usize,
    /// current position in string
    sp: usize,
    start: usize,
    /// (start, end)
    groups: Vec<(usize, usize)>,
    /// Queue of groups being matched
    active_group: Vec<usize>,
}

impl MatchState {
    fn is_match(&mut self, regex: &[Instruction], string: &Input) -> bool {
        use Instruction::*;
        let cache = &mut Vec::new();
        while self.pc < regex.len() {
            #[cfg(debug_assertions)]
            print_debug(regex, self, string);

            match &regex[self.pc] {
                Start => {
                    if self.sp == 0 {
                        self.pc += 1;
                    } else if !self.backtrack(cache, string.len()) {
                        return false;
                    }
                }
                End => {
                    if self.sp >= string.len() {
                        self.pc += 1;
                    } else if !self.backtrack(cache, string.len()) {
                        return false;
                    }
                }
                Boundary(b) => {
                    if self.is_boundary(string) == *b {
                        self.pc += 1;
                    } else if !self.backtrack(cache, string.len()) {
                        return false;
                    }
                }
                Is(pattern) => {
                    if let Some(c) = string.get(self.sp)
                        && pattern.is_match(*c)
                    {
                        #[cfg(debug_assertions)]
                        println!(" > OK");

                        self.pc += 1;
                        self.sp += 1;
                    } else if !self.backtrack(cache, string.len()) {
                        return false;
                    }
                }
                Split(jump) => {
                    let mut snapshot = self.clone();
                    snapshot.pc += jump;
                    cache.push(snapshot);
                    self.pc += 1;
                }
                Repeat(num) => self.pc -= num,
                Skip(num) => self.pc += num,
                Group(id) => {
                    if let Some((a, b)) = self.groups.get(id - 1) {
                        let slice = string.char_slice(*a, *b);
                        if string.contains_at(self.sp, slice) {
                            self.pc += 1;
                            self.sp += b - a;
                        } else if !self.backtrack(cache, string.len()) {
                            return false;
                        }
                    } else {
                        // skipping: this group does not exist
                        self.pc += 1;
                    }
                }
                GroupStart => {
                    self.groups.push((self.sp, self.sp));
                    self.active_group.push(self.groups.len() - 1);
                    self.pc += 1;
                }
                GroupEnd => {
                    let pos = self.active_group.pop().unwrap();
                    self.groups[pos].1 = self.sp;
                    self.pc += 1;
                }
                Recurse => {
                    let mut state = MatchState {
                        sp: self.sp,
                        start: self.start,
                        ..Default::default()
                    };
                    if state.is_match(regex, string) {
                        self.sp = state.sp;
                        self.pc += 1;
                    } else if !self.backtrack(cache, string.len()) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn backtrack(&mut self, cache: &mut Vec<MatchState>, string_length: usize) -> bool {
        #[cfg(debug_assertions)]
        println!(" > FAIL -> backtracking");

        if let Some(snapshot) = cache.pop() {
            *self = snapshot;
            true
        } else if self.sp + 1 < string_length {
            // start fresh from the next character
            self.pc = 0;
            self.groups.clear();
            self.active_group.clear();
            self.start += 1;
            self.sp = self.start;
            true
        } else {
            false
        }
    }

    fn is_boundary(&self, string: &Input) -> bool {
        if self.sp > 0
            && let Some(a) = string.get(self.sp - 1)
            && let Some(b) = string.get(self.sp)
        {
            a.is_whitespace() != b.is_whitespace()
        } else {
            !string.get(self.sp).is_some_and(|c| c.is_whitespace())
        }
    }
}

impl Character {
    fn is_match(&self, c: char) -> bool {
        use Character::*;
        match self {
            Anything => true,
            Literal(v) => *v == c,
            Space(b) => c.is_whitespace() == *b,
            Digit(b) => c.is_ascii_digit() == *b,
            Word(b) => (c.is_alphanumeric() || c == '_') == *b,
            Range(l, u) => *l <= c && c <= *u,
            Set(set, b) => set.iter().any(|v| v.is_match(c)) == *b,
        }
    }
}

#[cfg(debug_assertions)]
fn print_debug(regex: &[Instruction], state: &MatchState, string: &Input) {
    use Instruction::*;
    let node = &regex[state.pc];
    print!(" {}: {}", state.pc, node);
    match node {
        Group(id) => {
            if let Some((a, b)) = state.groups.get(id - 1) {
                let s = string.string_slice(*a, *b);
                println!(" ({})", s);
            } else {
                println!();
            }
        }
        GroupStart => println!(" {}", state.groups.len()),
        GroupEnd => println!(" {}", state.active_group.last().unwrap_or(&0)),
        Split(n) => println!(" (jump to {})", state.pc + n),
        Repeat(n) => println!(" (jump to {})", state.pc - n),
        Skip(n) => println!(" (jump to {})", state.pc + n),
        _ => println!(),
    }
    println!(" | {}", string);
    let spaces = String::from_utf8(vec![b' '; state.start]).unwrap();
    let markers = String::from_utf8(vec![b'^'; state.sp - state.start + 1]).unwrap();
    println!(" | {}{}", spaces, markers);
}
