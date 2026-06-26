**English** | [简体中文](README.md)
# Block Operations
This document is machine-translated, and if there are mistakes, you can submit them to an ISSUE or Pull Request.

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](http://www.apache.org/licenses/)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

A Rust CLI tool developed specifically for Android devices, used to find and flash block device partitions

```
  _     _ _                   
 | |__ | | | _____  _ __  ___ 
 | '_ \| | |/ / _ \| '_ \/ __|
 | |_) | |   < (_) | |_) \__ \
 |_.__/|_|_|\_\___/| .__/|___/
                   |_|        
```

# How to use
``` bash
blkops - Pure Rust Android Block Device Tool

blkops -s, --search <partition>            Search for a partition and show its device path
blkops -s <partition> -p                   Search and show only the device path
blkops -w, --write <image> <partition>     Write image to partition
blkops -d <partition> <image>              Dump partition to image file
blkops -h, --help                          Show this help message

```

# How to build
<details>
<summary>Compiling on Windows</summary>

Before compiling，Please make sure to install the following environment:

- [Visual Studio 2022 Build Tools](https://visualstudio.microsoft.com/downloads/)
  - Choose **“Desktop development using C++”**
  - Install **Windows SDK**
- [Rust](https://www.rust-lang.org/tools/install)
- [Android NDK](https://developer.android.google.cn/ndk/downloads?hl=zh-cn)

- You must manual edit Android-NDK Directory in compiling script.Otherwise, it will fail to compile

Then run it in the project's root directory：
``` powershell
./build-win.ps1
```
</details>

<details>
<summary>Compiling on Linux</summary>

Before compiling，Please make sure to install the following environment:
- [Rust](https://www.rust-lang.org/tools/install)
- [Android NDK](https://developer.android.google.cn/ndk/downloads?hl=zh-cn)

Then run it in the project's root directory：
``` bash
./build-linux.sh --ndk-path /path/to/android-ndk # Manually modify the actual directory of the Android NDK
```
</details>