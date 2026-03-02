# 下载与安装

## 系统要求

| 平台    | 最低版本                      |
| ------- | ----------------------------- |
| Windows | Windows 10 (1903) 及以上      |
| macOS   | macOS 10.15 (Catalina) 及以上 |

## 下载

前往 [GitHub Releases](https://github.com/b-yp/Onin/releases) 下载最新版本。

| 平台                  | 安装包                     |
| --------------------- | -------------------------- |
| Windows               | `Onin_x.x.x_x64-setup.exe` |
| macOS (Apple Silicon) | `Onin_x.x.x_aarch64.dmg`   |
| macOS (Intel)         | `Onin_x.x.x_x64.dmg`       |

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

## 设置开机自启

打开 Onin，按 `⌘,`（macOS）或 `Ctrl+,`（Windows）打开设置，在「系统设置」中开启「开机自启」。
