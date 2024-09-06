use tempfile;
use sangit;
use std::fs;

#[test]
fn should_init() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str();

    sangit::init(path).unwrap();

    let path = path.unwrap();

    assert!(fs::metadata(format!("{}/.git", path)).unwrap().is_dir());
    assert!(fs::metadata(format!("{}/.git/objects", path)).unwrap().is_dir());
    assert!(fs::metadata(format!("{}/.git/refs", path)).unwrap().is_dir());

    let head = fs::read_to_string(format!("{}/.git/HEAD", path)).unwrap();
    assert!(head.trim() == "ref: refs/heads/main");
}

fn should_cat_file() {
    // TODO: use mock-io to mock system files
}
