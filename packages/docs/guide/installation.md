# 下载与安装

## 系统要求

| 平台    | 最低版本                      |
| ------- | ----------------------------- |
| Windows | Windows 10 (1903) 及以上      |
| macOS   | macOS 10.15 (Catalina) 及以上 |

::: tip 💡 推荐使用智能下载页面
我们为所有用户开发了全新的 **[智能下载页面](/download)**。它能够**自动识别您的操作系统与 CPU 架构**，并提供**国内高速下载通道**（无需代理，一键秒级下载）！

建议直接前往：**👉 [前往 Onin 智能下载页面](/download)**
:::

## 下载方式

除上述 **[智能下载页面](/download)** 之外，您也可以前往 [GitHub Releases](https://github.com/b-yp/Onin/releases) 手动下载。

| 平台                      | 适用架构                    | 安装包文件名                |
| ------------------------- | --------------------------- | --------------------------- |
| **Windows**               | Intel/AMD 64位 (x64)        | `Onin_x.x.x_x64-setup.exe`  |
| **macOS (苹果芯片)**      | Apple Silicon (ARM64)       | `Onin_x.x.x_aarch64.dmg`    |
| **macOS (Intel芯片)**     | Intel 处理器 (x64)          | `Onin_x.x.x_x64.dmg`        |
| **Linux (Ubuntu/Debian)** | Debian/Ubuntu 系统 (x64)    | `onin_x.x.x_amd64.deb`      |
| **Linux (通用)**          | 绝大多数 Linux 发行版 (x64) | `onin_x.x.x_amd64.AppImage` |

## Windows 安装

直接双击下载的 `.exe` 安装包，按照安装向导完成安装即可。

## macOS 安装

1. 下载对应架构的 `.dmg` 文件
2. 打开 `.dmg` 并将 `Onin.app` 拖入应用程序目录
3. 从「应用程序」中打开 Onin

### macOS 常见问题：「已损坏，无法打开」

由于 Onin 尚未进行 Apple 公证，首次打开时可能出现此提示。在终端执行以下命令解除限制：

```bash
xattr -cr /Applications/Onin.app
```

然后重新双击打开即可。

## Linux 安装

### Debian / Ubuntu (.deb)

在终端执行以下命令进行安装：

```bash
sudo dpkg -i onin_x.x.x_amd64.deb
sudo apt-get install -f # 修复可能的依赖问题
```

### 通用 Linux (.AppImage)

赋予执行权限后即可直接运行：

```bash
chmod +x onin_x.x.x_amd64.AppImage
./onin_x.x.x_amd64.AppImage
```

## 设置开机自启

打开 Onin，按 `⌘,`（macOS）或 `Ctrl+,`（Windows）打开设置，在「系统设置」中开启「开机自启」。
