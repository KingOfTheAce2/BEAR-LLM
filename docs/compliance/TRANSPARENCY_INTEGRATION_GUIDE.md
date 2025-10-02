# Transparency Notice Integration Guide
## Developer Implementation Instructions

**Purpose:** Integrate AI transparency notice into BEAR AI LLM for EU AI Act compliance
**Target:** Development team implementing Articles 13, 52, and 53 requirements
**Status:** Documentation complete, UI component ready, backend integration pending

---

## Quick Start Checklist

- [x] Create AI Transparency Notice document
- [x] Create model cards for all LLMs (TinyLlama, Phi-2, Mistral-7B)
- [x] Create TransparencyNotice React component
- [ ] Add Rust backend commands
- [ ] Integrate component into App.tsx
- [ ] Add menu items for transparency access
- [ ] Test first-launch flow
- [ ] Deploy and monitor user acknowledgment

---

## 1. Backend Implementation (Rust)

### 1.1 Required Tauri Commands

Add these commands to `src-tauri/src/commands.rs`:

```rust
use tauri::Manager;
use std::fs;
use std::path::PathBuf;

/// Get the currently active LLM model name
#[tauri::command]
fn get_current_model() -> String {
    // Return current model from state or config
    // Example: "Mistral-7B-Instruct-v0.2"
    // TODO: Integrate with existing model management
    String::from("Unknown Model")
}

/// Mark transparency notice as acknowledged by user
#[tauri::command]
fn set_transparency_acknowledged() -> Result<(), String> {
    let app_data_dir = get_app_data_dir()?;
    let ack_file = app_data_dir.join("transparency_acknowledged");

    fs::write(&ack_file, "acknowledged")
        .map_err(|e| format!("Failed to save acknowledgment: {}", e))?;

    Ok(())
}

/// Check if transparency notice has been acknowledged
#[tauri::command]
fn check_transparency_acknowledged() -> Result<bool, String> {
    let app_data_dir = get_app_data_dir()?;
    let ack_file = app_data_dir.join("transparency_acknowledged");

    Ok(ack_file.exists())
}

/// Open the model cards folder in system file explorer
#[tauri::command]
fn open_model_cards_folder() -> Result<(), String> {
    use std::process::Command;

    // Get path to model cards (in resources or docs)
    let model_cards_path = PathBuf::from("docs/model_cards");

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(model_cards_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(model_cards_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(model_cards_path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(())
}

/// Helper: Get app data directory
fn get_app_data_dir() -> Result<PathBuf, String> {
    tauri::api::path::app_data_dir(&tauri::Config::default())
        .ok_or_else(|| "Failed to get app data directory".to_string())
}
```

### 1.2 Register Commands

Update `src-tauri/src/main.rs` to register new commands:

```rust
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...
            get_current_model,
            set_transparency_acknowledged,
            check_transparency_acknowledged,
            open_model_cards_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 1.3 Integration with Existing Setup Flow

Modify `check_first_run` or `get_setup_status` command to include transparency acknowledgment:

```rust
#[derive(serde::Serialize)]
struct SetupStatus {
    setup_complete: bool,
    transparency_acknowledged: bool,
    // ... other fields ...
}

#[tauri::command]
fn get_setup_status() -> Result<SetupStatus, String> {
    let transparency_acked = check_transparency_acknowledged()?;

    Ok(SetupStatus {
        setup_complete: /* existing check */,
        transparency_acknowledged: transparency_acked,
        // ... other fields ...
    })
}
```

---

## 2. Frontend Implementation (React/TypeScript)

### 2.1 Import TransparencyNotice Component

In `src/App.tsx`, add import:

```typescript
import TransparencyNotice from './components/TransparencyNotice';
```

### 2.2 Add State Management

Add state for transparency notice visibility:

```typescript
const [showTransparency, setShowTransparency] = useState(false);
const [transparencySource, setTransparencySource] = useState<'firstLaunch' | 'menu'>('firstLaunch');
```

### 2.3 Check Acknowledgment on Startup

Modify the existing `checkFirstRun` effect:

```typescript
useEffect(() => {
    const checkFirstRun = async () => {
        try {
            const timeout = new Promise((_, reject) =>
                setTimeout(() => reject(new Error('Setup check timeout')), 5000)
            );

            const checkPromise = Promise.all([
                invoke<boolean>('check_first_run'),
                invoke<any>('get_setup_status')
            ]);

            const [isFirstRun, setupStatus] = await Promise.race([checkPromise, timeout]) as [boolean, any];

            if (isFirstRun || !setupStatus.setup_complete) {
                setShowSetup(true);
            } else if (!setupStatus.transparency_acknowledged) {
                // Show transparency notice if not acknowledged
                setShowTransparency(true);
                setTransparencySource('firstLaunch');
                setSetupComplete(true);
            } else {
                setSetupComplete(true);
            }
        } catch (err) {
            logger.error('Error checking setup status', err);
            setSetupComplete(true);
        } finally {
            setIsInitializing(false);
        }
    };

    checkFirstRun();
}, []);
```

### 2.4 Add Transparency Notice to JSX

In the return statement of `App.tsx`:

```typescript
return (
    <>
        {showSetup && !setupComplete && (
            <SetupWizard
                onComplete={() => {
                    setShowSetup(false);
                    setSetupComplete(true);
                    // After setup, check if transparency needs to be shown
                    invoke<any>('get_setup_status').then(status => {
                        if (!status.transparency_acknowledged) {
                            setShowTransparency(true);
                            setTransparencySource('firstLaunch');
                        }
                    });
                }}
                theme={theme}
                onThemeToggle={toggleTheme}
            />
        )}
        {showTransparency && (
            <TransparencyNotice
                onClose={() => setShowTransparency(false)}
                theme={theme}
                triggerSource={transparencySource}
            />
        )}
        <UpdateNotification />
        {/* Rest of app ... */}
    </>
);
```

### 2.5 Add Menu Item for Transparency Access

Add a function to open transparency notice from menu:

```typescript
const handleShowTransparency = () => {
    setShowTransparency(true);
    setTransparencySource('menu');
};
```

**Option 1: Add to Sidebar Menu**
```typescript
<div className="p-4 border-t border-[var(--border-primary)] space-y-2">
    <ModelSelector />

    {/* Add Transparency Menu Item */}
    <button
        onClick={handleShowTransparency}
        className="w-full flex items-center justify-center gap-2 px-3 py-2 hover:bg-[var(--hover-bg)] rounded-lg transition-all"
    >
        <Scale className="w-4 h-4" />
        <span className="text-sm">About AI System</span>
    </button>

    <button
        onClick={toggleTheme}
        className="w-full flex items-center justify-center gap-2 px-3 py-2 hover:bg-[var(--hover-bg)] rounded-lg transition-all"
    >
        {/* Existing theme toggle ... */}
    </button>
</div>
```

**Option 2: Create Dedicated Menu Bar**
(If you add a proper menu bar in the future)

```typescript
<Menu>
    <MenuItem onClick={handleShowTransparency}>About AI System</MenuItem>
    <MenuItem onClick={() => invoke('open_model_cards_folder')}>View Model Cards</MenuItem>
    <MenuItem onClick={() => invoke('open_transparency_notice')}>Transparency Notice</MenuItem>
</Menu>
```

---

## 3. Testing Checklist

### 3.1 Unit Tests

```typescript
// src/components/TransparencyNotice.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import TransparencyNotice from './TransparencyNotice';

describe('TransparencyNotice', () => {
    test('renders risk classification warning', () => {
        render(<TransparencyNotice onClose={() => {}} theme="dark" />);
        expect(screen.getByText(/high-risk ai system/i)).toBeInTheDocument();
    });

    test('requires acknowledgment on first launch', () => {
        render(
            <TransparencyNotice
                onClose={() => {}}
                theme="dark"
                triggerSource="firstLaunch"
            />
        );
        const acceptButton = screen.getByText(/i understand and accept/i);
        expect(acceptButton).toBeDisabled();
    });

    test('enables accept button when checkbox checked', () => {
        render(
            <TransparencyNotice
                onClose={() => {}}
                theme="dark"
                triggerSource="firstLaunch"
            />
        );
        const checkbox = screen.getByRole('checkbox');
        fireEvent.click(checkbox);

        const acceptButton = screen.getByText(/i understand and accept/i);
        expect(acceptButton).not.toBeDisabled();
    });

    test('shows close button when opened from menu', () => {
        render(
            <TransparencyNotice
                onClose={() => {}}
                theme="dark"
                triggerSource="menu"
            />
        );
        expect(screen.getByText(/close/i)).toBeInTheDocument();
    });
});
```

### 3.2 Integration Tests

**Test Case 1: First Launch Flow**
1. ✅ Fresh install (no acknowledgment file)
2. ✅ Complete setup wizard
3. ✅ Transparency notice appears
4. ✅ Accept button disabled initially
5. ✅ Check checkbox enables accept button
6. ✅ Click accept saves acknowledgment
7. ✅ Main app loads

**Test Case 2: Returning User**
1. ✅ Acknowledgment file exists
2. ✅ App loads directly (no transparency notice)
3. ✅ Transparency accessible via menu

**Test Case 3: Menu Access**
1. ✅ Click "About AI System" menu item
2. ✅ Transparency notice opens
3. ✅ Shows "Close" button (not "Accept")
4. ✅ No acknowledgment required
5. ✅ Close button works

**Test Case 4: Model Information**
1. ✅ Current model displayed correctly
2. ✅ Model-specific limitations shown
3. ✅ Accuracy metrics displayed
4. ✅ Expandable sections work

**Test Case 5: Model Cards Access**
1. ✅ Click "View Model Cards" button
2. ✅ File explorer opens to model_cards folder
3. ✅ TOML files accessible

### 3.3 Accessibility Tests

- [ ] Keyboard navigation works (Tab, Enter, Esc)
- [ ] Screen reader announces content correctly
- [ ] Focus trap within modal (can't tab out)
- [ ] Esc key closes modal
- [ ] Color contrast meets WCAG 2.1 AA
- [ ] ARIA labels present and correct

### 3.4 Browser/Platform Tests

- [ ] Windows 10/11 (Edge, Chrome)
- [ ] Light theme display
- [ ] Dark theme display
- [ ] Various screen sizes (1920x1080, 1366x768, 2560x1440)

---

## 4. Deployment Steps

### 4.1 Pre-Deployment

1. ✅ Verify all documentation files in place:
   - `docs/AI_TRANSPARENCY_NOTICE.md`
   - `docs/model_cards/*.toml`
   - `docs/compliance/*.md`

2. ✅ Verify UI component created:
   - `src/components/TransparencyNotice.tsx`

3. ⏳ Implement and test backend commands:
   - `get_current_model`
   - `set_transparency_acknowledged`
   - `check_transparency_acknowledged`
   - `open_model_cards_folder`

4. ⏳ Integrate component into App.tsx

5. ⏳ Test all user flows

6. ⏳ Accessibility audit

### 4.2 Deployment

1. Update version in `package.json` and `Cargo.toml`
2. Update changelog with transparency compliance features
3. Build release (`npm run tauri build`)
4. Test installer on clean machine
5. Verify first-launch experience
6. Deploy to GitHub Releases
7. Update documentation website

### 4.3 Post-Deployment

1. Monitor user feedback on transparency notice
2. Track acknowledgment rates (analytics if available)
3. Collect questions/confusion points
4. Update documentation based on feedback
5. Consider adding FAQ section

---

## 5. Maintenance and Updates

### 5.1 When to Update Transparency Documentation

**Update AI Transparency Notice when:**
- Adding or removing supported LLMs
- Significant changes to PII detection capabilities
- Changes to privacy or data handling practices
- New limitations or risks identified
- EU AI Act guidance updates

**Update Model Cards when:**
- Adding new models to BEAR AI
- New benchmark results available
- Model versions updated
- Bias or fairness testing conducted
- Environmental impact data refined

**Update UI Component when:**
- New regulatory requirements emerge
- User feedback suggests clarity improvements
- Accessibility issues identified
- New sections needed in transparency notice

### 5.2 Version Control

**Transparency Notice Versioning:**
```markdown
**Last Updated:** [Date]
**Version:** [Semantic version]
**Effective Date:** [Date]
```

**Model Card Versioning:**
```toml
[metadata]
card_version = "1.0.0"
card_date = "2025-10-02"
last_updated = "2025-10-02"
```

**Component Versioning:**
- Track changes in git commit history
- Document breaking changes in changelog
- Use semantic versioning for major UI changes

### 5.3 Compliance Monitoring

**Quarterly Reviews:**
- Check for EU AI Act implementation updates
- Review user feedback on transparency materials
- Audit acknowledgment rates
- Assess need for documentation updates

**Annual Reviews:**
- Comprehensive compliance audit
- Legal review of compliance statements (recommended)
- Update performance benchmarks
- Review and update bias/fairness disclosures

---

## 6. Troubleshooting

### 6.1 Common Issues

**Issue: Transparency notice not appearing on first launch**
- Check `check_transparency_acknowledged` returns `false`
- Verify `get_setup_status` includes `transparency_acknowledged` field
- Check React state management in App.tsx

**Issue: Accept button stays disabled**
- Verify checkbox state management
- Check browser console for JavaScript errors
- Ensure `accepted` state updates on checkbox change

**Issue: Model cards folder doesn't open**
- Verify path to `docs/model_cards` is correct
- Check platform-specific commands (explorer/open/xdg-open)
- Ensure folder is bundled with application

**Issue: Current model shows as "Unknown"**
- Verify `get_current_model` command implemented
- Check integration with existing model management system
- Ensure state synchronization between Rust and React

### 6.2 Debug Commands

```bash
# Check if acknowledgment file exists
# Windows
dir "%LOCALAPPDATA%\BEAR AI LLM\transparency_acknowledged"

# Check transparency notice file
type "docs\AI_TRANSPARENCY_NOTICE.md" | more

# Validate TOML syntax
npx toml-cli validate docs/model_cards/mistral_card.toml

# Test component in isolation
npm test -- TransparencyNotice.test.tsx
```

---

## 7. Resources and References

### 7.1 Created Files

| File | Purpose | Lines | Status |
|------|---------|-------|--------|
| `docs/AI_TRANSPARENCY_NOTICE.md` | Main transparency notice | ~450 | ✅ |
| `docs/model_cards/tinyllama_card.toml` | TinyLlama model card | 371 | ✅ |
| `docs/model_cards/phi2_card.toml` | Phi-2 model card | 418 | ✅ |
| `docs/model_cards/mistral_card.toml` | Mistral-7B model card | 512 | ✅ |
| `src/components/TransparencyNotice.tsx` | UI component | 521 | ✅ |
| `docs/compliance/AI_ACT_COMPLIANCE_REPORT.md` | Compliance report | ~1000 | ✅ |
| `docs/compliance/TRANSPARENCY_INTEGRATION_GUIDE.md` | This guide | ~600 | ✅ |

### 7.2 External References

**EU AI Act:**
- [Regulation (EU) 2024/1689](https://eur-lex.europa.eu/eli/reg/2024/1689/oj)
- Article 13: Transparency and provision of information to deployers
- Article 52: Transparency obligations for certain AI systems
- Article 53: Technical documentation

**GDPR:**
- [Regulation (EU) 2016/679](https://eur-lex.europa.eu/eli/reg/2016/679/oj)
- Article 13: Information to be provided

**Technical Standards:**
- [ISO/IEC 23894:2023](https://www.iso.org/standard/77304.html) - AI Risk Management
- [ISO/IEC 42001:2023](https://www.iso.org/standard/81230.html) - AI Management Systems

**Accessibility:**
- [WCAG 2.1 Level AA](https://www.w3.org/WAI/WCAG21/quickref/)
- [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)

### 7.3 Support Contacts

**Development Questions:** support@bear-ai.com
**Compliance Questions:** compliance@bear-ai.com
**Security Issues:** security@bear-ai.com
**GitHub Issues:** https://github.com/KingOfTheAce2/BEAR-LLM/issues

---

## 8. Quick Reference: Key Code Snippets

### 8.1 Open Transparency Notice from Anywhere

```typescript
import { useState } from 'react';
import TransparencyNotice from './components/TransparencyNotice';

// In your component
const [showTransparency, setShowTransparency] = useState(false);

// Trigger from button/menu
<button onClick={() => setShowTransparency(true)}>
    About AI System
</button>

// Render modal
{showTransparency && (
    <TransparencyNotice
        onClose={() => setShowTransparency(false)}
        theme={theme}
        triggerSource="menu"
    />
)}
```

### 8.2 Check Acknowledgment Status

```typescript
import { invoke } from '@tauri-apps/api/core';

const checkAcknowledgment = async () => {
    try {
        const acknowledged = await invoke<boolean>('check_transparency_acknowledged');
        if (!acknowledged) {
            // Show transparency notice
            setShowTransparency(true);
        }
    } catch (error) {
        console.error('Failed to check acknowledgment:', error);
    }
};
```

### 8.3 Get Current Model Info

```typescript
import { invoke } from '@tauri-apps/api/core';

const getCurrentModel = async () => {
    try {
        const modelName = await invoke<string>('get_current_model');
        console.log('Current model:', modelName);
        return modelName;
    } catch (error) {
        console.error('Failed to get current model:', error);
        return 'Unknown';
    }
};
```

---

## 9. Conclusion

This integration guide provides everything needed to implement EU AI Act transparency compliance in BEAR AI LLM. Follow the steps sequentially:

1. ✅ Documentation created (transparency notice, model cards)
2. ✅ UI component ready (`TransparencyNotice.tsx`)
3. ⏳ **Next:** Implement backend Rust commands
4. ⏳ **Next:** Integrate component into App.tsx
5. ⏳ **Next:** Test thoroughly
6. ⏳ **Next:** Deploy and monitor

**Estimated Implementation Time:** 4-8 hours (backend + integration + testing)

**Priority:** HIGH (Required for EU AI Act compliance)

**Deadline:** Before next major release (recommend v1.0.25)

---

*For questions or clarifications, contact the compliance team at compliance@bear-ai.com*
