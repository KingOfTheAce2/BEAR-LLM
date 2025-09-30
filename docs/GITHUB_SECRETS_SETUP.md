# GitHub Secrets Setup for Auto-Updater

## Required GitHub Secrets

You need to add these two secrets to your GitHub repository for the auto-updater to work:

### 1. TAURI_SIGNING_PRIVATE_KEY

This should contain the **entire contents** of your private key file:
- File location: `updater-keys/bear_ai.key`
- **IMPORTANT**: This is your private key - keep it secret!

### 2. TAURI_SIGNING_PRIVATE_KEY_PASSWORD

This is the password you just entered when generating the keys.

## How to Add Secrets to GitHub

1. Go to your GitHub repository
2. Click on **Settings** (in the repository, not your profile)
3. In the left sidebar, click **Secrets and variables** → **Actions**
4. Click **New repository secret**
5. For each secret:
   - Enter the name exactly as shown above
   - Paste the value
   - Click **Add secret**

## Security Notes

- ⚠️ **NEVER** commit these values to your repository
- ⚠️ **NEVER** share your private key or password
- ✅ GitHub Secrets are encrypted and only available to GitHub Actions
- ✅ The private key has already been added to .gitignore

## Next Steps

After adding the secrets:

1. **Create a GitHub Gist** for hosting `latest.json`:
   - Go to https://gist.github.com/
   - Create a new gist named `bear-ai-latest.json`
   - Use the content from `latest.json.example` as a template

2. **Update the endpoint** in `src-tauri/tauri.conf.json`:
   - Replace the current endpoint URL with your Gist's raw URL
   - Format: `https://gist.githubusercontent.com/YOUR_USERNAME/GIST_ID/raw/bear-ai-latest.json`

3. **Test the release workflow**:
   - Create a new tag: `git tag v1.0.5`
   - Push it: `git push origin v1.0.5`
   - GitHub Actions will automatically build and sign the release

## Verification

To verify everything is working:
1. Check GitHub Actions for successful workflow runs
2. Look for `.sig` signature files in the release assets
3. Test the auto-updater by running an older version of the app

---

**Your Public Key** (already added to tauri.conf.json):
```
dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDQwOEU0NzhFOEQzQUI5NTEKV1JScnVUcU5qa2VPUUtOUU1ZWUtlWnl1RlhDSHRPMWVaTVB4L2I2MDN0UlN0aG5ybXZ1WUYzZ1kK
```

**Key ID**: 408E478E8D3AB951