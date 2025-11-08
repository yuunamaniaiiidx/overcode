use crate::config::IgnorePattern;
use std::path::PathBuf;

#[test]
fn test_ignore_pattern_directory() {
    let pattern = IgnorePattern::new("node_modules");
    let root = PathBuf::from("/project");
    let path = PathBuf::from("/project/src/node_modules/package.json");
    
    assert!(pattern.matches(&path, &root));
}

