import React from 'react';
import { HardDrive } from 'lucide-react';

interface FloppyDiskIconProps {
  size?: 'small' | 'medium' | 'large';
  className?: string;
  theme?: 'light' | 'dark';
  isAnimated?: boolean;
}

const FloppyDiskIcon: React.FC<FloppyDiskIconProps> = ({
  size = 'medium',
  className = '',
  theme,
  isAnimated = false
}) => {
  // Get theme from data attribute if not provided
  const currentTheme = theme || document.documentElement.getAttribute('data-theme') || 'dark';

  const sizeClasses = {
    small: 'w-4 h-4',
    medium: 'w-6 h-6',
    large: 'w-8 h-8'
  };

  const logoSrc = currentTheme === 'dark'
    ? '/Floppy_disk_icon_LLM_white.png'
    : '/Floppy_disk_icon_LLM_black.png';

  const animationClasses = isAnimated
    ? 'animate-pulse hover:animate-bounce transition-transform duration-200 hover:scale-110'
    : 'transition-transform duration-200 hover:scale-105';

  return (
    <div className={`inline-flex items-center justify-center ${className}`}>
      <img
        src={logoSrc}
        alt="Insert Model Disk"
        className={`${sizeClasses[size]} object-contain ${animationClasses}`}
        title="Insert Model Disk - Select your AI model"
        onError={(e) => {
          const target = e.target as HTMLImageElement;
          // If floppy disk image fails, show HardDrive icon as fallback
          target.style.display = 'none';
          const fallbackIcon = target.nextElementSibling as HTMLElement;
          if (fallbackIcon) {
            fallbackIcon.classList.remove('hidden');
          }
        }}
      />
      <HardDrive
        className={`${sizeClasses[size]} text-[var(--text-secondary)] hidden ${animationClasses}`}
        title="Select Model"
      />
    </div>
  );
};

export default FloppyDiskIcon;