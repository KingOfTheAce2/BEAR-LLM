// Type definitions for custom window properties

interface Window {
  /**
   * Failsafe function called when the application successfully loads
   * This signals to the error detection system that everything is working
   */
  __BEAR_AI_LOADED__?: () => void;
}
