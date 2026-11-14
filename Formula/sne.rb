# Written by packaging/update-manifests.ps1. Do not edit by hand.
class Sne < Formula
  desc "Simple Node Environment: a fast Node.js version manager"
  homepage "https://sne-sh.github.io/sne/"
  version "0.1.0-beta.5"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/sne-sh/sne/releases/download/0.1.0-beta.5/sne-0.1.0-beta.5-aarch64-apple-darwin.tar.gz"
      sha256 "b15b5d4a0b2aa3e575d67ea3b37d36e4ec9ea890889fedf88c4c9c9156a7085c"
    end
    on_intel do
      url "https://github.com/sne-sh/sne/releases/download/0.1.0-beta.5/sne-0.1.0-beta.5-x86_64-apple-darwin.tar.gz"
      sha256 "2e86207294c406823dd929b7c79c6c04ac157c73bce9b68b51b399ac88d00ebb"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/sne-sh/sne/releases/download/0.1.0-beta.5/sne-0.1.0-beta.5-aarch64-unknown-linux-musl.tar.gz"
      sha256 "ccc858ebe3dc3e3dc2e5651db3838cf882d62f7f59ade7dfd83aab0ea5411500"
    end
    on_intel do
      url "https://github.com/sne-sh/sne/releases/download/0.1.0-beta.5/sne-0.1.0-beta.5-x86_64-unknown-linux-musl.tar.gz"
      sha256 "9892fd6c2742c8220d9a33a2417ee4f5bbc9123683ed33055eab8305ac63601d"
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
