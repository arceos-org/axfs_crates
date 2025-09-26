# API 参考索引

<cite>
**Referenced Files in This Document **   
- [axfs_vfs/src/lib.rs](file://axfs_vfs/src/lib.rs)
- [axfs_vfs/src/structs.rs](file://axfs_vfs/src/structs.rs)
- [axfs_vfs/src/macros.rs](file://axfs_vfs/src/macros.rs)
- [axfs_devfs/src/lib.rs](file://axfs_devfs/src/lib.rs)
- [axfs_devfs/src/null.rs](file://axfs_devfs/src/null.rs)
- [axfs_devfs/src/zero.rs](file://axfs_devfs/src/zero.rs)
- [axfs_devfs/src/urandom.rs](file://axfs_devfs/src/urandom.rs)
- [axfs_devfs/src/dir.rs](file://axfs_devfs/src/dir.rs)
- [axfs_ramfs/src/lib.rs](file://axfs_ramfs/src/lib.rs)
- [axfs_ramfs/src/file.rs](file://axfs_ramfs/src/file.rs)
- [axfs_ramfs/src/dir.rs](file://axfs_ramfs/src/dir.rs)
</cite>

## 目录
1. [axfs_vfs 核心接口与数据结构](#axfs_vfs-核心接口与数据结构)  
   - [VfsOps](#vfsops)  
   - [VfsNodeOps](#vfsnodeops)  
   - [VfsNodeAttr](#vfsnodeattr)  
   - [VfsNodePerm](#vfsnodeperm)  
   - [VfsNodeType](#vfsnodetype)  
2. [axfs_devfs 设备实现](#axfs_devfs-设备实现)  
   - [DeviceFileSystem](#devicefilesystem)  
   - [NullDev](#nulldev)  
   - [ZeroDev](#zerodev)  
   - [UrandomDev](#urandomdev)  
3. [axfs_ramfs 可变节点类型](#axfs_ramfs-可变节点类型)  
   - [RamFileSystem](#ramfilesystem)  
   - [FileNode](#filenode)  
   - [DirNode](#dirnode)  
4. [宏说明](#宏说明)  
   - [impl_vfs_dir_default!](#impl_vfs_dir_default)  
   - [impl_vfs_non_dir_default!](#impl_vfs_non_dir_default)

## axfs_vfs 核心接口与数据结构

### VfsOps
[VfsOps](file://axfs_vfs/src/lib.rs#L65-L90) 是文件系统操作的核心 trait，定义了文件系统的生命周期和属性查询方法。

**方法列表：**
- `mount(&self, _path: &str, _mount_point: VfsNodeRef) -> VfsResult`  
  文件系统挂载时调用。默认实现为空。
- `umount(&self) -> VfsResult`  
  文件系统卸载时调用。默认实现为空。
- `format(&self) -> VfsResult`  
  格式化文件系统。默认返回 `Unsupported` 错误。
- `statfs(&self) -> VfsResult<FileSystemInfo>`  
  获取文件系统属性。默认返回 `Unsupported` 错误。
- `root_dir(&self) -> VfsNodeRef`  
  获取文件系统的根目录节点（必须由具体类型实现）。

**Section sources**
- [axfs_vfs/src/lib.rs](file://axfs_vfs/src/lib.rs#L65-L90)

### VfsNodeOps
[VfsNodeOps](file://axfs_vfs/src/lib.rs#L93-L177) 是节点（文件/目录）操作的核心 trait，定义了对文件或目录的各种操作。

**通用操作（文件与目录均支持）：**
- `open(&self) -> VfsResult`  
  节点打开时调用。默认实现为空。
- `release(&self) -> VfsResult`  
  节点关闭时调用。默认实现为空。
- `get_attr(&self) -> VfsResult<VfsNodeAttr>`  
  获取节点属性。默认返回 `Unsupported` 错误。

**文件专有操作：**
- `read_at(&self, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize>`  
  从指定偏移读取数据。默认返回 `InvalidInput` 错误。
- `write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize>`  
  向指定偏移写入数据。默认返回 `InvalidInput` 错误。
- `fsync(&self) -> VfsResult`  
  同步文件数据到存储。默认返回 `InvalidInput` 错误。
- `truncate(&self, _size: u64) -> VfsResult`  
  截断文件至指定大小。默认返回 `InvalidInput` 错误。

**目录专有操作：**
- `parent(&self) -> Option<VfsNodeRef>`  
  获取父目录。默认返回 `None`。
- `lookup(self: Arc<Self>, _path: &str) -> VfsResult<VfsNodeRef>`  
  在目录中查找路径对应的节点。默认返回 `Unsupported` 错误。
- `create(&self, _path: &str, _ty: VfsNodeType) -> VfsResult`  
  创建新节点。默认返回 `Unsupported` 错误。
- `remove(&self, _path: &str) -> VfsResult`  
  删除指定路径的节点。默认返回 `Unsupported` 错误。
- `read_dir(&self, _start_idx: usize, _dirents: &mut [VfsDirEntry]) -> VfsResult<usize>`  
  读取目录条目。默认返回 `Unsupported` 错误。
- `rename(&self, _src_path: &str, _dst_path: &str) -> VfsResult`  
  重命名或移动节点。默认返回 `Unsupported` 错误。
- `as_any(&self) -> &dyn core::any::Any`  
  将自身转换为 `Any` 类型以支持向下转型。默认未实现。

**Section sources**
- [axfs_vfs/src/lib.rs](file://axfs_vfs/src/lib.rs#L93-L177)

### VfsNodeAttr
[VfsNodeAttr](file://axfs_vfs/src/structs.rs#L13-L38) 表示节点（文件/目录）的属性。

**字段：**
- `mode: VfsNodePerm` - 文件权限模式
- `ty: VfsNodeType` - 文件类型
- `size: u64` - 总大小（字节）
- `blocks: u64` - 分配的 512B 块数量

**构造方法：**
- `new(mode: VfsNodePerm, ty: VfsNodeType, size: u64, blocks: u64)` - 创建具有指定属性的新实例
- `new_file(size: u64, blocks: u64)` - 为文件创建新实例，默认权限为 `0o666`
- `new_dir(size: u64, blocks: u64)` - 为目录创建新实例，默认权限为 `0o755`

**访问器方法：**
- `size()`, `blocks()`, `perm()`, `file_type()` - 获取对应属性
- `set_perm(perm: VfsNodePerm)` - 设置权限
- `is_file()`, `is_dir()` - 判断是否为文件或目录

**Section sources**
- [axfs_vfs/src/structs.rs](file://axfs_vfs/src/structs.rs#L13-L38)

### VfsNodePerm
[VfsNodePerm](file://axfs_vfs/src/structs.rs#L40-L82) 使用 bitflags 宏定义节点权限模式。

**权限位：**
- 所有者：`OWNER_READ (0o400)`, `OWNER_WRITE (0o200)`, `OWNER_EXEC (0o100)`
- 组：`GROUP_READ (0o40)`, `GROUP_WRITE (0o20)`, `GROUP_EXEC (0o10)`
- 其他：`OTHER_READ (0o4)`, `OTHER_WRITE (0o2)`, `OTHER_EXEC (0o1)`

**常量方法：**
- `default_file()` - 返回文件默认权限 `0o666`
- `default_dir()` - 返回目录默认权限 `0o755`

**实用方法：**
- `mode()` - 返回底层原始位
- `rwx_buf()` - 返回 9 字节字符串表示（如 `rwxr-xr-x`）
- `owner_readable()`, `owner_writable()`, `owner_executable()` - 检查所有者权限

**Section sources**
- [axfs_vfs/src/structs.rs](file://axfs_vfs/src/structs.rs#L40-L82)

### VfsNodeType
[VfsNodeType](file://axfs_vfs/src/structs.rs#L84-L114) 枚举定义节点类型。

**成员：**
- `Fifo = 0o1` - FIFO（命名管道）
- `CharDevice = 0o2` - 字符设备
- `Dir = 0o4` - 目录
- `BlockDevice = 0o6` - 块设备
- `File = 0o10` - 普通文件
- `SymLink = 0o12` - 符号链接
- `Socket = 0o14` - 套接字

**判断方法：**
- `is_file()`, `is_dir()`, `is_symlink()`, `is_block_device()`, `is_char_device()`, `is_fifo()`, `is_socket()` - 检查类型
- `as_char()` - 返回字符表示（如 `d` 表示目录，`-` 表示普通文件）

**Section sources**
- [axfs_vfs/src/structs.rs](file://axfs_vfs/src/structs.rs#L84-L114)

## axfs_devfs 设备实现

### DeviceFileSystem
[DeviceFileSystem](file://axfs_devfs/src/lib.rs#L38-L73) 是基于 `axfs_vfs` 实现的设备文件系统。

**字段：**
- `parent: Once<VfsNodeRef>` - 父节点引用（惰性初始化）
- `root: Arc<DirNode>` - 根目录节点

**方法：**
- `new() -> Self` - 创建新实例
- `mkdir(&self, name: &'static str) -> Arc<DirNode>` - 在根目录创建子目录
- `add(&self, name: &'static str, node: VfsNodeRef)` - 向根目录添加节点
- `default()` - 默认构造函数

**实现的 VfsOps：**
- `mount()` - 设置根目录的父节点
- `root_dir()` - 返回根目录节点

**Section sources**
- [axfs_devfs/src/lib.rs](file://axfs_devfs/src/lib.rs#L38-L73)

### NullDev
[NullDev](file://axfs_devfs/src/null.rs#L5-L31) 实现类似 `/dev/null` 的空设备。

**行为：**
- 读取时返回 0 字节
- 写入时丢弃所有数据并返回写入长度

**实现的方法：**
- `get_attr()` - 返回字符设备属性
- `read_at()` - 返回 `Ok(0)`
- `write_at()` - 返回 `Ok(buf.len())`
- `truncate()` - 返回 `Ok(())`
- 使用 `impl_vfs_non_dir_default!` 宏填充其他方法

**Section sources**
- [axfs_devfs/src/null.rs](file://axfs_devfs/src/null.rs#L5-L31)

### ZeroDev
[ZeroDev](file://axfs_devfs/src/zero.rs#L5-L32) 实现类似 `/dev/zero` 的零设备。

**行为：**
- 读取时返回全 `\0` 字节
- 写入时丢弃所有数据并返回写入长度

**实现的方法：**
- `get_attr()` - 返回字符设备属性
- `read_at()` - 填充缓冲区为 0 并返回长度
- `write_at()` - 返回 `Ok(buf.len())`
- `truncate()` - 返回 `Ok(())`
- 使用 `impl_vfs_non_dir_default!` 宏填充其他方法

**Section sources**
- [axfs_devfs/src/zero.rs](file://axfs_devfs/src/zero.rs#L5-L32)

### UrandomDev
