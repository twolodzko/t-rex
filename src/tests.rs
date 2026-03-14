use super::Regex;
use std::str::FromStr;
use test_case::test_case;

#[test_case(
        "",
        "",
        true;
        "empty regex for empty input"
    )]
#[test_case(
        "",
        "abcdfef",
        true;
        "empty regex for non-empty input"
    )]
#[test_case(
        "a",
        "a",
        true;
        "single letter"
    )]
#[test_case(
        "a",
        "xxx",
        false;
        "negative match for single letter"
    )]
#[test_case(
        "abc",
        "abc",
        true;
        "several letters"
    )]
#[test_case(
        "abc",
        "axx",
        false;
        "negative match for several letters"
    )]
#[test_case(
        "a{5}",
        "aaaaaaa",
        true;
        "repeated letters"
    )]
#[test_case(
        "a{5}",
        "aabcde",
        false;
        "negative match for repeated letters"
    )]
#[test_case(
        "a{5}",
        "aaa",
        false;
        "negative match for repeated letters wrong count"
    )]
#[test_case(
        "a{3,}",
        "aaa",
        true;
        "repeated letters with min count input is min"
    )]
#[test_case(
        "a{3,}",
        "aaaa",
        true;
        "repeated letters with min count input is more"
    )]
#[test_case(
        "a{3,}",
        "aa",
        false;
        "negative match for repeated letters with min count"
    )]
#[test_case(
        "a{3,5}",
        "aaa",
        true;
        "bounded repeat with min count"
    )]
#[test_case(
        "a{3,5}",
        "aaaa",
        true;
        "bounded repeat with mid count"
    )]
#[test_case(
        "a{3,5}",
        "aaaaa",
        true;
        "bounded repeat with max count"
    )]
#[test_case(
        "a{3,5}",
        "aa",
        false;
        "negative match for bounded repeat with too small count"
    )]
#[test_case(
        "a{3,5}",
        "aabc",
        false;
        "negative match for bounded repeat with wrong chars"
    )]
#[test_case(
        "a+",
        "a",
        true;
        "at least one letter for one letter"
    )]
#[test_case(
        "a+",
        "aaa",
        true;
        "at least one letter for many letters"
    )]
#[test_case(
        "a+",
        "bdesdf",
        false;
        "negative match at least one letter"
    )]
#[test_case(
        "a*",
        "aaaaaa",
        true;
        "infinite loop of letters"
    )]
#[test_case(
        "a*",
        "",
        true;
        "infinite loop of letters for empty"
    )]
#[test_case(
        "a*a*a*a*",
        "",
        true;
        "repeated infinite loop of letters for empty"
    )]
#[test_case(
        "a|b",
        "a",
        true;
        "trivial alternative left branch"
    )]
#[test_case(
        "a|b",
        "b",
        true;
        "trivial alternative right branch"
    )]
#[test_case(
        "abc|def",
        "def",
        true;
        "longer alternative"
    )]
#[test_case(
        "a|b",
        "x",
        false;
        "negative match for alternative"
    )]
#[test_case(
        "^abc$",
        "abc",
        true;
        "bounded string with guards"
    )]
#[test_case(
        "^$",
        "",
        true;
        "empty string with guards"
    )]
#[test_case(
        r"^abc$|^def$",
        "def",
        true;
        "alternate full matches"
    )]
#[test_case(
        r"a\b",
        "a ",
        true;
        "word boundary"
    )]
#[test_case(
        r"a\Bc",
        "ac",
        true;
        "non word boundary"
    )]
#[test_case(
        r"\b",
        "abc",
        true;
        "word boundary only"
    )]
fn matches(regex: &str, example: &str, expected: bool) {
    let regex = Regex::from_str(regex).unwrap();
    println!("{}", regex.graph());
    assert_eq!(regex.is_match(example), expected);
}
