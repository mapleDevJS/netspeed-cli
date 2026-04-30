class NetspeedCli < Formula
  desc "Command-line interface for testing internet bandwidth using speedtest.net"
  homepage "https://github.com/mapleDevJS/netspeed-cli"
  url "https://github.com/mapleDevJS/netspeed-cli/archive/refs/tags/v0.10.3.tar.gz"
  version "0.10.3"
  sha256 "b1393ca9fece3c5a045ee66b88d79f583755e6656a8de7758b6532c91d674999"

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
