# wincd

> WSL 下一步到位的 Windows 路径导航工具 — 粘贴 Windows 路径，直接 cd

[![CI](https://github.com/ccc007ccc/wincd/actions/workflows/ci.yml/badge.svg)](https://github.com/ccc007ccc/wincd/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/wincd)](https://crates.io/crates/wincd)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

## 为什么需要 wincd？

在 WSL 中使用 Windows 路径是一件痛苦的事：

```bash
# 传统方式：手动转换
cd /mnt/c/Users/foo/Documents/Projects   # 手敲半天

# 用 wslpath：还得拼接
cd $(wslpath 'C:\Users\foo\Documents\Projects')

# 用 wincd：一步到位
wcd 'C:\Users\foo\Documents\Projects'
```

你甚至可以直接从剪贴板读取路径 —— 在 Windows 资源管理器里 Ctrl+C 复制路径，然后在 WSL 里直接 `wcd` 就行了。

## 功能特性

- **多种路径格式支持**：`C:\`、`C:/`、`\\wsl$\`、`\\server\share`、混合分隔符
- **剪贴板集成**：无参数时自动读取 Windows 剪贴板
- **直接 cd**：通过 shell 集成实现真正的目录切换
- **路径存在性检查**：自动验证目标路径，给出模糊匹配建议
- **反向转换**：WSL 路径 → Windows 路径
- **Shell 补全**：支持 bash、zsh、fish 自动补全
- **纯 Rust 实现**：零外部依赖，编译即用

## 安装

### 一键安装（推荐）

```bash
curl -fsSL https://raw.githubusercontent.com/ccc007ccc/wincd/main/install.sh | sh
```

### 从 GitHub Release 下载

前往 [Releases](https://github.com/ccc007ccc/wincd/releases) 页面下载对应平台的二进制文件。

### 从源码编译

```bash
git clone https://github.com/ccc007ccc/wincd.git
cd wincd
cargo build --release
cp target/release/wincd ~/.local/bin/
```

### 通过 cargo 安装

```bash
cargo install wincd
```

## 快速开始

### 1. 基础用法

```bash
# 转换 Windows 路径
wincd 'C:\Users\foo\Documents'
# 输出: /mnt/c/Users/foo/Documents

# 支持正斜杠
wincd 'C:/Users/foo/Documents'
# 输出: /mnt/c/Users/foo/Documents

# 支持 UNC 路径
wincd '\\wsl$\Ubuntu\home\user'
# 输出: /home/user
```

### 2. 剪贴板模式

```bash
# 在 Windows 资源管理器中复制路径后，直接运行：
wincd
# 自动读取剪贴板并转换
```

### 3. Shell 集成（推荐）

添加到你的 `~/.bashrc` 或 `~/.zshrc`：

```bash
eval "$(wincd --init bash)"
```

然后就可以直接用 `wcd` 命令：

```bash
wcd 'C:\code\Rust'
# 直接切换到 /mnt/c/code/Rust

wcd  # 无参数 = 从剪贴板读取
```

支持的 shell：`bash`、`zsh`、`fish`

### 4. 反向转换

```bash
# WSL → Windows
wincd -w /home/user/projects
# 输出: C:\Users\...\home\user\projects

# 使用 / 分隔的 Windows 路径
wincd -m /home/user/projects
# 输出: C:/Users/.../home/user/projects
```

### 5. 路径不存在时

```bash
wincd 'C:\Users\foo\NonExistent'
# 警告: 路径不存在: /mnt/c/Users/foo/NonExistent
# 可能的目录:
#   /mnt/c/Users/foo/Documents
#   /mnt/c/Users/foo/Desktop
#   /mnt/c/Users/foo/Downloads

# 自动向上查找存在的父目录
wincd -p 'C:\Users\foo\NonExistent\deep\path'
# 输出: /mnt/c/Users/foo （最近存在的父目录）
```

## 完整用法

```
wincd [OPTIONS] [PATH]

参数:
  [PATH]  Windows 路径，省略则从剪贴板读取

选项:
  -w, --to-windows    反向转换：WSL → Windows
  -m, --mixed         输出 Windows 路径但用 / 分隔
  -p, --parent        自动向上查找存在的父目录
  -f, --force         跳过路径存在性检查
  -v, --verbose       显示转换详情
  --init <SHELL>      输出 shell 集成代码 [bash, zsh, fish]
  --no-color          禁用彩色输出
  -h, --help          显示帮助
  -V, --version       显示版本
```

## 自定义挂载点

如果你的 WSL 使用了自定义挂载点（在 `/etc/wsl.conf` 中配置），wincd 会自动检测：

```ini
# /etc/wsl.conf
[automount]
root = /drv
```

wincd 会自动使用 `/drv/c/...` 而不是 `/mnt/c/...`。

## Shell 补全

项目 `completions/` 目录下提供了 bash、zsh、fish 的补全脚本：

```bash
# bash
cp completions/wincd.bash ~/.local/share/bash-completion/completions/wincd

# zsh
cp completions/wincd.zsh ~/.zfunc/_wincd

# fish
cp completions/wincd.fish ~/.config/fish/completions/wincd.fish
```

## 许可证

[MIT](LICENSE-MIT)
