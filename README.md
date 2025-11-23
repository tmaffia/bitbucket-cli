# bbcli v0.3.3

Bitbucket CLI is a command line interface for interacting with Bitbucket.

## Installation

### macOS

To install the latest version, run:

```sh
curl -fsSL https://raw.githubusercontent.com/tmaffia/bitbucket-cli/main/scripts/install.sh | sh
```

## Features

- **Authentication**: Secure login using Bitbucket API Tokens (stored in system keyring).
- **Pull Requests**: List, view, diff, and comment on pull requests.
- **Configuration**: Manage multiple profiles and default settings.
- **Git Integration**: Auto-detects repository and branch from current directory.

## Configuration and Auth Setup

### 1. Create an API Token

To use this CLI, you need an API Token from your Atlassian account:

1. Log in to [id.atlassian.com](https://id.atlassian.com/manage-profile/security/api-tokens).
2. Click **Create API token**.
3. Give it a label (e.g., "bb-cli").
4. Click **Create**.
5. **Copy the generated token**. You will need this for the CLI login.

### 2. Installation and Setup

1. **Install the binary:**

    **macOS / Linux:**
    Use the install script to download the pre-built binary:

    ```bash
    curl -fsSL https://raw.githubusercontent.com/tmaffia/bitbucket-cli/main/scripts/install.sh | sh
    ```

    **Windows:**
    Clone the repository and install via Cargo:

    ```bash
    cargo install --path .
    ```

2. Authenticate with your Bitbucket account:

    ```bash
    bb auth login
    ```

    - Enter your Bitbucket email.
    - Enter the **API Token** you generated in step 1.

3. Initialize the configuration in your bitbucket repository folder:

    ```bash
    bb config init
    ```

## Development

For contributing to this repository, you can set up the pre-push hooks (recommended):

```sh
cp scripts/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```
