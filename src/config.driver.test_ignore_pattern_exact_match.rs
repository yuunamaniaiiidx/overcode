use crate::config::IgnorePattern;
use std::path::PathBuf;

#[test]
fn test_ignore_pattern_exact_match() {
    let pattern = IgnorePattern::new(".git");
    let root = PathBuf::from("/project");
    let path = PathBuf::from("/project/.git/config");
    
    assert!(pattern.matches(&path, &root));
}

