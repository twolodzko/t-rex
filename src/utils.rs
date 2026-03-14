use super::types::State;
use std::collections::HashSet;

#[inline]
pub(super) fn is_special(c: char) -> bool {
    matches!(
        c,
        '^' | '.' | '[' | '$' | '(' | ')' | '|' | '*' | '+' | '?' | '{' | '\\'
    )
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
