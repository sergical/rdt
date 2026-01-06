class Rdt < Formula
  desc "Reddit CLI for AI agents - search and interact with Reddit from the terminal"
  homepage "https://github.com/sergical/rdt"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/sergical/rdt/releases/download/v#{version}/rdt-aarch64-apple-darwin.tar.gz"
      # sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
    on_intel do
      url "https://github.com/sergical/rdt/releases/download/v#{version}/rdt-x86_64-apple-darwin.tar.gz"
      # sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  on_linux do
    url "https://github.com/sergical/rdt/releases/download/v#{version}/rdt-x86_64-unknown-linux-gnu.tar.gz"
    # sha256 "REPLACE_WITH_ACTUAL_SHA256"
  end

  def install
    bin.install "rdt"
  end

  test do
    assert_match "rdt", shell_output("#{bin}/rdt --version")
  end
end
