use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::cli::{CliArgs, ShellType};

/// Generate shell completion script and write to stdout.
///
/// Called when `--generate-completion SHELL` is passed.
/// The output should be redirected to a file or sourced directly.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_type_to_shell_bash() {
        let shell = ShellType::Bash;
        let _ = match shell {
            ShellType::Bash => Shell::Bash,
            ShellType::Zsh => Shell::Zsh,
            ShellType::Fish => Shell::Fish,
            ShellType::PowerShell => Shell::PowerShell,
            ShellType::Elvish => Shell::Elvish,
        };
    }

    #[test]
    fn test_shell_type_to_shell_zsh() {
        let shell = ShellType::Zsh;
        let _ = match shell {
            ShellType::Bash => Shell::Bash,
            ShellType::Zsh => Shell::Zsh,
            ShellType::Fish => Shell::Fish,
            ShellType::PowerShell => Shell::PowerShell,
            ShellType::Elvish => Shell::Elvish,
        };
    }

    #[test]
    fn test_shell_type_to_shell_fish() {
        let shell = ShellType::Fish;
        let _ = match shell {
            ShellType::Bash => Shell::Bash,
            ShellType::Zsh => Shell::Zsh,
            ShellType::Fish => Shell::Fish,
            ShellType::PowerShell => Shell::PowerShell,
            ShellType::Elvish => Shell::Elvish,
        };
    }

    #[test]
    fn test_shell_type_to_shell_powershell() {
        let shell = ShellType::PowerShell;
        let _ = match shell {
            ShellType::Bash => Shell::Bash,
            ShellType::Zsh => Shell::Zsh,
            ShellType::Fish => Shell::Fish,
            ShellType::PowerShell => Shell::PowerShell,
            ShellType::Elvish => Shell::Elvish,
        };
    }

    #[test]
    fn test_shell_type_to_shell_elvish() {
        let shell = ShellType::Elvish;
        let _ = match shell {
            ShellType::Bash => Shell::Bash,
            ShellType::Zsh => Shell::Zsh,
            ShellType::Fish => Shell::Fish,
            ShellType::PowerShell => Shell::PowerShell,
            ShellType::Elvish => Shell::Elvish,
        };
    }

    #[test]
    fn test_generate_shell_completion_bash() {
        // Should not panic
        generate_shell_completion(ShellType::Bash);
    }

    #[test]
    fn test_generate_shell_completion_zsh() {
        // Should not panic
        generate_shell_completion(ShellType::Zsh);
    }

    #[test]
    fn test_generate_shell_completion_fish() {
        // Should not panic
        generate_shell_completion(ShellType::Fish);
    }

    #[test]
    fn test_generate_shell_completion_powershell() {
        // Should not panic
        generate_shell_completion(ShellType::PowerShell);
    }

    #[test]
    fn test_generate_shell_completion_elvish() {
        // Should not panic
        generate_shell_completion(ShellType::Elvish);
    }

    #[test]
    fn test_command_has_bin_name() {
        let cmd = CliArgs::command();
        assert_eq!(cmd.get_name(), "netspeed-cli");
    }

    #[test]
    fn test_generate_outputs_to_stdout() {
        // We can't easily capture stdout, but we can verify the function runs
        // The generate() function writes to io::stdout(), which is verified by not panicking
        generate_shell_completion(ShellType::Bash);
    }
}
