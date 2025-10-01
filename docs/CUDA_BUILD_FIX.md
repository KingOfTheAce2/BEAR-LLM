# CUDA Build Fix for Windows CI/CD

## Problem Summary

The Windows GitHub Actions build was failing with:

```
CMake Error at ggml/src/ggml-cuda/CMakeLists.txt:183 (message):
  CUDA Toolkit not found
```

This occurred because `llama-cpp-2` was configured with `features = ["cuda"]` in `Cargo.toml`, forcing CUDA compilation even when the CUDA Toolkit wasn't installed on the CI runner.

## Root Cause

1. **Hard dependency on CUDA**: Line 44 in `src-tauri/Cargo.toml` had:
   ```toml
   llama-cpp-2 = { version = "0.1", features = ["cuda"] }
   ```

2. **GitHub Actions runners don't have CUDA Toolkit**: The Windows runners used by GitHub Actions don't come with NVIDIA CUDA Toolkit pre-installed.

3. **Build-time compilation**: `llama-cpp-2` uses `llama-cpp-sys-2` which compiles `llama.cpp` from source during the build, requiring CMake to find CUDA headers and libraries.

## Solution Implemented

### 1. Made CUDA Optional in Cargo.toml

**Changed** `src-tauri/Cargo.toml`:

```toml
# Before
llama-cpp-2 = { version = "0.1", features = ["cuda"] }

# After
llama-cpp-2 = { version = "0.1", default-features = false }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
# Optional CUDA support - requires CUDA Toolkit installed
cuda = ["llama-cpp-2/cuda"]
```

### 2. Disabled CUDA in CI Workflows

**Updated** `.github/workflows/tauri-windows-release.yml`:

```yaml
- name: Build Tauri (Windows)
  env:
    GGML_CUDA: OFF  # Disable CUDA compilation
    GGML_METAL: OFF  # Disable Metal (macOS GPU)
  run: npm run tauri:build
```

**Updated** `.github/workflows/release.yml`:

```yaml
- name: Build Tauri App
  uses: tauri-apps/tauri-action@v0
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    GGML_CUDA: OFF
    GGML_METAL: OFF
```

### 3. Fixed rustfmt Warning

Added `rustfmt` component to Rust toolchain setup:

```yaml
# tauri-windows-release.yml
- name: Setup Rust toolchain
  uses: actions-rs/toolchain@v1
  with:
    toolchain: stable
    override: true
    components: rustfmt  # Added this

# release.yml
- name: Setup Rust
  uses: dtolnay/rust-toolchain@stable
  with:
    components: rustfmt  # Added this
```

## How to Build with CUDA Support (Optional)

If you have an NVIDIA GPU and want GPU acceleration:

### Prerequisites

1. **Install CUDA Toolkit**: Download from [NVIDIA CUDA Downloads](https://developer.nvidia.com/cuda-downloads)
   - Recommended version: CUDA 12.6 or later
   - Ensure `nvcc` is in your PATH

2. **Verify installation**:
   ```bash
   nvcc --version
   ```

### Building with CUDA

#### Option 1: Cargo build with feature flag

```bash
cd src-tauri
cargo build --release --features cuda
```

#### Option 2: Tauri build with feature flag

```bash
npm run tauri:build -- -- --features cuda
```

#### Option 3: Set environment variable

```bash
# Windows (PowerShell)
$env:GGML_CUDA="ON"
npm run tauri:build

# Windows (CMD)
set GGML_CUDA=ON
npm run tauri:build

# Linux/macOS
GGML_CUDA=ON npm run tauri:build
```

## Build Configurations

### Default Build (CPU-only)
- **Use case**: Maximum compatibility, works on any hardware
- **Performance**: Slower inference but universally compatible
- **Command**: `npm run tauri:build`

### CUDA Build (GPU-accelerated)
- **Use case**: Users with NVIDIA GPUs and CUDA Toolkit installed
- **Performance**: 2-10x faster inference depending on model size
- **Command**: `npm run tauri:build -- -- --features cuda`
- **Requirements**: CUDA Toolkit 12.x, NVIDIA GPU with compute capability ≥ 5.0

## Testing the Fix

To verify the build works:

```bash
# Clean build to ensure no cached artifacts
cargo clean

# Build with CPU-only (should succeed)
cargo build --release

# Build with CUDA (only if you have CUDA Toolkit)
cargo build --release --features cuda
```

## Why This Approach?

### Benefits

1. **Maximum Compatibility**: Default builds work on any Windows system
2. **User Choice**: Users with NVIDIA GPUs can opt-in to GPU acceleration
3. **CI/CD Success**: GitHub Actions can build without CUDA Toolkit
4. **No Breaking Changes**: Existing users without CUDA continue to work
5. **Performance Options**: Power users get GPU acceleration when needed

### Alternative Approaches Considered

1. **Install CUDA in CI** ❌
   - Adds 5-10 minutes to build time
   - Increases runner costs
   - Not needed for CPU-only builds

2. **Separate CUDA/CPU builds** ❌
   - Doubles CI time
   - Confusing for users (which version to download?)
   - More maintenance overhead

3. **Dynamic runtime detection** ❌
   - Requires shipping both CUDA and CPU binaries
   - Large download sizes (500MB+)
   - Complex runtime switching logic

## Performance Impact

### CPU-only Build
- **Model loading**: ~5-10 seconds (depends on model size)
- **Inference**: ~50-200 tokens/second on modern CPUs
- **Memory**: Standard RAM usage

### CUDA Build (with NVIDIA GPU)
- **Model loading**: ~2-5 seconds (faster GPU memory transfer)
- **Inference**: ~200-1000 tokens/second (depends on GPU)
- **Memory**: Uses VRAM + RAM

## Troubleshooting

### Build still fails with CUDA errors

1. Ensure CUDA is truly disabled:
   ```bash
   echo $env:GGML_CUDA  # Should be "OFF" or empty
   ```

2. Clean Cargo cache:
   ```bash
   cargo clean
   rm -rf target/
   ```

3. Check CMake cache:
   ```bash
   # Delete CMake build directory
   rm -rf target/release/build/llama-cpp-sys-2-*/
   ```

### "CUDA Toolkit not found" despite having it installed

1. Verify CUDA installation:
   ```bash
   nvcc --version
   where nvcc  # Windows
   ```

2. Add CUDA to PATH:
   ```bash
   # Windows
   setx CUDA_PATH "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.6"

   # Linux/macOS
   export CUDA_PATH=/usr/local/cuda
   ```

3. Ensure CMake can find CUDA:
   ```bash
   cmake --find-package -DNAME=CUDA -DCOMPILER_ID=MSVC
   ```

## Related Issues

- [llama-cpp-2 CUDA build requirements](https://github.com/utilityai/llama-cpp-rs)
- [GitHub Actions CUDA setup](https://github.com/Jimver/cuda-toolkit)

## Future Improvements

1. **Auto-detect GPU at runtime**: Switch to GPU if available, fall back to CPU
2. **Vulkan support**: Cross-platform GPU acceleration (AMD, Intel, NVIDIA)
3. **Metal support**: GPU acceleration on macOS
4. **Prebuilt binaries**: Distribute both CPU and CUDA versions

## Summary

✅ **Fixed**: Windows CI/CD builds now succeed without CUDA Toolkit
✅ **Maintained**: GPU acceleration available via `--features cuda`
✅ **Improved**: Better documentation and user control over build options
✅ **Performance**: No regression for CPU users, GPU users can opt-in

This change makes BEAR AI more accessible while preserving advanced features for power users with NVIDIA GPUs.
