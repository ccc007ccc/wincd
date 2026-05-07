//! 端到端 CLI 测试

use assert_cmd::Command;
use predicates::prelude::*;

fn wincd() -> Command {
    Command::cargo_bin("wincd").expect("wincd binary not found")
}

#[test]
fn test_version() {
    wincd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("wincd"));
}

#[test]
fn test_help_top_level() {
    wincd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("convert"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("install"))
        .stdout(predicate::str::contains("uninstall"))
        .stdout(predicate::str::contains("completions"));
}

#[test]
fn test_init_bash_outputs_markers() {
    wincd()
        .args(["init", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains(">>> wincd initialize >>>"))
        .stdout(predicate::str::contains("<<< wincd initialize <<<"))
        .stdout(predicate::str::contains("wcd()"));
}

#[test]
fn test_init_zsh_outputs_markers() {
    wincd()
        .args(["init", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("emulate -L zsh"));
}

#[test]
fn test_init_fish_outputs_function() {
    wincd()
        .args(["init", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("function wcd"));
}

#[test]
fn test_init_powershell_works() {
    wincd()
        .args(["init", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("function wcd"));
}

#[test]
fn test_init_powershell_alias_pwsh() {
    wincd().args(["init", "pwsh"]).assert().success();
}

#[test]
fn test_init_invalid_shell_fails() {
    wincd().args(["init", "tcsh"]).assert().failure();
}

#[test]
fn test_completions_bash() {
    wincd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_wincd"))
        .stdout(predicate::str::contains("complete -F _wincd"));
}

#[test]
fn test_completions_includes_wcd_alias() {
    wincd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wcd"));
}

#[test]
fn test_legacy_init_flag_still_works() {
    wincd()
        .args(["--init", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wcd()"));
}

#[test]
fn test_force_skips_existence_check() {
    wincd()
        .args(["convert", r"C:\definitely_nonexistent_xyz_12345", "-f"])
        .assert()
        .success()
        .stdout(predicate::str::contains("definitely_nonexistent_xyz_12345"));
}

#[test]
fn test_double_dash_protects_path() {
    // -- 之后的内容都是位置参数;'-f' 当作路径,识别失败
    wincd().args(["convert", "--", "-f"]).assert().failure();
}

#[test]
fn test_convert_subcommand_explicit() {
    wincd()
        .args(["convert", r"C:\Windows", "-f"])
        .assert()
        .success();
}

#[test]
fn test_no_color_flag_after_subcommand() {
    wincd()
        .args(["convert", r"C:\foo", "-f", "--no-color"])
        .assert()
        .success();
}

#[test]
fn test_no_color_env_var() {
    wincd()
        .env("NO_COLOR", "1")
        .args(["convert", r"C:\foo", "-f"])
        .assert()
        .success();
}

#[test]
fn test_to_windows_basic() {
    // 注意:在非 WSL 环境下 git bash 可能转换 /mnt/c/... 路径,
    // 用环境变量 MSYS_NO_PATHCONV 或绕过此测试
    let mut cmd = wincd();
    cmd.env("MSYS_NO_PATHCONV", "1")
        .args(["convert", "-w", "/mnt/c/Users/foo"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("C:"));
}

#[test]
fn test_mixed_separator() {
    wincd()
        .env("MSYS_NO_PATHCONV", "1")
        .args(["convert", "-w", "-m", "/mnt/c/Users/foo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("C:/"));
}
