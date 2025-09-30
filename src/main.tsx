import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import ErrorBoundary from './components/ErrorBoundary';
import './index.css';

// Add error handler for uncaught errors during initialization
window.addEventListener('error', (event) => {
  console.error('🔴 Initialization error:', event.error);
});

window.addEventListener('unhandledrejection', (event) => {
  console.error('🔴 Unhandled promise rejection:', event.reason);
});

// Log successful initialization
console.log('🚀 BEAR AI initializing...');

const rootElement = document.getElementById('root');
if (!rootElement) {
  console.error('🔴 Fatal: Root element not found!');
  document.body.innerHTML = '<div style="padding: 20px; font-family: system-ui;">Fatal Error: Root element not found. Please reinstall the application.</div>';
} else {
  try {
    ReactDOM.createRoot(rootElement).render(
      <React.StrictMode>
        <ErrorBoundary>
          <App />
        </ErrorBoundary>
      </React.StrictMode>,
    );
    console.log('✅ BEAR AI React app mounted successfully');

    // Signal to failsafe that the app loaded successfully
    if (typeof window.__BEAR_AI_LOADED__ === 'function') {
      window.__BEAR_AI_LOADED__();
    }
  } catch (error) {
    console.error('🔴 Fatal error rendering app:', error);
    rootElement.innerHTML = '<div style="padding: 20px; font-family: system-ui;">Fatal Error: Failed to render application. Please check console for details.</div>';
  }
}