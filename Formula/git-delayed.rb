class GitDelayed < Formula
  desc "Schedule git commits and pushes for future execution"
  homepage "https://github.com/yourusername/git-delayed"
  url "https://github.com/yourusername/git-delayed/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "YOUR_SHA256_HERE"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  def post_install
    # Create storage directory
    if OS.mac?
      storage_dir = "#{ENV["HOME"]}/Library/Application Support/git-delayed"
    else
      storage_dir = "#{ENV["HOME"]}/.config/git-delayed"
    end
    mkdir_p storage_dir
  end

  service do
    run [opt_bin/"git-delayed", "daemon", "start"]
    keep_alive true
    log_path var/"log/git-delayed.log"
    error_log_path var/"log/git-delayed.log"
  end

  def caveats
    <<~EOS
      To start the git-delayed daemon automatically:
        brew services start git-delayed

      Or start it manually:
        git-delayed daemon start

      Usage:
        git-delayed schedule "+10 hours" commit -m "your commit message"
        git-delayed list
        git-delayed logs
        git-delayed daemon status
    EOS
  end

  test do
    # Test that the binary runs
    system "#{bin}/git-delayed", "--help"
    
    # Test time parsing
    output = shell_output("#{bin}/git-delayed schedule '+1 hour' commit -m 'test' 2>&1", 1)
    assert_match "Not in a git repository", output
  end
end
