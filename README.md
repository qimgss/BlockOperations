# Block Operations

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](http://www.apache.org/licenses/)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

一个专为 Android 设备设计的 Rust CLI 工具，用于查找和刷写块设备分区。

```
  _     _ _                   
 | |__ | | | _____  _ __  ___ 
 | '_ \| | |/ / _ \| '_ \/ __|
 | |_) | |   < (_) | |_) \__ \
 |_.__/|_|_|\_\___/| .__/|___/
                   |_|        
```

# 如何使用
``` bash
blkops - 纯 Rust 安卓 Block Device 工具

blkops -s, --search <partition>            Search for a partition and show its device path
blkops -s -p <partition>                   Search and show only the device path
blkops -w, --write <image> <partition>     Write image to partition
blkops -d <partition> <image>              Dump partition to image file
blkops -h, --help                          Show this help message

```

# 如何构建
<details>
<summary>在 Windows 上编译</summary>

构建前，请确保已安装以下环境：

- [Visual Studio 2022 Build Tools](https://visualstudio.microsoft.com/downloads/)
  - 勾选 **“使用 C++ 的桌面开发”**
  - 安装 **Windows SDK**
- [Rust](https://www.rust-lang.org/tools/install)
- [Android NDK](https://developer.android.google.cn/ndk/downloads?hl=zh-cn)

- 你需要手动修改编译脚本的Android NDK目录，否则会编译失败

然后在项目根目录执行：
``` powershell
./build-win.ps1
```
</details>

<details>
<summary>在 Linux 上编译</summary>

构建前，请确保已安装以下环境：
- [Rust](https://www.rust-lang.org/tools/install)
- [Android NDK](https://developer.android.google.cn/ndk/downloads?hl=zh-cn)

然后在项目根目录执行：
``` bash
./build-linux.sh --ndk-path /path/to/android-ndk # 手动修改为Android NDK的实际路径
```
</details>