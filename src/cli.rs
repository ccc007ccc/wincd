//! CLI 定义与分发
//!
//! 设计:
//! - 顶层兼容旧用法:`wincd <PATH>`、`wincd`(剪贴板)等价于 `wincd convert ...`
//! - 子命令风格(新):`wincd convert | init | install | uninstall | completions`
//! - 旧 flag(`--init <SHELL>`、`--setup`、`--uninstall`)保留,内部路由到子命令

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{
    completions as cmd_completions, convert as cmd_convert, init as cmd_init,
    install as cmd_install, uninstall as cmd_uninstall,
};
use crate::shell::Shell;
use crate::ui;

/// WSL 下一步到位的 Windows 路径导航工具
///
/// 不传子命令时等价于 `wincd convert`。
#[derive(Parser, Debug)]
#[command(
    name = "wincd",
    version,
    about,
    long_about,
    propagate_version = true,
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    // ============== 顶层 fallback flags(等价于 `wincd convert ...`) ==============
    /// Windows 路径,省略则从剪贴板读取
    pub path: Option<String>,

    /// 反向转换:WSL → Windows
    #[arg(short = 'w', long = "to-windows")]
    pub to_windows: bool,

    /// 输出 Windows 路径但用 / 分隔
    #[arg(short = 'm', long = "mixed")]
    pub mixed: bool,

    /// 自动向上查找存在的父目录
    #[arg(short = 'p', long = "parent")]
    pub parent: bool,

    /// 跳过路径存在性检查
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// 显示转换详情
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    // ============== 兼容旧版的 flag ==============
    /// [已废弃] 输出 shell 集成代码,改用 `wincd init <SHELL>`
    #[arg(long = "init", value_name = "SHELL", hide = true)]
    pub init_legacy: Option<String>,

    /// [已废弃] 一键配置,改用 `wincd install`
    #[arg(long = "setup", hide = true)]
    pub setup_legacy: bool,

    /// [已废弃] 卸载,改用 `wincd uninstall`
    #[arg(long = "uninstall", hide = true)]
    pub uninstall_legacy: bool,

    /// 禁用彩色输出(亦可设 NO_COLOR 环境变量)
    #[arg(long = "no-color", global = true)]
    pub no_color: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// 转换路径(默认行为)
    Convert {
        /// Windows 路径,省略则从剪贴板读
        path: Option<String>,
        #[arg(short = 'w', long = "to-windows")]
        to_windows: bool,
        #[arg(short = 'm', long = "mixed")]
        mixed: bool,
        #[arg(short = 'p', long = "parent")]
        parent: bool,
        #[arg(short = 'f', long = "force")]
        force: bool,
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },

    /// 输出 shell 集成代码到 stdout(用于 eval "$(wincd init bash)")
    Init {
        /// 目标 shell:bash/zsh/fish/powershell
        shell: Shell,
    },

    /// 安装 shell 集成 + 补全
    Install {
        /// 显式指定 shell;省略则自动检测
        #[arg(long = "shell")]
        shell: Option<Shell>,
        /// 强制覆盖已有的集成块(交互式终端会要求确认)
        #[arg(long = "force")]
        force: bool,
        /// 跳过 --force 时的确认提示(用于 CI/脚本)
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },

    /// 卸载 shell 集成 + 补全(默认含二进制,--keep-binary 保留)
    Uninstall {
        #[arg(long = "shell")]
        shell: Option<Shell>,
        /// 跳过所有确认提示
        #[arg(short = 'y', long = "yes")]
        yes: bool,
        /// 移除所有 shell 的集成
        #[arg(long = "all-shells")]
        all_shells: bool,
        /// 不删除二进制
        #[arg(long = "keep-binary")]
        keep_binary: bool,
    },

    /// 输出补全脚本到 stdout
    Completions {
        /// 目标 shell:bash/zsh/fish/powershell
        shell: Shell,
    },
}

/// 主入口分发。返回退出码。
pub fn run() -> i32 {
    let cli = Cli::parse();
    ui::init_color(cli.no_color);

    let result: Result<i32> = match cli.command {
        Some(Command::Convert {
            path,
            to_windows,
            mixed,
            parent,
            force,
            verbose,
        }) => cmd_convert::run(cmd_convert::ConvertArgs {
            path,
            to_windows,
            mixed,
            parent,
            force,
            verbose,
        }),
        Some(Command::Init { shell }) => cmd_init::run(shell).map(|_| 0),
        Some(Command::Install { shell, force, yes }) => {
            cmd_install::run(cmd_install::InstallArgs { shell, force, yes }).map(|_| 0)
        }
        Some(Command::Uninstall {
            shell,
            yes,
            all_shells,
            keep_binary,
        }) => cmd_uninstall::run(cmd_uninstall::UninstallArgs {
            shell,
            yes,
            all_shells,
            keep_binary,
        })
        .map(|_| 0),
        Some(Command::Completions { shell }) => cmd_completions::run(shell).map(|_| 0),
        None => {
            // 顶层 fallback:旧 flag 优先,否则按 convert 行为执行
            if let Some(s) = cli.init_legacy {
                let sh: Shell = match s.parse() {
                    Ok(sh) => sh,
                    Err(e) => {
                        ui::err(e);
                        return 1;
                    }
                };
                cmd_init::run(sh).map(|_| 0)
            } else if cli.setup_legacy {
                cmd_install::run(cmd_install::InstallArgs::default()).map(|_| 0)
            } else if cli.uninstall_legacy {
                cmd_uninstall::run(cmd_uninstall::UninstallArgs::default()).map(|_| 0)
            } else {
                cmd_convert::run(cmd_convert::ConvertArgs {
                    path: cli.path,
                    to_windows: cli.to_windows,
                    mixed: cli.mixed,
                    parent: cli.parent,
                    force: cli.force,
                    verbose: cli.verbose,
                })
            }
        }
    };

    match result {
        Ok(code) => code,
        Err(e) => {
            ui::err(format!("{:#}", e));
            1
        }
    }
}
