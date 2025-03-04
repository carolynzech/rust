#![cfg(windows)]

/// Attempting to delete a running binary should return an error on Windows.
#[test]
<<<<<<< HEAD
=======
#[cfg_attr(miri, ignore)] // `remove_file` does not work in Miri on Windows
>>>>>>> 98bc9a8d6d5d9e482b1f99face354f6b582b125c
fn win_delete_self() {
    let path = std::env::current_exe().unwrap();
    assert!(std::fs::remove_file(path).is_err());
}
