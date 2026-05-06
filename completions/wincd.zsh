#compdef wincd

_wincd() {
    _arguments \
        '1: :_wincd_paths' \
        '-w[-- to-windows:WSL → Windows 反向转换]' \
        '(-m --mixed)'{-m,--mixed}'[输出 Windows 路径但用 / 分隔]' \
        '(-p --parent)'{-p,--parent}'[自动向上查找存在的父目录]' \
        '(-f --force)'{-f,--force}'[跳过路径存在性检查]' \
        '(-v --verbose)'{-v,--verbose}'[显示转换详情]' \
        '--init[输出 shell 集成代码]:shell:(bash zsh fish)' \
        '--no-color[禁用彩色输出]' \
        '(-h --help)'{-h,--help}'[显示帮助]' \
        '(-V --version)'{-V,--version}'[显示版本]'
}

_wincd_paths() {
    _files
}

_wincd "$@"
