use crate::config::IgnorePattern;
use std::path::PathBuf;

#[test]
fn test_ignore_pattern_wildcard() {
    let pattern = IgnorePattern::new("*.log");
    let root = PathBuf::from("/project");
    let path = PathBuf::from("/project/src/app.log");
    
    assert!(pattern.matches(&path, &root));
}

