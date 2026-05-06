# wincd bash 自动补全
_wincd() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="-w --to-windows -m --mixed -p --parent -f --force -v --verbose --init --no-color -h --help -V --version"

    if [[ ${cur} == -* ]]; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi

    if [[ ${prev} == --init ]]; then
        COMPREPLY=( $(compgen -W "bash zsh fish" -- ${cur}) )
        return 0
    fi

    # 默认：文件路径补全
    COMPREPLY=( $(compgen -f -- ${cur}) )
    return 0
}
complete -F _wincd wincd
