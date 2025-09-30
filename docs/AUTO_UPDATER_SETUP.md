# BEAR AI Auto-Updater Setup Guide

## Overview

The BEAR AI application now includes an automatic update system powered by Tauri's built-in updater. This guide will help you set up and configure the auto-updater for your releases.

## Prerequisites

Before setting up auto-updates, you need:

1. **Signing Keys**: Required for verifying update authenticity
2. **GitHub Repository**: For hosting releases
3. **Public JSON Endpoint**: For hosting the update manifest (latest.json)

## Step 1: Generate Signing Keys

### IMPORTANT: Keys have been added to .gitignore

Run one of these commands to generate your signing keys:

```bash
# Option 1: Using npm
npm run tauri signer generate -- -w ./updater-keys/bear_ai.key

# Option 2: Using npx
npx @tauri-apps/cli signer generate -w ./updater-keys/bear_ai.key

# Option 3: Using cargo (requires cargo-tauri)
cargo tauri signer generate -w ./updater-keys/bear_ai.key
```

This generates:
- `bear_ai.key` - **PRIVATE KEY** (Never commit to Git!)
- `bear_ai.key.pub` - **PUBLIC KEY** (Add to tauri.conf.json)

## Step 2: Configure tauri.conf.json

Update the `pubkey` field in `src-tauri/tauri.conf.json`:

```json
"updater": {
  "active": true,
  "endpoints": [
    "YOUR_LATEST_JSON_URL_HERE"
  ],
  "dialog": true,
  "pubkey": "YOUR_PUBLIC_KEY_HERE"
}
```

## Step 3: Set Up GitHub Secrets

Add these secrets to your GitHub repository (Settings → Secrets → Actions):

1. `TAURI_SIGNING_PRIVATE_KEY` - Contents of `bear_ai.key`
2. `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` - Password used when generating keys

## Step 4: Host latest.json

You have several options for hosting the latest.json file:

### Option A: GitHub Gist (Recommended)

1. Create a new Gist at https://gist.github.com/
2. Name it `bear-ai-latest.json`
3. Copy content from `latest.json.example`
4. Update the endpoint in tauri.conf.json:
   ```json
   "endpoints": [
     "https://gist.githubusercontent.com/YOUR_USERNAME/GIST_ID/raw/bear-ai-latest.json"
   ]
   ```

### Option B: GitHub Pages

1. Create a `gh-pages` branch
2. Add `latest.json` to the root
3. Enable GitHub Pages in repository settings
4. Update the endpoint:
   ```json
   "endpoints": [
     "https://YOUR_USERNAME.github.io/bear-ai-llm/latest.json"
   ]
   ```

### Option C: Custom CDN

Host the file on any public CDN or server that supports HTTPS.

## Step 5: Release Process

### Creating a New Release

1. **Update Version**:
   ```bash
   npm version patch  # or minor/major
   ```

2. **Tag and Push**:
   ```bash
   git tag v1.0.5
   git push origin v1.0.5
   ```

3. **GitHub Actions**: The workflow will automatically:
   - Build for all platforms
   - Sign the artifacts
   - Create a GitHub Release
   - Generate update signatures

4. **Update latest.json**: After release is complete:
   - Copy the generated signatures from the release assets
   - Update your hosted latest.json with:
     - New version number
     - Release notes
     - Download URLs
     - Signatures for each platform

## How Updates Work

1. **Check on Startup**: The app checks for updates 5 seconds after launch
2. **User Notification**: If an update is available, a notification appears
3. **Download & Install**: User can choose to install immediately or later
4. **Auto Restart**: After installation, the app restarts automatically

## Update UI Components

- **UpdateNotification.tsx**: Shows update availability
- **updater.ts**: Handles update logic
- **App.tsx**: Initializes update checking

## Troubleshooting

### Common Issues

1. **"Invalid signature"**:
   - Ensure the public key in tauri.conf.json matches your generated key
   - Verify GitHub secrets are set correctly

2. **"Update not found"**:
   - Check that latest.json is accessible at the configured URL
   - Verify version number in latest.json is higher than current

3. **"Download failed"**:
   - Ensure release assets are publicly accessible
   - Check that URLs in latest.json are correct

### Testing Updates Locally

1. Build a test version with a lower version number
2. Host a test latest.json with a higher version
3. Run the app and verify update notification appears

## Security Considerations

- **Never commit private keys** to version control
- Use strong passwords for key generation
- Regularly rotate signing keys if compromised
- Verify HTTPS is used for all update endpoints

## Additional Resources

- [Tauri Updater Documentation](https://tauri.app/v1/guides/distribution/updater/)
- [GitHub Actions for Tauri](https://github.com/tauri-apps/tauri-action)
- [Signing Updates](https://tauri.app/v1/guides/distribution/sign)

## Support

For issues with the auto-updater, please check:
1. GitHub Actions logs for build errors
2. Browser console for update check errors
3. Tauri logs for installation issues

---

**Note**: Remember to test the update process thoroughly before releasing to production!