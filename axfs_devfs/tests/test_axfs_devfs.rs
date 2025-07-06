// In tests/test_axfs_devfs.rs

// 1. Import dependencies as if this were an external crate.
//    - `axfs_devfs` is the name of the crate we are testing.
//    - `axfs_vfs` is a regular dependency.
use axfs_devfs::{DeviceFileSystem, NullDev, ZeroDev};
use axfs_vfs::{VfsError, VfsNodeType, VfsOps};
use std::sync::Arc;

// 2. Helper function to build a complex filesystem instance for use in multiple test scenarios.
//    .
//    ├── foo
//    │   ├── bar
//    │   │   └── f1 (null)
//    │   └── f2 (zero)
//    ├── null
//    └── zero
fn setup_complex_devfs() -> DeviceFileSystem {
    let devfs = DeviceFileSystem::new();
    devfs.add("null", Arc::new(NullDev));
    devfs.add("zero", Arc::new(ZeroDev));

    let dir_foo = devfs.mkdir("foo");
    dir_foo.add("f2", Arc::new(ZeroDev));
    let dir_bar = dir_foo.mkdir("bar");
    dir_bar.add("f1", Arc::new(NullDev));
    devfs
}

// Scene 1: Test basic I/O operations and attributes of device nodes.
#[test]
fn test_root_device_operations() {
    let devfs = setup_complex_devfs();
    let root = devfs.root_dir();

    // --- Verify root directory attributes ---
    assert!(root.get_attr().unwrap().is_dir());
    assert_eq!(root.get_attr().unwrap().file_type(), VfsNodeType::Dir);

    // --- Verify lookup of a non-existent node ---
    assert_eq!(
        root.clone().lookup("non_existent_file").err(),
        Some(VfsError::NotFound)
    );

    // --- Test the /null device ---
    let null_dev = root.clone().lookup("null").unwrap();
    assert_eq!(
        null_dev.get_attr().unwrap().file_type(),
        VfsNodeType::CharDevice
    );
    assert!(!null_dev.get_attr().unwrap().is_dir());
    assert_eq!(null_dev.get_attr().unwrap().size(), 0);

    let mut buffer = [42u8; 16];
    assert_eq!(
        null_dev.read_at(0, &mut buffer).unwrap(),
        0,
        "Reading from null should return 0 bytes"
    );
    assert_eq!(
        buffer, [42u8; 16],
        "Buffer should be unchanged after reading from null"
    );
    assert_eq!(
        null_dev.write_at(0, &[1, 2, 3]).unwrap(),
        3,
        "Writing to null should succeed"
    );

    // --- Test the /zero device ---
    let zero_dev = root.lookup("zero").unwrap();
    assert_eq!(
        zero_dev.get_attr().unwrap().file_type(),
        VfsNodeType::CharDevice
    );

    let mut buffer_for_zero = [42u8; 16];
    assert_eq!(
        zero_dev.read_at(0, &mut buffer_for_zero).unwrap(),
        buffer_for_zero.len()
    );
    assert_eq!(
        buffer_for_zero, [0u8; 16],
        "Buffer should be filled with zeros after reading from zero"
    );
    assert_eq!(
        zero_dev.write_at(0, &[1, 2, 3]).unwrap(),
        3,
        "Writing to zero should succeed"
    );
}

// Scene 2: Test complex path traversal, parent directory lookup, and node identity.
#[test]
fn test_complex_path_traversal_and_parent_lookup() {
    let devfs = setup_complex_devfs();
    let root = devfs.root_dir();
    let foo = root.clone().lookup("foo").unwrap();

    // --- Perform strict node identity checks using Arc::ptr_eq ---
    assert!(Arc::ptr_eq(
        &foo.clone().lookup("f2").unwrap(), // Note: lookup from `foo`, not `/f2`
        &root.clone().lookup(".//./foo///f2").unwrap(),
    ));
    assert!(Arc::ptr_eq(
        &root.clone().lookup("foo/..").unwrap(), // Note: `lookup` handles `..`
        &root.clone().lookup(".//./foo/././bar/../..").unwrap(),
    ));
    assert!(Arc::ptr_eq(
        &root
            .clone()
            .lookup("././/foo//./../foo//bar///..//././")
            .unwrap(),
        &root.clone().lookup(".//./foo/").unwrap(),
    ));
    assert!(Arc::ptr_eq(
        &root.clone().lookup("///foo//bar///../f2").unwrap(),
        &root.clone().lookup("foo/.//f2").unwrap(),
    ));

    // --- In-depth test of parent directory lookup (`..` and .parent()) ---
    assert!(root.parent().is_none(), "Root should not have a parent");

    let bar = root.clone().lookup("foo/bar").unwrap();
    let parent_of_bar = bar.parent().unwrap();
    assert!(
        Arc::ptr_eq(&parent_of_bar, &foo),
        "Parent of 'bar' should be 'foo'"
    );

    let foo_from_bar = bar.lookup("..").unwrap();
    assert!(
        Arc::ptr_eq(&foo_from_bar, &foo),
        "Lookup of '..' from 'bar' should return 'foo'"
    );
}

// Scene 3: Verify various expected error conditions.
#[test]
fn test_error_conditions() {
    let devfs = setup_complex_devfs();
    let root = devfs.root_dir();
    let null_dev = root.clone().lookup("null").unwrap();
    let foo_dir = root.clone().lookup("foo").unwrap();

    // --- Perform directory operations on a file node ---
    assert_eq!(
        null_dev.lookup("any").err(),
        Some(VfsError::NotADirectory),
        "Lookup on a file node should fail"
    );
    assert_eq!(
        root.clone().lookup("null/some_file").err(),
        Some(VfsError::NotADirectory),
        "Path traversal through a file node should fail"
    );

    // --- Perform file operations on a directory node ---
    assert_eq!(
        foo_dir.read_at(0, &mut [0; 1]).err(),
        Some(VfsError::IsADirectory),
        "Reading a directory should fail"
    );
    assert_eq!(
        foo_dir.write_at(0, &[0; 1]).err(),
        Some(VfsError::IsADirectory),
        "Writing to a directory should fail"
    );

    // --- Verify the read-only nature of devfs (no runtime modifications) ---
    assert_eq!(
        root.clone().create("new_file", VfsNodeType::File).err(),
        Some(VfsError::PermissionDenied),
        "Creating files at runtime should be denied"
    );
    assert_eq!(
        root.remove("null").err(),
        Some(VfsError::PermissionDenied),
        "Removing nodes at runtime should be denied"
    );
}
