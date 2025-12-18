class Superego < Formula
  desc "Superego - Metacognitive advisor for Claude Code"
  homepage "https://github.com/cloud-atlas-ai/superego"
  url "https://github.com/cloud-atlas-ai/superego/archive/refs/tags/v0.4.4.tar.gz"
  sha256 "2b7fcf0719f0935c8423518e178e5d1a181657c50de456ff0efac6154dc29464"
  license :cannot_represent

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "sg", shell_output("#{bin}/sg --help")
  end
end
