import React from 'react';
import { Scale } from 'lucide-react';

interface BearLogoProps {
  size?: 'small' | 'medium' | 'large';
  className?: string;
  theme?: 'light' | 'dark';
}

const BearLogo: React.FC<BearLogoProps> = ({
  size = 'medium',
  className = '',
  theme
}) => {
  // Get theme from data attribute if not provided
  const currentTheme = theme || document.documentElement.getAttribute('data-theme') || 'dark';

  const sizeClasses = {
    small: 'w-6 h-6',
    medium: 'w-16 h-16',
    large: 'w-20 h-20'
  };

  const logoSrc = currentTheme === 'dark'
    ? '/BEAR_AI_LLM_Logo_white.png'
    : '/BEAR_AI_LLM_Logo_black.png';

  const fallbackSrc = '/BEAR_AI_logo.png';

  return (
    <img
      src={logoSrc}
      alt="BEAR AI"
      className={`${sizeClasses[size]} object-contain transition-opacity duration-300 ${className}`}
      onError={(e) => {
        const target = e.target as HTMLImageElement;
        // Try fallback logo first
        if (target.src !== fallbackSrc) {
          target.src = fallbackSrc;
        } else {
          // If fallback also fails, hide image and show icon
          target.style.display = 'none';
          const fallbackIcon = target.nextElementSibling as HTMLElement;
          if (fallbackIcon) {
            fallbackIcon.classList.remove('hidden');
          }
        }
      }}
    />
  );
};

export default BearLogo;