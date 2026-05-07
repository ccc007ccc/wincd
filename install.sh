#!/usr/bin/env sh
# wincd 一键安装脚本
#
# 用法:
#   curl -fsSL https://raw.githubusercontent.com/ccc007ccc/wincd/main/install.sh | sh
#
# 环境变量:
#   WINCD_VERSION   指定版本(默认: latest)
#   WINCD_PREFIX    安装前缀(默认: $HOME/.local)
#   WINCD_NO_SETUP  非空时跳过自动 wincd install
#   WINCD_VERIFY    "0" 关闭 sha256 校验(默认: 开启)

set -eu

REPO="ccc007ccc/wincd"
VERSION="${WINCD_VERSION:-latest}"
PREFIX="${WINCD_PREFIX:-${HOME}/.local}"
INSTALL_DIR="${PREFIX}/bin"

# ------- 颜色 -------
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; CYAN=''; NC=''
fi
info()  { printf "${GREEN}[信息]${NC} %s\n" "$*"; }
warn()  { printf "${YELLOW}[警告]${NC} %s\n" "$*" >&2; }
hint()  { printf "${CYAN}[提示]${NC} %s\n" "$*"; }
error() { printf "${RED}[错误]${NC} %s\n" "$*" >&2; exit 1; }

# ------- 工具检测 -------
need() { command -v "$1" >/dev/null 2>&1 || error "缺少依赖: $1"; }
need uname
need curl
need chmod
need mkdir

# 检测 sha256 工具(任一即可)
SHA256_CMD=""
if command -v sha256sum >/dev/null 2>&1; then
    SHA256_CMD="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
    SHA256_CMD="shasum -a 256"
fi

# ------- 架构检测 -------
detect_arch() {
    a=$(uname -m)
    case "$a" in
        x86_64|amd64)   echo "amd64" ;;
        aarch64|arm64)  echo "arm64" ;;
        *)              error "不支持的架构: $a(支持 amd64/arm64)" ;;
    esac
}

ARCH=$(detect_arch)
ASSET="wincd-linux-${ARCH}"

# 拼接下载 URL
if [ "$VERSION" = "latest" ]; then
    BIN_URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
    SHA_URL="https://github.com/${REPO}/releases/latest/download/${ASSET}.sha256"
else
    BIN_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
    SHA_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}.sha256"
fi

info "目标版本: ${VERSION}"
info "目标架构: ${ARCH}"
info "安装目录: ${INSTALL_DIR}"

mkdir -p "$INSTALL_DIR"

# 临时目录
TMP=$(mktemp -d "${TMPDIR:-/tmp}/wincd-install.XXXXXX")
trap 'rm -rf "$TMP"' EXIT INT TERM

# 下载二进制
info "下载 ${ASSET}..."
if ! curl -fSL "$BIN_URL" -o "${TMP}/${ASSET}"; then
    error "下载失败,请检查网络或前往 https://github.com/${REPO}/releases 手动下载"
fi

# sha256 校验(可关闭)
if [ "${WINCD_VERIFY:-1}" != "0" ]; then
    if [ -z "$SHA256_CMD" ]; then
        warn "找不到 sha256sum/shasum,跳过校验(可设 WINCD_VERIFY=0 显式关闭警告)"
    else
        info "下载校验文件..."
        if curl -fSL "$SHA_URL" -o "${TMP}/${ASSET}.sha256" 2>/dev/null; then
            (cd "$TMP" && $SHA256_CMD -c "${ASSET}.sha256" >/dev/null 2>&1) \
                || error "sha256 校验失败,文件可能损坏或被篡改"
            info "sha256 校验通过"
        else
            warn "无 sha256 校验文件(旧版本 release 可能没提供),跳过"
        fi
    fi
fi

# 安装
mv "${TMP}/${ASSET}" "${INSTALL_DIR}/wincd"
chmod +x "${INSTALL_DIR}/wincd"
info "已安装: ${INSTALL_DIR}/wincd"

# PATH 检查
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        warn "${INSTALL_DIR} 不在 PATH 中"
        printf "\n请将以下行添加到 shell 配置:\n\n  export PATH=\"%s:\$PATH\"\n\n" "$INSTALL_DIR"
        ;;
esac

# 版本输出
"${INSTALL_DIR}/wincd" --version

# 自动 setup
if [ -z "${WINCD_NO_SETUP:-}" ]; then
    printf "\n"
    info "运行 wincd install 配置 shell 集成与补全..."
    "${INSTALL_DIR}/wincd" install || warn "自动 install 失败,可稍后手动运行: wincd install"
fi

printf "\n"
info "完成。请重启 shell 或执行 source ~/.bashrc(或对应 rc 文件)"
