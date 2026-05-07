# wincd

> WSL 下一步到位的 Windows 路径导航工具 — 粘贴 Windows 路径,直接 cd

[![CI](https://github.com/ccc007ccc/wincd/actions/workflows/ci.yml/badge.svg)](https://github.com/ccc007ccc/wincd/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/wincd)](https://crates.io/crates/wincd)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

[English](README.md)

## 这是什么

把 Windows 路径转成 WSL 路径(或反过来),并一步 `cd` 过去。

```bash
wcd 'C:\Users\foo\Documents\Projects'    # → cd /mnt/c/Users/foo/Documents/Projects
wcd                                       # ← 读剪贴板,失败则交互式提示
wincd -w /mnt/c/Users/foo                 # → C:\Users\foo (反向)
```

## 特性

- **多种路径格式**:`C:\`、`C:/`、`\wsl$\…`、`\wsl.localhost\…`、`\server\share\…`、混合分隔符、`~/…`
- **剪贴板集成**:无参时读 Windows 剪贴板;`arboard` 初始化失败(无 WSLg/X11)时 fallback 到 `powershell.exe Get-Clipboard`
- **交互式输入**:剪贴板非路径时,`wcd` 用 `read -r` 读一行 — 完全规避 shell 转义
- **真正切目录**:bash / zsh / fish / PowerShell 的 shell wrapper
- **存在性检查 + 智能建议**:路径不存在时按 Jaro-Winkler 相似度推荐相邻目录
- **一键安装/卸载**:`wincd install` / `wincd uninstall`
- **发行版感知**:识别 `WSL_DISTRO_NAME` 与 `/etc/wsl.conf` 中 `[automount] root`

## 安装

### 一行命令

```bash
curl -fsSL https://raw.githubusercontent.com/ccc007ccc/wincd/main/install.sh | sh
```

脚本会:
1. 检测架构(`amd64` / `arm64`)
2. 从 GitHub Releases 下载二进制
3. **校验 SHA256**(`WINCD_VERIFY=0` 可关闭)
4. 自动运行 `wincd install`(`WINCD_NO_SETUP=1` 可跳过)

指定版本:

```bash
curl -fsSL https://raw.githubusercontent.com/ccc007ccc/wincd/main/install.sh | WINCD_VERSION=v2.0.0 sh
```

### 源码 / cargo

```bash
cargo install wincd          # 或: git clone … && cargo install --path .
wincd install
```

## CLI

`wincd` 有两套等价用法 — 顶层参数(兼容旧版),或显式子命令(脚本推荐)。

```
# 默认行为 — 等价于 `wincd convert`
wincd [OPTIONS] [PATH]

# 显式子命令
wincd convert      [OPTIONS] [PATH]      # 转换路径
wincd init         <SHELL>               # 输出集成代码,配合 eval "$(wincd init bash)"
wincd install      [--shell SHELL] [--force] [-y]
wincd uninstall    [--shell SHELL] [--all-shells] [--keep-binary] [-y]
wincd completions  <SHELL>               # 输出补全脚本
```

`<SHELL>`:`bash` | `zsh` | `fish` | `powershell`(别名:`pwsh`、`ps1`)。

### convert 选项

| 选项                  | 含义                                          |
|---------------------|---------------------------------------------|
| `-w, --to-windows`  | 反向:WSL → Windows                            |
| `-m, --mixed`       | Windows 输出用 `/` 分隔                          |
| `-p, --parent`      | 路径不存在时自动找最近的存在父目录                            |
| `-f, --force`       | 跳过存在性检查(直接当成有效路径)                            |
| `-v, --verbose`     | 在 stderr 打印转换详情                             |
| `--no-color`        | 禁用着色(亦可设 `NO_COLOR` 环境变量)                  |

### 旧 flag(仍然可用)

`--init <SHELL>` ↔ `wincd init <SHELL>`
`--setup`        ↔ `wincd install`
`--uninstall`    ↔ `wincd uninstall`

## 为什么 `wcd 'C:\foo'` 行,`wcd C:\foo` 不行?

因为 **bash 和 zsh 在程序看到参数之前就会把 `\c`、`\R`、`\s` 当作转义序列处理**,这是 shell 的限制,任何工具都没法直接绕过。`wcd` 函数提供三种应对方式:

1. **显式加引号**:`wcd 'C:\foo'`(单引号阻止转义)
2. **走剪贴板**:`wcd`(无参) — 读 Windows 剪贴板
3. **交互式输入**:`wcd`(剪贴板为空/非路径时) — 用 `read -r` 读一行,shell 完全不参与

## 安全:`install` / `uninstall` 的确认行为

`install` 和 `uninstall` 子命令会动文件,所以默认走**安全**路径,防止打错或子命令撞名:

| 操作                                        | 默认行为                                              |
|-------------------------------------------|---------------------------------------------------|
| `wincd install` (首次)                      | 写 rc + 补全。幂等,重跑无害。                                |
| `wincd install` (已配置)                     | 跳过,提示用 `--force` 强制覆盖。                            |
| `wincd install --force`                   | 交互式终端下**要求 `y/N` 确认**。                            |
| `wincd install --force --yes`             | 跳过确认(CI / 脚本用)。                                  |
| `wincd uninstall`                         | **列出将执行的所有操作,要求 `y/N` 确认。**                       |
| `wincd uninstall --yes`                   | 跳过确认。                                             |
| `wincd uninstall` 在非 TTY 中且无 `--yes`      | **拒绝执行。** 流水线中必须显式 `--yes` 才能跑。                   |

为什么这么谨慎?因为 `uninstall` 默认会删除 `~/.local/bin/wincd` 或 `~/.cargo/bin/wincd` 这个二进制。如果你不小心把一个以 `uninstall` 开头的字串当成了第一个参数(虽然概率很低),也有机会中止。

## 智能路径查找

```bash
# 路径不存在 — wincd 列出最像的兄弟目录
wincd 'C:\Users\foo\NonExistnt'
# 警告: 路径不存在
# 可能的目录:
#   /mnt/c/Users/foo/NonExistent      ← 最相似
#   /mnt/c/Users/foo/Documents
#   …

# -p 自动向上找最近的存在父目录
wincd 'C:\Users\foo\NonExistent\deep\path' -p
# → /mnt/c/Users/foo
```

## 自定义挂载点

`/etc/wsl.conf`:
```ini
[automount]
root = /drv
```
→ wincd 输出 `/drv/c/…` 而不是 `/mnt/c/…`。

## 发行版名称检测(用于反向 UNC)

输出 `\wsl$\<distro>\…` 时,wincd 按以下顺序解析 `<distro>`:

1. `WSL_DISTRO_NAME` 环境变量(WSL 自动注入,最可靠,如 `Ubuntu-22.04`)
2. `/etc/os-release` 的 `ID` + `VERSION_ID`(如 `ubuntu` + `22.04` → `Ubuntu-22.04`)
3. 默认 `Ubuntu`

显式覆盖:`WSL_DISTRO_NAME=Debian wincd -w /home/user`。

## 故障排查

**安装后 `wcd: command not found`**
重新加载 rc:`source ~/.bashrc`(或开新 shell)。

**Tab 补全不生效**
- Bash:确认 `bash-completion` 已安装,`~/.local/share/bash-completion/completions` 在搜索路径上(多数发行版默认)。
- Zsh:加到 `~/.zshrc`(若没有):
  ```zsh
  fpath=(~/.zfunc $fpath)
  autoload -Uz compinit && compinit
  ```
- Fish:补全装到 `~/.config/fish/completions/`,fish 自动加载。

**剪贴板读取失败**
WSL 无 WSLg/X11 时 arboard 可能初始化失败,wincd 会自动 fallback 到 `powershell.exe Get-Clipboard`。若也失败,确认 `powershell.exe` 在 PATH 上(WSL 默认有)。

**反向 UNC 中 distro 名称不对**
显式设置 `WSL_DISTRO_NAME` 环境变量。

**想干跑(dry-run)看看 uninstall 会做什么**
直接 `wincd uninstall` 不带 `--yes` — 它会列出全部待执行操作并要求确认,选 `n` 干净退出,不改任何文件。

## 许可证

[MIT](LICENSE-MIT)
