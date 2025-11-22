# bitbucket-cli

Bitbucket CLI is a command line interface for interacting with Bitbucket.

## Installation

### macOS

To install the latest version, run:

```sh
curl -fsSL https://raw.githubusercontent.com/tmaffia/bbcli/main/scripts/install.sh | sh
```

## Development

To set up the pre-push hooks (recommended):

```sh
cp scripts/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

## [Requirements](docs/requirements.md)
