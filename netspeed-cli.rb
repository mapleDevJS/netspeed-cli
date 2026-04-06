class NetspeedCli < Formula
  desc "Command-line interface for testing internet bandwidth using speedtest.net"
  homepage "https://github.com/mapleDevJS/netspeed-cli"
  url "https://github.com/mapleDevJS/netspeed-cli/releases/download/v0.5.1/netspeed-cli-0.5.1.tar.gz"
  sha256 "bd798778f6212ee008b8d1922e0a70c115009381f8f35455623e6e4fdf83f932"

  license "MIT"

  livecheck do
    url :stable
    regex(/^v?(\d+(?:\.\d+)+)$/i)
  end

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args

    # Install shell completions
    bash_completion.install "completions/netspeed-cli.bash" => "netspeed-cli"
    zsh_completion.install "completions/_netspeed-cli" => "_netspeed-cli"
    fish_completion.install "completions/netspeed-cli.fish"

    # Install man page
    man1.install "netspeed-cli.1"
  end

  test do
    assert_match "internet bandwidth", shell_output("#{bin}/netspeed-cli --help")
  end
end
