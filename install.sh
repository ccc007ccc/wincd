#!/bin/bash
# wincd 一键安装脚本
# 用法: curl -fsSL https://raw.githubusercontent.com/ccc007ccc/wincd/main/install.sh | sh

set -e

REPO="ccc007ccc/wincd"
INSTALL_DIR="${HOME}/.local/bin"

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[信息]${NC} $1"; }
warn() { echo -e "${YELLOW}[警告]${NC} $1"; }
error() { echo -e "${RED}[错误]${NC} $1"; exit 1; }

# 检测系统架构
detect_arch() {
    local arch
    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)   echo "amd64" ;;
        aarch64|arm64)   echo "arm64" ;;
        *)               error "不支持的架构: $arch" ;;
    esac
}

# 检测操作系统
detect_os() {
    local os
    os=$(uname -s)
    case "$os" in
        Linux*)   echo "linux" ;;
        MINGW*|MSYS*|CYGWIN*)  echo "windows" ;;
        *)        error "不支持的操作系统: $os" ;;
    esac
}

# 获取最新版本号
get_latest_version() {
    local version
    version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"tag_name":\s*"([^"]+)".*/\1/')
    if [ -z "$version" ]; then
        error "无法获取最新版本号，请检查网络连接"
    fi
    echo "$version"
}

main() {
    local arch os version asset_name download_url wcd_asset wcd_url
    arch=$(detect_arch)
    os=$(detect_os)

    info "检测到系统: ${os}-${arch}"

    version=$(get_latest_version)
    info "最新版本: ${version}"

    # 构建下载文件名
    if [ "$os" = "windows" ]; then
        asset_name="wincd-windows-${arch}.exe"
        wcd_asset="wcd-windows-${arch}.exe"
    else
        asset_name="wincd-linux-${arch}"
        wcd_asset="wcd-linux-${arch}"
    fi

    download_url="https://github.com/${REPO}/releases/download/${version}/${asset_name}"
    wcd_url="https://github.com/${REPO}/releases/download/${version}/${wcd_asset}"

    # 创建安装目录
    mkdir -p "$INSTALL_DIR"

    # 下载 wincd
    info "下载 ${asset_name}..."
    curl -fsSL "$download_url" -o "${INSTALL_DIR}/wincd"
    chmod +x "${INSTALL_DIR}/wincd"

    # 下载 wcd
    info "下载 ${wcd_asset}..."
    curl -fsSL "$wcd_url" -o "${INSTALL_DIR}/wcd"
    chmod +x "${INSTALL_DIR}/wcd"

    info "安装完成:"
    info "  ${INSTALL_DIR}/wincd"
    info "  ${INSTALL_DIR}/wcd"

    # 检查 PATH
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        info "已可以使用 wincd / wcd 命令"
    else
        warn "${INSTALL_DIR} 不在 PATH 中"
        echo ""
        echo "请将以下内容添加到你的 shell 配置文件中:"
        echo ""
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
        echo "然后运行:"
        echo "  source ~/.bashrc  # 或 source ~/.zshrc"
        echo ""
    fi

    # 提示 shell 集成
    echo ""
    info "提示: wcd 可以直接输出转换后的路径"
    info "要让 wcd 自动 cd（推荐），请运行:"
    echo ""
    echo "  echo 'eval \"\$(wincd --init bash)\"' >> ~/.bashrc"
    echo ""
    info "支持的 shell: bash, zsh, fish"
}

main "$@"
