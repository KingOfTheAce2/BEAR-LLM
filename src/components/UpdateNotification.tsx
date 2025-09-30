import React, { useState, useEffect } from 'react';
import { Download, X, AlertCircle, CheckCircle } from 'lucide-react';
import { checkForUpdates, UpdateInfo } from '../utils/updater';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export function UpdateNotification() {
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    // Check for updates on component mount
    const checkUpdates = async () => {
      try {
        const update = await check();
        if (update?.available) {
          setUpdateInfo({
            version: update.version,
            notes: update.body || 'No release notes available',
            date: update.date
          });
          setUpdateAvailable(true);
        }
      } catch (err) {
        console.error('Failed to check for updates:', err);
      }
    };

    // Check after 10 seconds
    const timer = setTimeout(checkUpdates, 10000);
    return () => clearTimeout(timer);
  }, []);

  const handleInstallUpdate = async () => {
    try {
      setDownloading(true);
      setError(null);

      const update = await check();
      if (update?.available) {
        // Track download progress
        const progressInterval = setInterval(() => {
          setProgress(prev => {
            if (prev >= 90) {
              clearInterval(progressInterval);
              return 90;
            }
            return prev + 10;
          });
        }, 500);

        await update.downloadAndInstall();

        clearInterval(progressInterval);
        setProgress(100);

        // Wait a moment before restarting
        setTimeout(async () => {
          await relaunch();
        }, 1500);
      }
    } catch (err) {
      setError(`Failed to install update: ${err}`);
      setDownloading(false);
    }
  };

  const handleDismiss = () => {
    setDismissed(true);
    setTimeout(() => {
      setUpdateAvailable(false);
    }, 300);
  };

  if (!updateAvailable || dismissed) {
    return null;
  }

  return (
    <div className={`fixed bottom-4 right-4 max-w-sm bg-[var(--bg-primary)] border border-[var(--border-primary)] rounded-lg shadow-xl transition-all duration-300 ${dismissed ? 'opacity-0 translate-y-2' : 'opacity-100 translate-y-0'}`}>
      <div className="p-4">
        <div className="flex items-start justify-between mb-3">
          <div className="flex items-center gap-2">
            <AlertCircle className="w-5 h-5 text-[var(--accent)]" />
            <h3 className="font-semibold text-[var(--text-primary)]">Update Available</h3>
          </div>
          <button
            onClick={handleDismiss}
            className="text-[var(--text-tertiary)] hover:text-[var(--text-secondary)] transition-colors"
            disabled={downloading}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <p className="text-sm text-[var(--text-secondary)] mb-1">
          Version {updateInfo?.version} is now available
        </p>

        {updateInfo?.notes && (
          <div className="text-xs text-[var(--text-tertiary)] mb-3 max-h-20 overflow-y-auto">
            {updateInfo.notes}
          </div>
        )}

        {error && (
          <div className="text-xs text-red-500 mb-2">
            {error}
          </div>
        )}

        {downloading ? (
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm text-[var(--text-secondary)]">
              <Download className="w-4 h-4 animate-pulse" />
              <span>Installing update...</span>
            </div>
            <div className="w-full bg-[var(--bg-secondary)] rounded-full h-2">
              <div
                className="bg-[var(--accent)] h-2 rounded-full transition-all duration-300"
                style={{ width: `${progress}%` }}
              />
            </div>
            {progress === 100 && (
              <div className="flex items-center gap-1 text-xs text-green-500">
                <CheckCircle className="w-3 h-3" />
                <span>Restarting application...</span>
              </div>
            )}
          </div>
        ) : (
          <div className="flex gap-2">
            <button
              onClick={handleInstallUpdate}
              className="flex-1 px-3 py-2 bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white rounded-lg text-sm font-medium transition-colors flex items-center justify-center gap-2"
            >
              <Download className="w-4 h-4" />
              Install Now
            </button>
            <button
              onClick={handleDismiss}
              className="flex-1 px-3 py-2 bg-[var(--bg-secondary)] hover:bg-[var(--hover-bg)] text-[var(--text-secondary)] rounded-lg text-sm font-medium transition-colors"
            >
              Later
            </button>
          </div>
        )}
      </div>
    </div>
  );
}