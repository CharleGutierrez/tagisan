# Bun Installation Guide

Complete guide for installing Bun on all platforms.

## Windows Installation

### PowerShell (Recommended)
```powershell
# Install Bun
powershell -c "irm bun.sh/install.ps1|iex"

# Verify installation
bun --version

# If not found, add to PATH
$env:Path += ";$env:USERPROFILE\.bun\bin"

# Make permanent (run as admin or add to profile)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\.bun\bin", "User")
```

### Scoop
```powershell
scoop install bun
```

### npm
```bash
npm install -g bun
```

### Winget (Known Issues)
```powershell
# Works but may have PATH issues
winget install Oven-sh.Bun
```

## macOS Installation

### curl (Recommended)
```bash
curl -fsSL https://bun.sh/install | bash
source ~/.zshrc  # or ~/.bashrc
```

### Homebrew
```bash
brew install oven-sh/bun/bun
```

### npm
```bash
npm install -g bun
```

## Linux Installation

### curl (All distros)
```bash
curl -fsSL https://bun.sh/install | bash
source ~/.bashrc
```

### Specific Version
```bash
curl -fsSL https://bun.sh/install | bash -s "bun-v1.1.0"
```

### npm
```bash
npm install -g bun
```

## Docker

```dockerfile
FROM oven/bun:latest

WORKDIR /app
COPY package.json bun.lockb ./
RUN bun install --frozen-lockfile

COPY . .
CMD ["bun", "run", "start"]
```

## Upgrading Bun

```bash
bun upgrade

# Specific version
bun upgrade --to 1.2.0

# Canary (latest dev)
bun upgrade --canary
```

## Uninstalling

### Windows
```powershell
rm -r $env:USERPROFILE\.bun
# Remove from PATH manually
```

### macOS/Linux
```bash
rm -rf ~/.bun
# Remove from .bashrc/.zshrc
```

## Verification

```bash
# Check version
bun --version

# Check installation path
which bun  # Unix
where bun  # Windows

# Test with simple script
bun -e "console.log('Bun works!')"
```

## Troubleshooting

### PATH Issues

**Windows PowerShell:**
```powershell
# Add to current session
$env:Path = "$env:USERPROFILE\.bun\bin;" + $env:Path

# Add to profile (permanent)
Add-Content $PROFILE "`n`$env:Path = `"`$env:USERPROFILE\.bun\bin;`" + `$env:Path"
```

**Unix/macOS:**
```bash
# Add to .bashrc or .zshrc
export BUN_INSTALL="$HOME/.bun"
export PATH="$BUN_INSTALL/bin:$PATH"

# Reload
source ~/.bashrc
```

### Permission Issues (Linux)

```bash
# If permission denied
sudo chown -R $USER:$USER ~/.bun
```

### Reinstall Clean

```bash
# Remove existing
rm -rf ~/.bun

# Fresh install
curl -fsSL https://bun.sh/install | bash
```
