use super::parser::parse;
use test_case::test_case;

// #[test_case("^(abc|d(e)?f*)+x")]
// fn compiler(input: &str) {
//     let result = parse(input).unwrap();

//     println!("input: {}\n", input);
//     result.show_code();

//     assert!(false)
// }

#[test_case(
        r"a",
        "a",
        true,
        &["a"];
        "single character"
    )]
#[test_case(
        r"a",
        "X",
        false,
        &[];
        "false match for single character"
    )]
#[test_case(
        r"a",
        "XXXXXX",
        false,
        &[];
        "no match for single char in a string"
    )]
#[test_case(
        r"a",
        "abc",
        true,
        &["a"];
        "input is longer than pattern"
    )]
#[test_case(
        r"a+",
        "aaa",
        true,
        &["aaa"];
        "repeated character with open bounds"
    )]
#[test_case(
        r"a{5,}.",
        "aaaXaaaaaY",
        true,
        &["aaaaaY"];
        "repeated without upper bound"
    )]
#[test_case(
        r"a+",
        "xxx",
        false,
        &[];
        "false match for repeated character with open bounds"
    )]
#[test_case(
        r"a{3}b",
        "aaaaaaaabaaaa",
        true,
        &["aaab"];
        "character repeated 3 times"
    )]
#[test_case(
        r"a{3,5}",
        "aaaaaaaaaaaa",
        true,
        &["aaaaa"];
        "repeated character with fixed bounds"
    )]
#[test_case(
        r"a{3,5}",
        "aa",
        false,
        &[];
        "false match for repeated character with fixed bounds"
    )]
#[test_case(
        r"a+b",
        "aaab",
        true,
        &["aaab"];
        "repeated character"
    )]
#[test_case(
        r"ab*c",
        "abbbbbc",
        true,
        &["abbbbbc"];
        "repeated character inside bounds"
    )]
#[test_case(
        r"a\b",
        "a ",
        true,
        &["a"];
        "word boundary"
    )]
#[test_case(
        r"a\Bc",
        "ac",
        true,
        &["ac"];
        "non word boundary"
    )]
#[test_case(
        r"\b",
        "abc",
        true,
        &[""];
        "word boundary only"
    )]
#[test_case(
        r"a|b",
        "b",
        true,
        &["b"];
        "alteration"
    )]
#[test_case(
        r"a|b",
        "X",
        false,
        &[];
        "false match for alteration"
    )]
#[test_case(
        r"^$",
        "",
        true,
        &[""];
        "empty string"
    )]
#[test_case(
        r"^abc$|^def$",
        "def",
        true,
        &["def"];
        "alternate full matches"
    )]
#[test_case(
        r"$",
        "",
        true,
        &[""];
        "empty string only end"
    )]
#[test_case(
        r"^",
        "",
        true,
        &[""];
        "empty string only start"
    )]
#[test_case(
        r"^.*$",
        "",
        true,
        &[""];
        "any whole string empty"
    )]
#[test_case(
        r"[a-z]+",
        "&*&**&*&$#$$",
        false,
        &[];
        "negative match set"
    )]
#[test_case(
        r"[^a-z]{3,10}",
        "&a*&**&*&aaa$#$$",
        true,
        &["*&**&*&"];
        "match negated set"
    )]
#[test_case(
        r"^.*$",
        "hello, world!",
        true,
        &["hello, world!"];
        "any whole string non empty"
    )]
#[test_case(
        r"(a+|(a.x)|(ab.))d",
        "abcd",
        true,
        &["abcd", "abc", "abc"];
        "alteration longer"
    )]
#[test_case(
        r"((a)(b)(x)|(a)(b))c",
        "abc",
        true,
        &["abc", "ab", "a", "b"];
        "matching groups"
    )]
#[test_case(
        r"(a)(b)cx|(a)bc",
        "abc",
        true,
        &["abc", "a"];
        "more matching groups on left"
    )]
#[test_case(
        r"(a)",
        "abc",
        true,
        &["a", "a"];
        "single character group"
    )]
#[test_case(
        r"(a)bcx|(a)(b)c",
        "abc",
        true,
        &["abc", "a", "b"];
        "more matching groups on right"
    )]
#[test_case(
        r"a+",
        "xxx aaa xxx",
        true,
        &["aaa"];
        "find subpattern"
    )]
#[test_case(
        r"a+b",
        "aaax aab",
        true,
        &["aab"];
        "needs to backtrack"
    )]
#[test_case(
        r"(a+)|(b+)",
        "xxx",
        false,
        &[];
        "false match for either with repeated char"
    )]
#[test_case(
        r"(\w+) (\w+) \1",
        "The Latin phrase 'idem per idem' means 'the same for the same'.",
        true,
        &["idem per idem", "idem", "per"];
        "match groups recursively"
    )]
#[test_case(
        r"a\5",
        "aaaaaa",
        true,
        &["a"];
        "match groups recursively for nonexistent group"
    )]
#[test_case(
        r"(x+x+)+y",
        "xxxxz xxxxxy",
        true,
        &["xxxxxy", "xxxxx"];
        "catastrophic backtracking example"
    )]
#[test_case(
        r"a(?R)?",
        "aaa",
        true,
        &["aaa"];
        "trivial recursive pattern"
    )]
#[test_case(
        r"\((?R)?\)",
        "((()))",
        true,
        &["((()))"];
        "inner recursive pattern"
    )]
#[test_case(
        r"\w{3}\d{3}(?R)?",
        "aaa111bbb222",
        true,
        &["aaa111bbb222"];
        "recursive pattern"
    )]
#[test_case(
        r"(\w)((?R)|\w?)\1",
        "okonoko",
        true,
        &["okonoko", "o", "konok"];
        "palindrome"
    )]
fn matching(regex: &str, input: &str, expected_status: bool, expected_matches: &[&str]) {
    let mut rx = parse(regex).unwrap();

    println!("Using regex: {}", regex);
    println!("Matching:    {}\n", input);
    rx.show_code();
    println!();

    assert_eq!(rx.is_match(input), expected_status);
    assert_eq!(rx.captures(input), expected_matches);
}

// #[test_case(
//         r"a",
//         Branch(vec![
//             Value(Literal('a'))
//         ].into());
//         "single character"
//     )]
// #[test_case(
//         r"abc",
//         Branch(vec![
//             Value(Literal('a')),
//             Value(Literal('b')),
//             Value(Literal('c')),
//         ].into());
//         "many characters"
//     )]
// #[test_case(
//         r"^.$",
//         Branch(vec![
//             Start, Anything, End,
//         ].into());
//         "start end any"
//     )]
// #[test_case(
//         r"a+",
//         Branch(vec![
//             Repeat(types::Repeat::new(Value(Literal('a')), Between(1, usize::MAX)))
//         ].into());
//         "more than one character"
//     )]
// #[test_case(
//         r"(\n)+",
//         Branch(vec![
//             Repeat(types::Repeat::new(Group(Branch(vec![Value(Literal('\n'))].into()), true), Between(1, usize::MAX)))
//         ].into());
//         "more than one groups"
//     )]
// #[test_case(
//         r"\S{7,9}",
//         Branch(vec![
//             Repeat(types::Repeat::new(Value(Space(false)), Between(7, 9)))
//         ].into());
//         "repeat non space"
//     )]
// #[test_case(
//         r"\d{7,}",
//         Branch(vec![
//             Repeat(types::Repeat::new(Value(Digit(true)), Between(7, usize::MAX)))
//         ].into());
//         "repeat digit"
//     )]
// #[test_case(
//         r"(){42}",
//         Branch(vec![
//             Repeat(
//                 types::Repeat::new(Group(Branch(Default::default()), true), Exact(42)),
//             )
//         ].into());
//         "repeat empty group"
//     )]
// #[test_case(
//         r"[]]?",
//         Branch(vec![
//             Repeat(types::Repeat::new(Set(vec![Literal(']')], false), Between(0, 1)))
//         ].into());
//         "zero or one set with bracket"
//     )]
// #[test_case(
//         r"[--]",
//         Branch(vec![
//             Set(vec![Literal('-'), Literal('-')], false)
//         ].into());
//         "minus minus set"
//     )]
// #[test_case(
//         r"[--4]",
//         Branch(vec![
//             Set(vec![Range('-', '4')], false)
//         ].into());
//         "minus range set"
//     )]
// #[test_case(
//         r"[-a-z-]",
//         Branch(vec![Set(vec![
//             Literal('-'),
//             Range('a', 'z'),
//             Literal('-'),
//         ], false)].into());
//         "range with minuses on both sides"
//     )]
// #[test_case(
//         r"[]-}]",
//         Branch(vec![
//             Set(vec![Range(']', '}')], false)
//         ].into());
//         "set of strange chars"
//     )]
// #[test_case(
//         r"[^][]]",
//         Branch(vec![
//             Set(vec![Literal(']'), Literal('[')], true),
//             Value(Literal(']')),
//         ].into());
//         "set of non brackets and bracket"
//     )]
// #[test_case(
//         r"()|()",
//         Either(vec![
//             vec![Group(Branch(Default::default()), true)].into(),
//             vec![Group(Branch(Default::default()), true)].into(),
//         ].into());
//         "either empty set"
//     )]
// #[test_case(
//         r"a|b",
//         Either(vec![
//             vec![Value(Literal('a'))].into(),
//             vec![Value(Literal('b'))].into(),
//         ].into());
//         "either a or b"
//     )]
// #[test_case(
//         r"(a|b)",
//         Branch(vec![Group(Either(vec![
//             vec![Value(Literal('a'))].into(),
//             vec![Value(Literal('b'))].into(),
//         ].into()), true)].into());
//         "group of either a or b"
//     )]
// fn parsing(input: &str, expected: Regex) {
//     let mut parser = Parser::from(input);
//     let result = parser.regex().unwrap();
//     assert_eq!(result, expected);
//     assert_eq!(result.to_string(), input);
// }
