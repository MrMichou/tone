# tone - Terminal UI for OpenNebula

A terminal user interface for navigating, observing, and managing OpenNebula cloud resources. Similar to [taws](https://github.com/huseyinbabal/taws) for AWS and tgcp for GCP.

## Features

- Browse and manage OpenNebula resources:
  - Virtual Machines (VMs)
  - Hosts
  - Datastores
  - Virtual Networks
  - Images
  - VM Templates
  - Clusters
  - Users/Groups
- Vim-style keyboard navigation
- Filter and search resources
- View detailed JSON representations
- Perform VM actions (resume, suspend, stop, power off, reboot, terminate)
- Read-only mode for safe browsing

## Installation

### Homebrew (macOS / Linux)

```bash
brew install MrMichou/tap/tone
```

### Shell installer (macOS / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/MrMichou/tone/master/install.sh | sh
```

### Debian / Ubuntu

Download the `.deb` package from the [latest release](https://github.com/MrMichou/tone/releases/latest):

```bash
sudo dpkg -i tone_amd64.deb
```

### Scoop (Windows)

```powershell
scoop install https://github.com/MrMichou/tone/releases/latest/download/tone.json
```

### Download binaries

Pre-built binaries for Linux (x86_64, aarch64), macOS (x86_64, Apple Silicon), and Windows (x86_64) are available on the [releases page](https://github.com/MrMichou/tone/releases/latest).

### From source

```bash
cargo install --path .
# or
cargo build --release
```

## Configuration

### Authentication

tone uses OpenNebula's standard authentication methods:

1. **Environment variable** `ONE_AUTH` - Path to auth file or `username:password` string
2. **Config file** `~/.one/one_auth` - Contains `username:password`

### Endpoint

Set the OpenNebula XML-RPC endpoint:

1. **Command line**: `--endpoint http://your-one-server:2633/RPC2`
2. **Environment variable**: `ONE_XMLRPC=http://your-one-server:2633/RPC2`

Default: `http://localhost:2633/RPC2`

## Usage

```bash
# Connect to default endpoint
tone

# Connect to specific endpoint
tone --endpoint http://opennebula.example.com:2633/RPC2

# Read-only mode (no write operations)
tone --readonly

# Enable debug logging
tone --log-level debug
```

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` | Move down |
| `k` / `Up` | Move up |
| `gg` | Go to top |
| `G` | Go to bottom |
| `Ctrl+f` | Page down |
| `Ctrl+b` | Page up |
| `b` / `Backspace` | Go back |

### Commands

| Key | Action |
|-----|--------|
| `:` | Open command mode |
| `/` | Filter items |
| `Enter` / `d` | View details (JSON) |
| `R` | Refresh |
| `?` | Show help |
| `q` | Quit |

### VM Actions

| Key | Action |
|-----|--------|
| `r` | Resume VM |
| `u` | Suspend VM |
| `s` | Stop VM |
| `S` | Power off VM |
| `h` | Hold VM |
| `l` | Release VM |
| `Ctrl+d` | Terminate VM (destructive) |

### Available Resources

Switch between resources using command mode (`:resource-name`):

- `:one-vms` - Virtual Machines
- `:one-hosts` - Hosts
- `:one-datastores` - Datastores
- `:one-vnets` - Virtual Networks
- `:one-images` - Images
- `:one-templates` - VM Templates
- `:one-clusters` - Clusters
- `:one-users` - Users
- `:one-groups` - Groups
- `:one-zones` - Zones

## Logs

Logs are stored at:
- Linux/macOS: `~/.config/tone/tone.log`
- Fallback: `~/.tone/tone.log`

## License

MIT
