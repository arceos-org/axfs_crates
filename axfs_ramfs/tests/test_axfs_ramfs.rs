// tests/test_axfs_ramfs.rs

use axfs_ramfs::*;
use axfs_vfs::{VfsError, VfsNodeType, VfsOps};
use std::sync::Arc;

/// Helper function: Creates a standard filesystem structure for multiple tests.
fn setup_test_fs() -> RamFileSystem {
    let ramfs = RamFileSystem::new();
    let root = ramfs.root_dir();

    // .clone() before calling methods that consume ownership.
    root.clone().create("file.txt", VfsNodeType::File).unwrap();
    root.clone().create("dir1", VfsNodeType::Dir).unwrap();
    let dir1 = root.clone().lookup("dir1").unwrap();
    dir1.clone().create("dir2", VfsNodeType::Dir).unwrap();
    let dir2 = dir1.clone().lookup("dir2").unwrap();
    dir2.create("nested_file.txt", VfsNodeType::File).unwrap();
    ramfs
}

// =================================================================
// Test Case 1: Edge cases for path resolution and lookup.
// =================================================================
#[test]
fn test_path_manipulation_and_lookup() {
    println!("\n--- Test: Path Resolution and Lookup ---");
    let ramfs = setup_test_fs();
    let root = ramfs.root_dir();

    // Clone before each call to .lookup() or other methods that consume self.
    let dir1 = root.clone().lookup("dir1").unwrap();
    let dir2 = root.clone().lookup("dir1/dir2").unwrap();

    // 1. Test with redundant slashes.
    println!("Testing: Redundant slashes");
    let found_dir2 = root.clone().lookup("///dir1//dir2/").unwrap();
    assert!(
        Arc::ptr_eq(&dir2, &found_dir2),
        "Path resolution with extra slashes should be correct"
    );

    // 2. Test with current directory indicator '.'.
    println!("Testing: Current directory indicator '.'");
    let found_dir2_with_dots = root.clone().lookup("./dir1/./dir2").unwrap();
    assert!(
        Arc::ptr_eq(&dir2, &found_dir2_with_dots),
        "Path resolution with '.' should be correct"
    );

    // 3. Test with parent directory indicator '..'.
    println!("Testing: Parent directory indicator '..'");
    let found_dir1 = dir2.clone().lookup("..").unwrap();
    assert!(
        Arc::ptr_eq(&dir1, &found_dir1),
        "'..' should correctly return the parent directory"
    );

    // 4. Test complex '..' combination paths.
    let found_root = dir2.clone().lookup("../..").unwrap();
    assert!(
        Arc::ptr_eq(&root, &found_root),
        "'../..' should return the root directory"
    );

    // 5. Test lookup from a file node (should fail).
    println!("Testing: Lookup on a file node");
    let file = root.clone().lookup("file.txt").unwrap();
    assert_eq!(
        file.lookup("any").err(), // 'file' is moved here, but it's okay as it's not used afterward.
        Some(VfsError::NotADirectory),
        "Performing lookup on a file should return NotADirectory"
    );

    println!("--- Path resolution tests passed ---");
}

// =================================================================
// Test Case 2: Edge cases for file I/O.
// =================================================================
#[test]
fn test_io_edge_cases() {
    println!("\n--- Test: File I/O Edge Cases ---");
    let ramfs = RamFileSystem::new();
    let root = ramfs.root_dir();
    root.clone().create("data.dat", VfsNodeType::File).unwrap();
    let file = root.clone().lookup("data.dat").unwrap();

    let mut buf = [0u8; 32];

    // 1. Write past the end of the file.
    println!("Testing: Writing past the end of the file");
    file.clone().write_at(10, b"world").unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 15);

    let bytes_read = file.clone().read_at(0, &mut buf).unwrap();
    assert_eq!(bytes_read, 15);
    assert_eq!(&buf[..bytes_read], b"\0\0\0\0\0\0\0\0\0\0world");

    // 2. Read past the end of the file.
    println!("Testing: Reading past the end of the file");
    let mut small_buf = [0u8; 8];
    let bytes_read = file.clone().read_at(10, &mut small_buf).unwrap();
    assert_eq!(bytes_read, 5);
    assert_eq!(&small_buf[..bytes_read], b"world");

    // 3. Truncate file to make it larger.
    println!("Testing: Truncating file to make it larger");
    file.clone().truncate(20).unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 20);
    let bytes_read = file.clone().read_at(0, &mut buf).unwrap();
    assert_eq!(bytes_read, 20);
    assert_eq!(&buf[..15], b"\0\0\0\0\0\0\0\0\0\0world");
    assert_eq!(&buf[15..20], &[0, 0, 0, 0, 0]);

    // 4. Truncate file to zero.
    println!("Testing: Truncating file to zero");
    file.clone().truncate(0).unwrap();
    assert_eq!(file.get_attr().unwrap().size(), 0);
    let bytes_read = file.read_at(0, &mut buf).unwrap(); // Last use, no clone needed.
    assert_eq!(
        bytes_read, 0,
        "Reading from an empty file should return 0 bytes"
    );

    println!("--- File I/O edge case tests passed ---");
}

// =================================================================
// Test Case 3: Verification of various error conditions.
// =================================================================
#[test]
fn test_error_conditions() {
    println!("\n--- Test: Error Conditions ---");
    let ramfs = setup_test_fs();
    let root = ramfs.root_dir();
    let dir1 = root.clone().lookup("dir1").unwrap();
    let file = root.clone().lookup("file.txt").unwrap();

    // 1. AlreadyExists error.
    println!("Testing: AlreadyExists error");
    assert_eq!(
        root.clone().create("file.txt", VfsNodeType::File).err(),
        Some(VfsError::AlreadyExists)
    );
    assert_eq!(
        root.clone().create("dir1", VfsNodeType::Dir).err(),
        Some(VfsError::AlreadyExists)
    );

    // 2. DirectoryNotEmpty error.
    println!("Testing: DirectoryNotEmpty error");
    assert_eq!(
        root.clone().remove("dir1").err(),
        Some(VfsError::DirectoryNotEmpty)
    );

    // 3. NotADirectory error.
    println!("Testing: NotADirectory error");
    assert_eq!(
        file.clone().create("anything", VfsNodeType::File).err(),
        Some(VfsError::NotADirectory)
    );

    // 4. IsADirectory error.
    println!("Testing: IsADirectory error");
    let mut buf = [0u8; 1];
    assert_eq!(
        dir1.clone().read_at(0, &mut buf).err(),
        Some(VfsError::IsADirectory)
    );
    assert_eq!(
        dir1.write_at(0, &buf).err(), // Last use of dir1, no clone needed.
        Some(VfsError::IsADirectory)
    );

    // 5. InvalidInput error.
    println!("Testing: InvalidInput error");
    assert_eq!(root.clone().remove(".").err(), Some(VfsError::InvalidInput));
    assert_eq!(
        root.clone().remove("..").err(),
        Some(VfsError::InvalidInput)
    );

    // 6. NotFound error.
    println!("Testing: NotFound error");
    assert_eq!(
        root.clone().lookup("no-such-file").err(),
        Some(VfsError::NotFound)
    );
    assert_eq!(root.remove("no-such-file").err(), Some(VfsError::NotFound)); // Last use of root.

    println!("--- Error condition tests passed ---");
}
