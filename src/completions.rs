use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::cli::{CliArgs, ShellType};

pub fn generate_shell_completion(shell: ShellType) {
    let mut cmd = CliArgs::command();
    let shell = match shell {
        ShellType::Bash => Shell::Bash,
        ShellType::Zsh => Shell::Zsh,
        ShellType::Fish => Shell::Fish,
        ShellType::PowerShell => Shell::PowerShell,
        ShellType::Elvish => Shell::Elvish,
    };

    let bin_name = "netspeed-cli";
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
}
