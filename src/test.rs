#![cfg(test)]

use std::str::FromStr;

use glob::Pattern;
use uuid::Uuid;

#[test]
fn glob_match_child_glob() {
    let parent = Pattern::new("/a/**/b").unwrap();
    assert!(parent.matches("/a/b"));
    assert!(parent.matches("/a/*/b"));
    assert!(parent.matches("/a/**/b"));
    assert!(parent.matches("/a/c/*/b"));
    assert!(parent.matches("/a/c/**/b"));
    assert!(!parent.matches("/"));
    assert!(!parent.matches("/a"));
    assert!(!parent.matches("/a/"));
    assert!(!parent.matches("/a/c"));
}

#[test]
fn uuid_simple_as_string() {
    let uuid = Uuid::new_v4();
    let simple_str = uuid.simple().to_string();
    assert!(uuid == Uuid::from_str(&simple_str).unwrap())
}
