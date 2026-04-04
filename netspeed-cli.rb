class NetspeedCli < Formula
  desc "Command line interface for testing internet bandwidth using speedtest.net"
  homepage "https://github.com/mapleDevJS/netspeed-cli"
  url "https://github.com/mapleDevJS/netspeed-cli/archive/refs/tags/v0.1.2.tar.gz"
  sha256 "a21eea4bd7af7835ee7291da1d6b80a5bfa86483814491ee29b23e090ef2f700"
  version "0.1.2"
  
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
    
    # Install shell completions
    bash_completion.install "completions/netspeed-cli.bash" => "netspeed-cli"
    zsh_completion.install "completions/_netspeed-cli" => "_netspeed-cli"
    fish_completion.install "completions/netspeed-cli.fish"
    
    # Install man page
    man1.install "netspeed-cli.1"
  end

  test do
    assert_match "netspeed-cli", shell_output("#{bin}/netspeed-cli --version")
  end
end
