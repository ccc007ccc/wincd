//! `wincd init <SHELL>` — 输出 shell 集成代码到 stdout

use crate::shell::{self, Shell};
use anyhow::Result;

pub fn run(shell: Shell) -> Result<()> {
    print!("{}", shell::init_script(shell));
    Ok(())
}
