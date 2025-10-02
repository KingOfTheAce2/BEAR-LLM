# BEAR AI Installation Guide

## 📦 Installation Options

### Option 1: Windows Installer (Recommended for Users)

1. **Download the installer:**
   - `BEAR-AI-Setup-1.0.0.exe` (Windows)

2. **Run the installer:**
   - Double-click the downloaded file
   - Follow the installation wizard
   - Choose installation directory (default: `C:\Program Files\BEAR AI`)

3. **Launch BEAR AI:**
   - Desktop shortcut created automatically
   - Start menu entry: `BEAR AI Assistant`
   - System tray icon for quick access

### Option 2: Build from Source (For Developers)

#### Prerequisites:
- Node.js 18+ and npm
- Rust 1.70+ (install from https://rustup.rs)
- Windows Build Tools (for Windows)
- Git

#### Build Steps:

```bash
# Clone the repository
git clone https://github.com/yourusername/BEAR-AI.git
cd BEAR-AI

# Install dependencies
npm install

# Build the application
npm run tauri build

# The installer will be created in:
# src-tauri/target/release/bundle/
```

## 🚀 Running BEAR AI

### Desktop Application (Primary)
- Launch from Start Menu or Desktop shortcut
- The app runs as a native Windows application
- System tray icon shows status
- Full GPU acceleration support

### Web Browser Interface (Secondary)
After starting the desktop app:
1. Open any web browser
2. Navigate to: `http://localhost:11434`
3. Full interface available in browser
4. Still 100% local - no internet required

### Command Line Interface
```bash
# Run in development mode
npm run tauri dev

# Run production build
./BEAR-AI.exe

# With custom port for web interface
./BEAR-AI.exe --port 8080
```

## 🔧 Configuration

### First Run Setup
1. **System Check**: Automatic GPU/CPU detection
2. **Model Selection**: Browse and download compatible models
3. **Privacy Settings**: Configure PII protection levels
4. **Storage Location**: Choose where to store models (default: `%USERPROFILE%\.bear-ai\models`)

### Model Storage Locations
- Windows: `C:\Users\[Username]\.bear-ai\models\`
- Linux: `~/.bear-ai/models/`
- macOS: `~/Library/Application Support/bear-ai/models/`

## 🖥️ System Requirements

### Minimum:
- OS: Windows 10/11 (64-bit)
- CPU: 4 cores, 3.0 GHz
- RAM: 8 GB
- Storage: 20 GB free space
- GPU: Optional (CPU-only mode available)

### Recommended:
- OS: Windows 11 (64-bit)
- CPU: 8+ cores, 3.5+ GHz
- RAM: 32 GB
- Storage: 100 GB SSD
- GPU: NVIDIA RTX 3060+ (12GB VRAM)

### Optimal:
- GPU: NVIDIA RTX 4090 (24GB VRAM)
- RAM: 64 GB
- Can run multiple 13B-30B models simultaneously

## 🛡️ Security & Privacy

### Local Operation
- ✅ No internet connection required after model download
- ✅ All processing happens on YOUR hardware
- ✅ No telemetry or data collection
- ✅ No cloud services or external APIs
- ✅ PII automatically detected and removed

### Firewall Configuration
If you want to access the web interface from other devices on your network:
```bash
# Windows Firewall
netsh advfirewall firewall add rule name="BEAR AI" dir=in action=allow protocol=TCP localport=11434

# Or use Windows Defender Firewall GUI
# Add inbound rule for port 11434
```

## 🚨 Troubleshooting

### GPU Not Detected
1. Update NVIDIA/AMD drivers
2. Install CUDA Toolkit 12.1+ (for NVIDIA)
3. Restart BEAR AI

### Model Won't Load
1. Check available VRAM in System Monitor
2. Close other GPU applications
3. Try smaller or quantized model version

### Web Interface Not Accessible
1. Check if desktop app is running
2. Verify firewall settings
3. Try: `http://127.0.0.1:11434` instead of localhost

### High Memory Usage
1. Use quantized models (Q4, Q5)
2. Reduce context length in settings
3. Enable memory limit in preferences

## 📞 Support

- Documentation: `http://localhost:11434/docs` (when app is running)
- Issues: Create a local log at `%APPDATA%\bear-ai\logs\`
- License: See LICENSE file

## 🔄 Updates

BEAR AI checks for updates automatically (can be disabled in settings).
Updates are downloaded but require user confirmation to install.

---

**Remember:** BEAR AI is designed for complete privacy. Your data NEVER leaves your computer!