---
title: Installation
description: All methods to install conclaude including shell scripts, npm, Nix, and building from source.
---

conclaude supports multiple installation methods. Choose the one that best fits your workflow.

## Shell Script (Recommended)

The fastest way to install on Linux or macOS:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-installer.sh | sh
```

This downloads the latest release binary and installs it to `~/.cargo/bin`.

## PowerShell (Windows)

For Windows users:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-installer.ps1 | iex"
```

## npm Package

Install as an npm package:

```bash
npm install conclaude
```

Or globally:

```bash
npm install -g conclaude
```

## Manual Binary Download

Download the pre-built binary for your platform from [GitHub Releases](https://github.com/connerohnesorge/conclaude/releases/latest).

### Linux x86_64

```bash
curl -L -o conclaude.tar.xz \
  https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-x86_64-unknown-linux-gnu.tar.xz
tar -xf conclaude.tar.xz
chmod +x conclaude
sudo mv conclaude /usr/local/bin/
```

### Linux ARM64

```bash
curl -L -o conclaude.tar.xz \
  https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-aarch64-unknown-linux-gnu.tar.xz
tar -xf conclaude.tar.xz
chmod +x conclaude
sudo mv conclaude /usr/local/bin/
```

### Linux x86_64 (musl)

For Alpine Linux and other musl-based distributions:

```bash
curl -L -o conclaude.tar.xz \
  https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-x86_64-unknown-linux-musl.tar.xz
tar -xf conclaude.tar.xz
chmod +x conclaude
sudo mv conclaude /usr/local/bin/
```

### macOS (Apple Silicon)

```bash
curl -L -o conclaude.tar.xz \
  https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-aarch64-apple-darwin.tar.xz
tar -xf conclaude.tar.xz
chmod +x conclaude
sudo mv conclaude /usr/local/bin/
```

### macOS (Intel)

```bash
curl -L -o conclaude.tar.xz \
  https://github.com/connerohnesorge/conclaude/releases/latest/download/conclaude-x86_64-apple-darwin.tar.xz
tar -xf conclaude.tar.xz
chmod +x conclaude
sudo mv conclaude /usr/local/bin/
```

### Available Platforms

| Platform | Architecture | File |
|----------|--------------|------|
| Linux | x86_64 | `conclaude-x86_64-unknown-linux-gnu.tar.xz` |
| Linux | ARM64 | `conclaude-aarch64-unknown-linux-gnu.tar.xz` |
| Linux (musl) | x86_64 | `conclaude-x86_64-unknown-linux-musl.tar.xz` |
| macOS | Apple Silicon | `conclaude-aarch64-apple-darwin.tar.xz` |
| macOS | Intel | `conclaude-x86_64-apple-darwin.tar.xz` |
| Windows | x64 | `conclaude-x86_64-pc-windows-msvc.zip` |

## Nix Flake

### Direct Usage

Run conclaude directly without installation:

```bash
nix run github:connerohnesorge/conclaude -- --help
```

### Development Shell

Add conclaude to your project's development shell:

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    conclaude.url = "github:connerohnesorge/conclaude";
  };

  outputs = { self, nixpkgs, conclaude, ... }:
    let
      system = "x86_64-linux"; # or your system
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.default = pkgs.mkShell {
        packages = [
          conclaude.packages.${system}.default
          # your other packages...
        ];

        shellHook = ''
          echo "conclaude available"
          conclaude --version
        '';
      };
    };
}
```

Enter the development shell:

```bash
nix develop
```

## Cargo (From Crates.io)

If you have Rust installed:

```bash
cargo install conclaude
```

## Build From Source

Clone and build the repository:

```bash
git clone https://github.com/connerohnesorge/conclaude.git
cd conclaude
cargo build --release
```

The binary will be at `target/release/conclaude`. Copy it to your PATH:

```bash
sudo cp target/release/conclaude /usr/local/bin/
```

### Development Setup

For development with hot reloading:

```bash
git clone https://github.com/connerohnesorge/conclaude.git
cd conclaude
cargo install --path .
```

## Verify Installation

After installation, verify conclaude is working:

```bash
conclaude --version
```

You should see output like:

```
conclaude 0.1.0
```

## Next Steps

- **[Getting Started](/conclaude/guides/getting-started)** — Initialize conclaude in your project
- **[Hooks Overview](/conclaude/guides/hooks)** — Learn about the hook system
- **[CLI Reference](/conclaude/reference/cli)** — All available commands
