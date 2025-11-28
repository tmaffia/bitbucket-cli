# bbcli v0.3.8

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

## Global Flags

- `--json`: Output results in JSON format (available for `list` commands).

## Usage

### Repositories

List repositories in your workspace:

```bash
bb repo list

# List with a custom limit (default is 100)
bb repo list --limit 20
```

### Pull Requests

List pull requests:

```bash
bb pr list
```

View a pull request (auto-detected from branch or by ID):

```bash
bb pr view
bb pr view 123
```

**View Diff with Filtering:**

You can filter the diff by file patterns or size.

**Inferred Context (Current Branch):**

```bash
# Filter by file extension
bb pr diff "*.rs"

# Filter by directory
bb pr diff src/commands/
```

**Manual Context (Explicit ID):**

```bash
# Filter by specific file for PR #123
bb pr diff 123 src/main.rs

# Skip large files for PR #123
bb pr diff 123 --max-diff-size 100
```

**Review a Pull Request:**

Start an interactive review or submit immediately with flags.

**Inferred Context (Current Branch):**

```bash
# Interactive mode
bb pr review

# Approve immediately
bb pr review --approve
```

**Manual Context (Explicit ID):**

```bash
# Interactive mode for PR #123
bb pr review 123

# Request changes for PR #123
bb pr review 123 --request-changes

# Comment on PR #123
bb pr review 123 --comment --body "Great work!"
```

**Override Repository:**

You can run any command against a specific repository using `-R`:

```bash
# List PRs in a different repo
bb pr list -R my-workspace/other-repo

# View PR #123 in a different repo
bb pr view 123 -R my-workspace/other-repo
```

### Configuration

View your current configuration (active profile and local overrides):

```bash
bb config list
```

Set a configuration value:

```bash
# Set the active profile (user)
bb config set user <PROFILE_NAME>

# Set workspace for the default profile
bb config set profile.default.workspace <WORKSPACE_NAME>
```

## Development

For contributing to this repository, you can set up the pre-push hooks (recommended):

```sh
cp scripts/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```
