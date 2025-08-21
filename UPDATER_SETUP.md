# Auto Updater Setup Guide

This guide will help you set up the auto updater for your Tauri app with GitHub hosting.

## Prerequisites

1. A GitHub repository for your app
2. Tauri CLI installed (`pnpm add -g @tauri-apps/cli`)

## Step 1: Generate Signing Keys

You need to generate signing keys for the updater to work securely:

```bash
# Generate a new key pair
pnpm tauri signer generate -- -w ~/.tauri/myapp.key

# This will output something like:
# Private key written to ~/.tauri/myapp.key
# Public key: dW50cnVzdGVkIGNvbW1lbnQ6...
```

**Important**: Save both the private key file and the public key string safely!

## Step 2: Update Configuration

1. Replace the `pubkey` in `src-tauri/tauri.conf.json` with your generated public key:

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/YOUR_USERNAME/YOUR_REPO/releases/latest/download/latest.json"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

2. Update the GitHub repository URL in the endpoint to match your repository.

## Step 3: Set up GitHub Secrets

Add these secrets to your GitHub repository (Settings > Secrets and variables > Actions):

1. **TAURI_PRIVATE_KEY**: The content of your private key file (e.g., `~/.tauri/myapp.key`)
2. **TAURI_KEY_PASSWORD**: The password you used when generating the key (if any)

## Step 4: Create a Release

To trigger the auto updater:

1. Create and push a new tag:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

2. The GitHub Action will automatically:
   - Build your app for all platforms
   - Create a GitHub release
   - Generate the `latest.json` file needed for updates

## Step 5: Test the Updater

1. Install the current version of your app
2. Create a new release with a higher version number
3. Open your app - it should automatically check for updates and show a notification

## How It Works

1. **On App Start**: The app checks the GitHub endpoint for updates
2. **Update Available**: Shows a green notification banner with update details
3. **User Clicks Update**: Downloads and installs the update automatically
4. **Restart**: The app restarts with the new version

## Troubleshooting

- **No updates detected**: Check that your endpoint URL is correct and the release exists
- **Signature verification failed**: Ensure your public/private keys match
- **Download fails**: Check network connectivity and GitHub release assets

## Manual Testing

You can test the updater manually:

```bash
# Build with updater enabled
pnpm tauri build

# Create a release on GitHub with the built files
# Then test with a lower version number in tauri.conf.json
```

## Security Notes

- Keep your private key secure and never commit it to version control
- The public key in the config is safe to commit
- Updates are cryptographically signed and verified before installation
