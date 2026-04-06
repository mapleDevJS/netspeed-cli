use clap::CommandFactory;
use clap_complete::{Shell, generate_to};
use std::fs;
use std::path::Path;

// Include CLI args (which pulls in shared validation via its own include!)
include!("src/cli.rs");

fn main() -> std::io::Result<()> {
    // Create completions directory
    let out_dir = Path::new("completions");
    fs::create_dir_all(out_dir)?;

    // Generate shell completions
    let mut cmd = CliArgs::command();

    generate_to(Shell::Bash, &mut cmd, "netspeed-cli", out_dir)?;
    generate_to(Shell::Zsh, &mut cmd, "netspeed-cli", out_dir)?;
    generate_to(Shell::Fish, &mut cmd, "netspeed-cli", out_dir)?;
    generate_to(Shell::PowerShell, &mut cmd, "netspeed-cli", out_dir)?;
    generate_to(Shell::Elvish, &mut cmd, "netspeed-cli", out_dir)?;

    // Generate man page
    let man_dir = Path::new(".");
    fs::create_dir_all(man_dir)?;

    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Vec::new();
    man.render(&mut buffer)?;

    fs::write(man_dir.join("netspeed-cli.1"), buffer)?;

    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    Ok(())
}
