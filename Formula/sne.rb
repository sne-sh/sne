# Written by packaging/update-manifests.ps1. Do not edit by hand.
class Sne < Formula
  desc "Simple Node Environment: a fast Node.js version manager"
  homepage "https://sne-sh.github.io/sne/"
  version "0.1.0-beta.4"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/sne-sh/sne/releases/download/0.1.0-beta.4/sne-0.1.0-beta.4-aarch64-apple-darwin.tar.gz"
      sha256 "74f0b765d29b88ffa4170bd088de2581e6abab6bc2845a269b56c4b8cace7ed3"
    end
    on_intel do
      url "https://github.com/sne-sh/sne/releases/download/0.1.0-beta.4/sne-0.1.0-beta.4-x86_64-apple-darwin.tar.gz"
      sha256 "6b0baba84588bb14a03f0c90bcf4999091e1309c6459b0f0dbfa96a5c26e8a88"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/sne-sh/sne/releases/download/0.1.0-beta.4/sne-0.1.0-beta.4-aarch64-unknown-linux-musl.tar.gz"
      sha256 "8a54a4448e991319233c265e4268f2981d05768b11e14d3cb9202a48794170d4"
    end
    on_intel do
      url "https://github.com/sne-sh/sne/releases/download/0.1.0-beta.4/sne-0.1.0-beta.4-x86_64-unknown-linux-musl.tar.gz"
      sha256 "8bd729e2a3fb81122e54aef08bb9c9f162e98d134e14c1fcb10411456b84fb28"
    end
  end

  def install
    bin.install "sne"
  end

  def caveats
    <<~EOS
      Add the setup line to your shell profile, then open a new terminal:
        bash, zsh:  eval "$(sne env --use-on-cd)"
        fish:       sne env --use-on-cd | source
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/sne --version")
  end
end
