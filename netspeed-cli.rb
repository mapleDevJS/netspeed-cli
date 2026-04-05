class NetspeedCli < Formula
  desc "Command-line interface for testing internet bandwidth using speedtest.net"
  homepage "https://github.com/mapleDevJS/netspeed-cli"
  url "https://github.com/mapleDevJS/netspeed-cli/archive/refs/tags/v0.3.0.tar.gz"
  sha256 "cc1ef0239b42414a3b3656efc32079be669c1a2cf39cd6d7a4d23e7b59560c45"

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
