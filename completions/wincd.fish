# wincd fish 自动补全

complete -c wincd -f
complete -c wincd -s w -l to-windows -d '反向转换: WSL → Windows'
complete -c wincd -s m -l mixed -d '输出 Windows 路径但用 / 分隔'
complete -c wincd -s p -l parent -d '自动向上查找存在的父目录'
complete -c wincd -s f -l force -d '跳过路径存在性检查'
complete -c wincd -s v -l verbose -d '显示转换详情'
complete -c wincd -l init -d '输出 shell 集成代码' -xa 'bash zsh fish'
complete -c wincd -l no-color -d '禁用彩色输出'
complete -c wincd -s h -l help -d '显示帮助'
complete -c wincd -s V -l version -d '显示版本'
