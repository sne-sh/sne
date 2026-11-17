# Written by packaging/update-manifests.ps1. Do not edit by hand.
class Sne < Formula
  desc "Simple Node Environment: a fast Node.js version manager"
  homepage "https://sne-sh.github.io/sne/"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/sne-sh/sne/releases/download/0.1.0/sne-0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "ec854d4da502622642caebc1d8c3ab84f097c94fe854a42f27c2755a2e394336"
    end
    on_intel do
      url "https://github.com/sne-sh/sne/releases/download/0.1.0/sne-0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "7b1a05b9a889ad7363480fc5489d94503b9d96eaa8c31db5541f3dad884c1c39"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/sne-sh/sne/releases/download/0.1.0/sne-0.1.0-aarch64-unknown-linux-musl.tar.gz"
      sha256 "0cbe011ff847abc3a63f6388b7a2c131325832d798a599a11494328ac1118ae6"
    end
    on_intel do
      url "https://github.com/sne-sh/sne/releases/download/0.1.0/sne-0.1.0-x86_64-unknown-linux-musl.tar.gz"
      sha256 "6440eb883b17e11e1eb92fc61787e492078cfa31d4f3ba21517c037095b6a257"
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
