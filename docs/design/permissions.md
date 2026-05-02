# macOS Permissions

## Required Permissions

Zecho needs two macOS permissions to function:

### 1. Microphone Access
- **Why**: To record voice for transcription
- **Where**: System Settings > Privacy & Security > Microphone
- **Behavior**: macOS auto-prompts on first recording attempt IF the app has `NSMicrophoneUsageDescription` in Info.plist
- **Important**: Permission is tied to the app's code signature. Reinstalling a differently-signed binary resets the permission.

### 2. Input Monitoring (Accessibility)
- **Why**: To capture the FN key globally for push-to-talk
- **Where**: System Settings > Privacy & Security > Input Monitoring
- **Behavior**: `AXIsProcessTrustedWithOptions` with `kAXTrustedCheckOptionPrompt` triggers the system prompt
- **Important**: Same code-signature issue as microphone. Each new build may need re-granting.
- **Fallback**: Option+Space works as a global shortcut without Input Monitoring permission

## Development Notes

- During development, `cargo run` runs through Terminal which has its own permissions
- Release builds (.app bundles) have a different signature and need their own permissions
- **Never kill and reinstall the app repeatedly during user testing** — it resets permissions each time
- `tccutil reset Microphone com.dzearing.zecho` can be used to force re-prompt
