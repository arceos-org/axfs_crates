// In tests/test_axfs_ramfs.rs

use std::sync::Arc;

// FIX 1: Removed `RamDir` as it's not a public type.
use axfs_ramfs::RamFileSystem;
// FIX 2: Removed `VfsNodeOps` as it's an unused import according to the compiler.
use axfs_vfs::{VfsError, VfsNodeType, VfsOps, VfsResult};

// =================================================================
// Helper Functions
// =================================================================

/// Helper: Creates a standard filesystem structure for multiple tests.
fn setup_complex_fs() -> RamFileSystem {
    let ramfs = RamFileSystem::new();
    let root = ramfs.root_dir();

    root.clone().create("file.txt", VfsNodeType::File).unwrap();
    root.clone().create("dir1", VfsNodeType::Dir).unwrap();

    let dir1 = root.lookup("dir1").unwrap();
    dir1.clone().create("dir2", VfsNodeType::Dir).unwrap();

    let dir2 = dir1.lookup("dir2").unwrap();
    dir2.create("nested_file.txt", VfsNodeType::File).unwrap();

    ramfs
}

/// Helper: Tests basic filesystem operations.
fn run_basic_ops_tests(ramfs: &RamFileSystem) -> VfsResult<()> {
    const N: usize = 32;
    const N_HALF: usize = N / 2;
    let mut buf = [1; N];

    let root = ramfs.root_dir();
    let f1 = root.lookup("f1")?;

    assert_eq!(f1.clone().get_attr()?.file_type(), VfsNodeType::File);
    assert_eq!(f1.clone().get_attr()?.size(), 0);

    assert_eq!(f1.clone().write_at(N_HALF as u64, &buf[..N_HALF])?, N_HALF);
    assert_eq!(f1.clone().get_attr()?.size(), N as u64);
    assert_eq!(f1.read_at(0, &mut buf)?, N);
    assert_eq!(buf[..N_HALF], [0; N_HALF]);
    assert_eq!(buf[N_HALF..], [1; N_HALF]);
    Ok(())
}

/// Helper: Tests parent directory and path resolution.
fn run_parent_and_path_tests(ramfs: &RamFileSystem) -> VfsResult<()> {
    let root = ramfs.root_dir();
    let foo = root.clone().lookup("foo")?;
    let bar = foo.clone().lookup("bar")?;

    assert!(bar.parent().is_some());
    assert!(Arc::ptr_eq(&bar.parent().unwrap(), &foo));

    assert!(Arc::ptr_eq(&bar.clone().lookup("..")?, &foo));
    assert!(Arc::ptr_eq(&foo.clone().lookup("..")?, &root));

    assert!(Arc::ptr_eq(
        &root.clone().lookup("foo/bar/..")?,
        &root.clone().lookup("foo")?,
    ));
    assert!(Arc::ptr_eq(
        &root.clone().lookup("./foo/../foo/bar/f4")?,
        &root.lookup("foo/bar/f4")?,
    ));
    Ok(())
}

// =================================================================
// Test Scenarios
// =================================================================

#[test]
fn test_full_lifecycle() {
    let ramfs = RamFileSystem::new();
    let root = ramfs.root_dir();
    root.clone().create("f1", VfsNodeType::File).unwrap();
    root.clone().create("f2", VfsNodeType::File).unwrap();
    root.clone().create("foo", VfsNodeType::Dir).unwrap();

    let foo = root.clone().lookup("foo").unwrap();
    foo.clone().create("f3", VfsNodeType::File).unwrap();
    foo.clone().create("bar", VfsNodeType::Dir).unwrap();

    let bar = foo.lookup("bar").unwrap();
    bar.create("f4", VfsNodeType::File).unwrap();

    // This call works because `root_dir_node` on RamFileSystem and its `get_entries`
    // method are public, without needing to name the concrete type.
    let mut entries = ramfs.root_dir_node().get_entries();
    entries.sort();
    assert_eq!(entries, ["f1", "f2", "foo"]);

    run_basic_ops_tests(&ramfs).expect("Basic ops tests failed");
    run_parent_and_path_tests(&ramfs).expect("Parent and path tests failed");

    assert_eq!(root.clone().remove("f1"), Ok(()));
    assert_eq!(root.clone().remove("//f2"), Ok(()));
    assert_eq!(
        root.clone().remove("foo").err(),
        Some(VfsError::DirectoryNotEmpty)
    );
    assert_eq!(root.clone().remove("foo/bar/f4"), Ok(()));
    assert_eq!(root.clone().remove("foo/bar"), Ok(()));
    assert_eq!(root.clone().remove("./foo//.//f3"), Ok(()));
    assert_eq!(root.clone().remove("./foo"), Ok(()));
    assert!(ramfs.root_dir_node().get_entries().is_empty());
}

#[test]
fn test_path_manipulation_and_lookup() {
    let ramfs = setup_complex_fs();
    let root = ramfs.root_dir();
    let dir1 = root.clone().lookup("dir1").unwrap();
    let dir2 = root.clone().lookup("dir1/dir2").unwrap();

    assert!(Arc::ptr_eq(
        &root.clone().lookup("///dir1//./dir2/..").unwrap(),
        &dir1
    ));
    assert!(Arc::ptr_eq(&dir2.lookup("..").unwrap(), &dir1));

    let file = root.lookup("file.txt").unwrap();
    assert_eq!(file.lookup("any").err(), Some(VfsError::NotADirectory));
}

#[test]
fn test_io_edge_cases() {
    let ramfs = RamFileSystem::new();
    let root = ramfs.root_dir();
    root.clone().create("data.dat", VfsNodeType::File).unwrap();
    let file = root.lookup("data.dat").unwrap();
    let mut buf = [0u8; 32];

    file.clone().write_at(5, b"hello").unwrap();
    assert_eq!(file.clone().get_attr().unwrap().size(), 10);
    file.clone().read_at(0, &mut buf[..10]).unwrap();
    assert_eq!(&buf[..10], b"\0\0\0\0\0hello");

    file.clone().truncate(5).unwrap();
    assert_eq!(file.clone().get_attr().unwrap().size(), 5);

    file.clone().truncate(15).unwrap();
    assert_eq!(file.clone().get_attr().unwrap().size(), 15);
    let bytes_read = file.read_at(0, &mut buf).unwrap();
    assert_eq!(bytes_read, 15);
    // After truncating to 5, the "hello" part was cut off.
    // Expanding to 15 should result in all zeros.
    assert_eq!(&buf[..bytes_read], &[0; 15]);
}

#[test]
fn test_error_conditions() {
    let ramfs = setup_complex_fs();
    let root = ramfs.root_dir();
    let dir1 = root.clone().lookup("dir1").unwrap();

    assert_eq!(
        root.clone().create("dir1", VfsNodeType::Dir).err(),
        Some(VfsError::AlreadyExists)
    );
    assert_eq!(
        root.clone().remove("dir1").err(),
        Some(VfsError::DirectoryNotEmpty)
    );
    assert_eq!(
        dir1.clone().read_at(0, &mut [0]).err(),
        Some(VfsError::IsADirectory)
    );
    assert_eq!(
        dir1.clone().write_at(0, &[0]).err(),
        Some(VfsError::IsADirectory)
    );
    assert_eq!(dir1.clone().remove(".").err(), Some(VfsError::InvalidInput));
    assert_eq!(dir1.remove("..").err(), Some(VfsError::InvalidInput));
    assert_eq!(root.lookup("no-such-file").err(), Some(VfsError::NotFound));
}
