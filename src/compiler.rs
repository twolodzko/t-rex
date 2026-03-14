use super::{
    Regex,
    parser::{ParsingError, Token, parse},
    types::{Arrow, State},
    utils::regex_to_lower,
};

impl Regex {
    pub fn new(regex: &str, ignore_case: bool) -> Result<Regex, ParsingError> {
        let token = if ignore_case {
            let regex = regex_to_lower(regex);
            parse(&regex)?
        } else {
            parse(regex)?
        };
        let start = State::default();
        make_link(&token, start.clone(), State::end());
        Ok(Regex {
            graph: start,
            ignore_case,
        })
    }
}

fn make_link(token: &Token, start: State, end: State) {
    use Arrow::*;
    use Token::*;
    match token {
        // ^, $
        Special(g) => {
            start.insert_arrow(Guard(g.clone(), end));
        }
        // a
        Matcher(c) => {
            start.insert_arrow(Match(c.clone(), end));
        }
        // abc
        Branch(b) => link_branch(b, start, end),
        // a|b
        Alternative(b) => {
            for t in b {
                let state = State::default();
                make_link(t, start.clone(), state.clone());
                state.insert_arrow(Epsilon(end.clone()));
            }
        }
        // a*
        Repeat(t, 0, None) => make_loop(t, start, end),
        // a+, a{m,}
        Repeat(t, m, None) => {
            repeated(t, *m, start, end.clone());
            make_link(t, end.clone(), end);
        }
        // a{m}
        Repeat(t, m, Some(n)) if m == n => repeated(t, *m, start, end),
        // a{m,n}
        Repeat(t, m, Some(n)) => range(t, *m, *n, start, end),
    }
}

fn link_branch(branch: &[Token], start: State, end: State) {
    use Arrow::Epsilon;
    let mut this = start;
    for t in branch.iter().take(branch.len().saturating_sub(1)) {
        let next = State::default();
        make_link(t, this.clone(), next.clone());
        this = next;
    }
    if let Some(t) = branch.last() {
        make_link(t, this, end);
    } else {
        // handle empty branch
        this.insert_arrow(Epsilon(end));
    }
}

fn make_loop(token: &Token, start: State, end: State) {
    use Arrow::Epsilon;
    make_link(token, start.clone(), start.clone());
    start.insert_arrow(Epsilon(end));
}

fn repeated(token: &Token, n: usize, start: State, end: State) {
    debug_assert!(n > 0);
    let rep: Vec<Token> = std::iter::repeat_n(token.clone(), n).collect();
    link_branch(&rep, start, end)
}

fn range(token: &Token, m: usize, n: usize, start: State, end: State) {
    use Arrow::Epsilon;
    let mut this;
    if m > 0 {
        this = State::default();
        repeated(token, m, start, this.clone());
    } else {
        this = start;
    }
    for _ in m..n - 1 {
        let next = State::default();
        make_link(token, this.clone(), next.clone());
        this.insert_arrow(Epsilon(end.clone()));
        this = next;
    }
    make_link(token, this.clone(), end.clone());
    this.insert_arrow(Epsilon(end));
}
