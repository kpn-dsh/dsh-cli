use std::fmt::{Display, Formatter};

use regex::Regex;

use crate::query_processor::Part::{Matching, NonMatching};

#[derive(Debug, PartialEq)]
pub enum Part {
  Matching(String),
  NonMatching(String),
}

pub trait QueryProcessor: Send + Sync {
  /// Returns a description of the query
  ///
  /// ## Returns
  /// * a `String` describing the query processor
  fn describe(&self) -> String;

  /// # Applies query to string
  ///
  /// ## Parameters
  /// * `haystack` - `String` that will be searched for parts that match the query
  ///
  /// ## Returns
  /// * `Ok(Vec<Part>)` - when the `haystack` contains one or more parts that match the query
  /// * `None` - when the `haystack` did not match the query
  fn matching_parts(&self, haystack: &str) -> Option<Vec<Part>>;
}

impl Display for Part {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Matching(part) => write!(f, "{}", part),
      NonMatching(part) => write!(f, "{}", part),
    }
  }
}

pub struct ExactMatchQueryProcessor<'a> {
  pattern: &'a str,
}

impl<'a> ExactMatchQueryProcessor<'a> {
  pub fn create(pattern: &'a str) -> Result<Self, String> {
    Ok(Self { pattern })
  }
}

impl QueryProcessor for ExactMatchQueryProcessor<'_> {
  fn describe(&self) -> String {
    format!("match the pattern \"{}\"", self.pattern)
  }

  fn matching_parts(&self, haystack: &str) -> Option<Vec<Part>> {
    if self.pattern == haystack {
      Some(vec![Matching(haystack.to_string())])
    } else {
      None
    }
  }
}

pub struct RegexQueryProcessor {
  regex: Regex,
}

impl RegexQueryProcessor {
  pub fn create(pattern: &str) -> Result<Self, String> {
    match Regex::new(pattern) {
      Ok(regex) => Ok(Self { regex }),
      Err(error) => Err(error.to_string()),
    }
  }
}

impl QueryProcessor for RegexQueryProcessor {
  fn describe(&self) -> String {
    format!("match against regular expression \"{}\"", self.regex.as_str())
  }

  fn matching_parts(&self, haystack: &str) -> Option<Vec<Part>> {
    let mut parts: Vec<Part> = vec![];
    let mut ptr: usize = 0;
    let mut match_found = false;
    for matching in self.regex.find_iter(haystack) {
      if matching.start() > ptr {
        parts.push(NonMatching(haystack[ptr..matching.start()].to_string()))
      }
      match_found = true;
      parts.push(Matching(matching.as_str().to_string()));
      ptr = matching.end();
    }
    if haystack.len() > ptr {
      parts.push(NonMatching(haystack[ptr..haystack.len()].to_string()));
    }
    if match_found {
      Some(parts)
    } else {
      None
    }
  }
}

#[test]
fn test_exact_match_query_processor() {
  let haystacks: [(&str, &str, Option<Vec<Part>>); 4] = [("aa", "", None), ("aa", "a", None), ("aa", "aa", Some(vec![Matching("aa".to_string())])), ("aa", "aaa", None)];
  for (pattern, haystack, parts) in haystacks {
    let exact_match_query_processor = ExactMatchQueryProcessor::create(pattern).unwrap();
    assert_eq!(exact_match_query_processor.describe(), format!("match the pattern \"{}\"", pattern));
    assert_eq!(exact_match_query_processor.matching_parts(haystack), parts);
  }
}

#[test]
fn test_regex_query_processor() {
  let haystacks: [(&str, &str, Option<Vec<Part>>); 19] = [
    ("a+", "", None),
    ("a+", "b", None),
    ("a+", "a", Some(vec![Matching("a".to_string())])),
    ("a+", "aaa", Some(vec![Matching("aaa".to_string())])),
    (
      "a+",
      "bbabbbaab",
      Some(vec![
        NonMatching("bb".to_string()),
        Matching("a".to_string()),
        NonMatching("bbb".to_string()),
        Matching("aa".to_string()),
        NonMatching("b".to_string()),
      ]),
    ),
    (
      "a+",
      "aaabbabbbaab",
      Some(vec![
        Matching("aaa".to_string()),
        NonMatching("bb".to_string()),
        Matching("a".to_string()),
        NonMatching("bbb".to_string()),
        Matching("aa".to_string()),
        NonMatching("b".to_string()),
      ]),
    ),
    (
      "a+",
      "bbabbbaabaaa",
      Some(vec![
        NonMatching("bb".to_string()),
        Matching("a".to_string()),
        NonMatching("bbb".to_string()),
        Matching("aa".to_string()),
        NonMatching("b".to_string()),
        Matching("aaa".to_string()),
      ]),
    ),
    (
      "a+",
      "aaabbabbbaabaaa",
      Some(vec![
        Matching("aaa".to_string()),
        NonMatching("bb".to_string()),
        Matching("a".to_string()),
        NonMatching("bbb".to_string()),
        Matching("aa".to_string()),
        NonMatching("b".to_string()),
        Matching("aaa".to_string()),
      ]),
    ),
    ("aa", "", None),
    ("aa", "bbb", None),
    ("aa", "aa", Some(vec![Matching("aa".to_string())])),
    ("aa", "aaa", Some(vec![Matching("aa".to_string()), NonMatching("a".to_string())])),
    ("aa", "aaaa", Some(vec![Matching("aa".to_string()), Matching("aa".to_string())])),
    (
      "aa",
      "aaaaa",
      Some(vec![Matching("aa".to_string()), Matching("aa".to_string()), NonMatching("a".to_string())]),
    ),
    ("aa", "aaabb", Some(vec![Matching("aa".to_string()), NonMatching("abb".to_string())])),
    (
      "aa",
      "bbaaabbbaaab",
      Some(vec![
        NonMatching("bb".to_string()),
        Matching("aa".to_string()),
        NonMatching("abbb".to_string()),
        Matching("aa".to_string()),
        NonMatching("ab".to_string()),
      ]),
    ),
    (
      "aa",
      "aaabbabbbaab",
      Some(vec![
        Matching("aa".to_string()),
        NonMatching("abbabbb".to_string()),
        Matching("aa".to_string()),
        NonMatching("b".to_string()),
      ]),
    ),
    (
      "aa",
      "bbabbbaabaaa",
      Some(vec![
        NonMatching("bbabbb".to_string()),
        Matching("aa".to_string()),
        NonMatching("b".to_string()),
        Matching("aa".to_string()),
        NonMatching("a".to_string()),
      ]),
    ),
    (
      "aa",
      "aaabbabbbaabaaa",
      Some(vec![
        Matching("aa".to_string()),
        NonMatching("abbabbb".to_string()),
        Matching("aa".to_string()),
        NonMatching("b".to_string()),
        Matching("aa".to_string()),
        NonMatching("a".to_string()),
      ]),
    ),
  ];
  for (pattern, haystack, parts) in haystacks {
    let regex_query_processor = RegexQueryProcessor::create(pattern).unwrap();
    assert_eq!(regex_query_processor.describe(), format!("match against regular expression \"{}\"", pattern));
    assert_eq!(regex_query_processor.matching_parts(haystack), parts);
  }
}
