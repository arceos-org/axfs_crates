// tests/test_axfs_dexfs.rs

// 1. Import the crate itself.
use axfs_devfs;

// 2. Import other dependencies required for testing.
use std::sync::Arc;
use axfs_vfs::{VfsError, VfsNodeType, VfsOps};

// Test Scene 1: Basic filesystem construction and operations on device nodes in the root directory.
#[test]
fn test_root_device_operations() {
    // === Setup Phase ===
    let devfs = axfs_devfs::DeviceFileSystem::new();
    devfs.add("null", Arc::new(axfs_devfs::NullDev));
    devfs.add("zero", Arc::new(axfs_devfs::ZeroDev));

    let root = devfs.root_dir();

    // === Test Phase ===

    // Verify root directory attributes.
    assert!(root.get_attr().unwrap().is_dir());
    assert_eq!(root.get_attr().unwrap().file_type(), VfsNodeType::Dir);

    // Verify lookup of a non-existent node.
    // Use clone() to move a copy, not root itself.
    assert_eq!(root.clone().lookup("non_existent_file").err(), Some(VfsError::NotFound));

    // --- Test /null device ---
    // Clone again.
    let null_dev = root.clone().lookup("null").expect("Failed to lookup /null");
    assert_eq!(null_dev.get_attr().unwrap().file_type(), VfsNodeType::CharDevice);

    let mut buffer = [42u8; 16];
    assert_eq!(null_dev.read_at(0, &mut buffer).unwrap(), 0);
    assert_eq!(buffer, [42u8; 16]);
    assert_eq!(null_dev.write_at(0, &[1, 2, 3]).unwrap(), 3);

    // --- Test /zero device ---
    // This is the last use of root, so we can move it directly without cloning.
    let zero_dev = root.lookup("zero").expect("Failed to lookup /zero");
    assert_eq!(zero_dev.get_attr().unwrap().file_type(), VfsNodeType::CharDevice);
    assert_eq!(zero_dev.read_at(0, &mut buffer).unwrap(), buffer.len());
    assert_eq!(buffer, [0u8; 16]);
}

// Test Scene 2: Subdirectory creation, nested lookups, and path traversal.
#[test]
fn test_subdirectory_and_path_traversal() {
    // === Setup Phase ===
    let devfs = axfs_devfs::DeviceFileSystem::new();
    devfs.add("null", Arc::new(axfs_devfs::NullDev));
    let dir_sub = devfs.mkdir("sub");
    dir_sub.add("zero", Arc::new(axfs_devfs::ZeroDev));
    let dir_sub_nested = dir_sub.mkdir("nested");
    dir_sub_nested.add("another_null", Arc::new(axfs_devfs::NullDev));

    let root = devfs.root_dir();

    // === Test Phase ===

    // --- Basic path lookup ---
    let sub_dir = root.clone().lookup("sub").unwrap();
    assert!(sub_dir.get_attr().unwrap().is_dir());

    // Clone sub_dir to use it for lookup.
    let zero_in_sub = sub_dir.clone().lookup("zero").unwrap();
    assert_eq!(zero_in_sub.get_attr().unwrap().file_type(), VfsNodeType::CharDevice);

    // --- Test complex and redundant paths ---
    let same_zero_in_sub = root.clone().lookup("sub/./zero").unwrap();
    assert!(Arc::ptr_eq(&zero_in_sub, &same_zero_in_sub));

    // --- Test '..' parent directory lookup ---
    let sub_from_nested = root.clone().lookup("sub/nested/..").unwrap();
    assert!(Arc::ptr_eq(&sub_dir, &sub_from_nested));

    // --- Test deep lookup from the root directory ---
    let deep_null = root.clone().lookup("sub/nested/another_null").unwrap();
    assert_eq!(deep_null.get_attr().unwrap().file_type(), VfsNodeType::CharDevice);

    // --- Verify parent node relationship ---
    let nested_dir = root.clone().lookup("sub/nested").unwrap();
    let parent_of_nested = nested_dir.parent().expect("nested dir should have a parent");
    assert!(Arc::ptr_eq(&parent_of_nested, &sub_dir));
}

// Test Scene 3: Error handling.
#[test]
fn test_error_conditions() {
    let devfs = axfs_devfs::DeviceFileSystem::new();
    devfs.add("null", Arc::new(axfs_devfs::NullDev));
    let root = devfs.root_dir();

    // Perform directory operations on a file node.
    assert_eq!(root.clone().lookup("null/some_file").err(), Some(VfsError::NotADirectory));

    // Perform file operations on a directory node.
    assert_eq!(root.read_at(0, &mut [0; 1]).err(), Some(VfsError::IsADirectory));
    assert_eq!(root.write_at(0, &[0; 1]).err(), Some(VfsError::IsADirectory));
    
    // devfs does not support runtime creation/deletion; verify it returns a permission error.
    assert_eq!(root.clone().create("new_file", VfsNodeType::File).err(), Some(VfsError::PermissionDenied));
    // This is the last use of root, so we can move it directly without cloning.
    assert_eq!(root.remove("null").err(), Some(VfsError::PermissionDenied));
}
