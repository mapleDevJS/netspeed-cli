class NetspeedCli < Formula
  desc "Command line interface for testing internet bandwidth using speedtest.net"
  homepage "https://github.com/mapleDevJS/netspeed-cli"
  url "https://github.com/mapleDevJS/netspeed-cli/archive/69a1b0286516bc5baeb586898b1e1d9c7c150af0.tar.gz"
  sha256 "da9bcf860d2ebccc4488ad1640f2377726f9730575e4b6ce4c2fe091f411c047"
  version "0.1.0"
  
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
