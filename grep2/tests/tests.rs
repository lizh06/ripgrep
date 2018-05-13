extern crate failure;
extern crate grep2;
extern crate regex;

use grep2::{Captures, Matcher};
use regex::bytes::Regex;

use util::{RegexMatcher, RegexMatcherNoCaps};

mod util;

fn matcher(pattern: &str) -> RegexMatcher {
    RegexMatcher::new(Regex::new(pattern).unwrap())
}

fn matcher_no_caps(pattern: &str) -> RegexMatcherNoCaps {
    RegexMatcherNoCaps(Regex::new(pattern).unwrap())
}

#[test]
fn find() {
    let matcher = matcher(r"(\w+)\s+(\w+)");
    assert_eq!(matcher.find(b" homer simpson ").unwrap(), Some((1, 14)));
}

#[test]
fn find_iter() {
    let matcher = matcher(r"(\w+)\s+(\w+)");
    let mut matches = vec![];
    matcher.find_iter(b"aa bb cc dd", |s, e| {
        matches.push((s, e));
        true
    }).unwrap();
    assert_eq!(matches, vec![(0, 5), (6, 11)]);

    // Test that find_iter respects short circuiting.
    matches.clear();
    matcher.find_iter(b"aa bb cc dd", |s, e| {
        matches.push((s, e));
        false
    }).unwrap();
    assert_eq!(matches, vec![(0, 5)]);
}

#[test]
fn shortest_match() {
    let matcher = matcher(r"a+");
    // This tests that the default impl isn't doing anything smart, and simply
    // defers to `find`.
    assert_eq!(matcher.shortest_match(b"aaa").unwrap(), Some(3));
    // The actual underlying regex is smarter.
    assert_eq!(matcher.re.shortest_match(b"aaa"), Some(1));
}

#[test]
fn captures() {
    let matcher = matcher(r"(?P<a>\w+)\s+(?P<b>\w+)");
    assert_eq!(matcher.capture_count(), 3);
    assert_eq!(matcher.capture_index("a"), Some(1));
    assert_eq!(matcher.capture_index("b"), Some(2));
    assert_eq!(matcher.capture_index("nada"), None);

    let mut caps = matcher.new_captures().unwrap();
    assert!(matcher.captures(b" homer simpson ", &mut caps).unwrap());
    assert_eq!(caps.get(0), Some((1, 14)));
    assert_eq!(caps.get(1), Some((1, 6)));
    assert_eq!(caps.get(2), Some((7, 14)));
}

#[test]
fn captures_iter() {
    let matcher = matcher(r"(?P<a>\w+)\s+(?P<b>\w+)");
    let mut caps = matcher.new_captures().unwrap();
    let mut matches = vec![];
    matcher.captures_iter(b"aa bb cc dd", &mut caps, |caps| {
        matches.push(caps.get(0).unwrap());
        matches.push(caps.get(1).unwrap());
        matches.push(caps.get(2).unwrap());
        true
    }).unwrap();
    assert_eq!(matches, vec![
        (0, 5), (0, 2), (3, 5),
        (6, 11), (6, 8), (9, 11),
    ]);

    // Test that captures_iter respects short circuiting.
    matches.clear();
    matcher.captures_iter(b"aa bb cc dd", &mut caps, |caps| {
        matches.push(caps.get(0).unwrap());
        matches.push(caps.get(1).unwrap());
        matches.push(caps.get(2).unwrap());
        false
    }).unwrap();
    assert_eq!(matches, vec![
        (0, 5), (0, 2), (3, 5),
    ]);
}

// Test that our default impls for capturing are correct. Namely, when
// capturing isn't supported by the underlying matcher, then all of the
// various capturing related APIs fail fast.
#[test]
fn no_captures() {
    let matcher = matcher_no_caps(r"(?P<a>\w+)\s+(?P<b>\w+)");
    assert_eq!(matcher.capture_count(), 0);
    assert_eq!(matcher.capture_index("a"), None);
    assert_eq!(matcher.capture_index("b"), None);
    assert_eq!(matcher.capture_index("nada"), None);

    let mut caps = matcher.new_captures().unwrap();
    assert!(!matcher.captures(b"homer simpson", &mut caps).unwrap());

    let mut called = false;
    matcher.captures_iter(b"homer simpson", &mut caps, |_| {
        called = true;
        true
    }).unwrap();
    assert!(!called);
}

#[test]
fn replace() {
    let matcher = matcher(r"(\w+)\s+(\w+)");
    let mut dst = vec![];
    matcher.replace(b"aa bb cc dd", &mut dst, |_, _, dst| {
        dst.push(b'z');
        true
    }).unwrap();
    assert_eq!(dst, b"z z");

    // Test that replacements respect short circuiting.
    dst.clear();
    matcher.replace(b"aa bb cc dd", &mut dst, |_, _, dst| {
        dst.push(b'z');
        false
    }).unwrap();
    assert_eq!(dst, b"z cc dd");
}

#[test]
fn replace_with_captures() {
    let matcher = matcher(r"(\w+)\s+(\w+)");
    let haystack = b"aa bb cc dd";
    let mut caps = matcher.new_captures().unwrap();
    let mut dst = vec![];
    matcher.replace_with_captures(haystack, &mut caps, &mut dst, |caps, dst| {
        caps.interpolate(
            |name| matcher.capture_index(name),
            haystack,
            b"$2 $1",
            dst,
        );
        true
    }).unwrap();
    assert_eq!(dst, b"bb aa dd cc");

    // Test that replacements respect short circuiting.
    dst.clear();
    matcher.replace_with_captures(haystack, &mut caps, &mut dst, |caps, dst| {
        caps.interpolate(
            |name| matcher.capture_index(name),
            haystack,
            b"$2 $1",
            dst,
        );
        false
    }).unwrap();
    assert_eq!(dst, b"bb aa cc dd");
}
