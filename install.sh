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
    local arch version asset_name download_url
    arch=$(detect_arch)

    info "检测到架构: ${arch}"

    version=$(get_latest_version)
    info "最新版本: ${version}"

    asset_name="wincd-linux-${arch}"
    download_url="https://github.com/${REPO}/releases/download/${version}/${asset_name}"

    # 创建安装目录
    mkdir -p "$INSTALL_DIR"

    # 下载 wincd
    info "下载 ${asset_name}..."
    curl -fsSL "$download_url" -o "${INSTALL_DIR}/wincd"
    chmod +x "${INSTALL_DIR}/wincd"

    info "安装完成: ${INSTALL_DIR}/wincd"

    # 检查 PATH
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        info "已可以使用 wincd 命令"
    else
        warn "${INSTALL_DIR} 不在 PATH 中"
        echo ""
        echo "请将以下内容添加到你的 shell 配置文件中:"
        echo ""
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
    fi

    # 自动配置 shell 集成
    echo ""
    info "正在配置 shell 集成..."
    "${INSTALL_DIR}/wincd" --setup || warn "自动配置失败，请手动运行: wincd --setup"
}

main "$@"
