import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { ask, message } from '@tauri-apps/plugin-dialog';

export interface UpdateInfo {
  version: string;
  notes?: string;
  date?: string;
}

export async function checkForUpdates(silent = false): Promise<UpdateInfo | null> {
  try {
    console.log('Checking for updates...');
    const update = await check();

    if (update?.available) {
      console.log(`Update available: ${update.version}`);

      const updateInfo: UpdateInfo = {
        version: update.version,
        notes: update.body || 'No release notes available',
        date: update.date
      };

      if (!silent) {
        const shouldInstall = await ask(
          `A new version (${update.version}) of BEAR AI is available!\n\n${update.body || 'No release notes available'}\n\nWould you like to install it now?`,
          {
            title: 'Update Available',
            kind: 'info',
            okLabel: 'Install & Restart',
            cancelLabel: 'Later'
          }
        );

        if (shouldInstall) {
          await installUpdate(update);
        }
      }

      return updateInfo;
    } else {
      console.log('Application is up to date');
      if (!silent) {
        await message('BEAR AI is up to date!', {
          title: 'No Updates',
          kind: 'info'
        });
      }
      return null;
    }
  } catch (error) {
    console.error('Update check failed:', error);
    if (!silent) {
      await message(`Failed to check for updates: ${error}`, {
        title: 'Update Check Failed',
        kind: 'error'
      });
    }
    return null;
  }
}

async function installUpdate(update: any) {
  try {
    console.log('Starting download and install...');

    // Show progress message
    await message('Downloading update... Please wait.', {
      title: 'Installing Update',
      kind: 'info'
    });

    // Download and install the update
    await update.downloadAndInstall();

    // Relaunch the application
    await relaunch();
  } catch (error) {
    console.error('Failed to install update:', error);
    await message(`Failed to install update: ${error}`, {
      title: 'Update Failed',
      kind: 'error'
    });
  }
}

export async function checkForUpdatesOnStartup() {
  // Check for updates 5 seconds after app start
  setTimeout(() => {
    checkForUpdates(true);
  }, 5000);
}

export async function manualUpdateCheck() {
  return checkForUpdates(false);
}