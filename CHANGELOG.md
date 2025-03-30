# Changelog

## [v0.3.1](https://github.com/nixcodeit/nixcode-ai/compare/v0.3.0...HEAD)

### New Features
- Added new git log tool allowing AI to view commit history between references ([e1c8e8d](https://github.com/nixcodeit/nixcode-ai/commit/e1c8e8d)) [@nixuuu](https://github.com/nixuuu)

### Bug Fixes
- Temporarily removed commit tool due to issues ([c8b9e98](https://github.com/nixcodeit/nixcode-ai/commit/c8b9e98)) [@nixuuu](https://github.com/nixuuu)

### Other Changes
- Reversed git commit tool changes and finalized git log tool implementation ([0023acf](https://github.com/nixcodeit/nixcode-ai/commit/0023acf)) [@nixuuu](https://github.com/nixuuu)

## [v0.3.0](https://github.com/nixcodeit/nixcode-ai/compare/v0.2.0...v0.3.0)

### New Features
- Tool execution parameters now visible in chat interface ([0cf6c6f](https://github.com/nixcodeit/nixcode-ai/commit/0cf6c6f)) [@nixuuu](https://github.com/nixuuu)
- Added new filesystem tools for enhanced file operations ([91162bc](https://github.com/nixcodeit/nixcode-ai/commit/91162bc)) [@nixuuu](https://github.com/nixuuu)
- Added git stash tools for stash management ([cd11a86](https://github.com/nixcodeit/nixcode-ai/commit/cd11a86)) [@nixuuu](https://github.com/nixuuu)
- Updated default Anthropic model to claude-3-7-sonnet-20250219 ([5a1c82d](https://github.com/nixcodeit/nixcode-ai/commit/5a1c82d)) [@nixuuu](https://github.com/nixuuu)
- Added project status badges to README ([5fb29ee](https://github.com/nixcodeit/nixcode-ai/commit/5fb29ee)) [@nixuuu](https://github.com/nixuuu)
- Added file name prefix to project analysis and updated documentation ([82ab343](https://github.com/nixcodeit/nixcode-ai/commit/82ab343)) [@nixuuu](https://github.com/nixuuu)

### Bug Fixes
- Updated system prompt ([5fb3567](https://github.com/nixcodeit/nixcode-ai/commit/5fb3567)) [@nixuuu](https://github.com/nixuuu)
- Updated system prompt to prevent automatic git commits ([6ee198e](https://github.com/nixcodeit/nixcode-ai/commit/6ee198e)) [@nixuuu](https://github.com/nixuuu)
- Fixed usage of tool description in macro ([103a991](https://github.com/nixcodeit/nixcode-ai/commit/103a991)) [@nixuuu](https://github.com/nixuuu)
- Updated glob test for better compatibility ([87c0946](https://github.com/nixcodeit/nixcode-ai/commit/87c0946)) [@nixuuu](https://github.com/nixuuu)
- Disabled automatic git commits ([8e2ea1b](https://github.com/nixcodeit/nixcode-ai/commit/8e2ea1b)) [@nixuuu](https://github.com/nixuuu)

### Refactoring
- Major reorganization of tools into separate files and modules ([166071a](https://github.com/nixcodeit/nixcode-ai/commit/166071a)) [@nixuuu](https://github.com/nixuuu)
  - Moved each tool into its own file
  - Grouped related tools into subdirectories (git, fs, search, etc.)
  - Organized common utilities into appropriate modules
  - Updated imports and module structure
  - Added proper tests organization
  - Maintained all existing functionality with better code organization

### Other Changes
- Updated Rust CI workflow ([ac1d936](https://github.com/nixcodeit/nixcode-ai/commit/ac1d936)) [@nixuuu](https://github.com/nixuuu)
- Bumped CLI version ([914fe30](https://github.com/nixcodeit/nixcode-ai/commit/914fe30)) [@nixuuu](https://github.com/nixuuu)

## [v0.2.0](https://github.com/nixcodeit/nixcode-ai/compare/v0.1.0...v0.2.0)

### New Features
- Added display of current version in UI ([55064e7](https://github.com/nixcodeit/nixcode-ai/commit/55064e7)) [@nixuuu](https://github.com/nixuuu)
- Added replace_content tool for file content replacement ([79e3544](https://github.com/nixcodeit/nixcode-ai/commit/79e3544)) [@nixuuu](https://github.com/nixuuu)
- Added new tool for searching files based on content ([070d042](https://github.com/nixcodeit/nixcode-ai/commit/070d042)) [@nixuuu](https://github.com/nixuuu)

### Bug Fixes
- Applied cargo fix changes for better code quality ([0084b0b](https://github.com/nixcodeit/nixcode-ai/commit/0084b0b)) [@nixuuu](https://github.com/nixuuu)

### Other Changes
- Bumped CLI version ([dc766a8](https://github.com/nixcodeit/nixcode-ai/commit/dc766a8)) [@nixuuu](https://github.com/nixuuu)

## [v0.1.0](https://github.com/nixcodeit/nixcode-ai/commits/v0.1.0)

### Initial Release Features
- Core TUI application with vim-inspired input modes
- Integration with Anthropic Claude AI
- File system tools (Create, Read, Update, Delete)
- Git integration with status, diff, and basic operations
- Glob pattern file searching
- Project analysis tool for better understanding by LLM
- Command popup UI
- Horizontal scrolling in user input field
- System prompt customization
- Anthropic response caching
- Clear chat and retry functionality

### Notable Commits
- Added git diff tool functionality ([4cef032](https://github.com/nixcodeit/nixcode-ai/commit/4cef032)) [@nixuuu](https://github.com/nixuuu)
- Added git integration ([920de52](https://github.com/nixcodeit/nixcode-ai/commit/920de52)) [@nixuuu](https://github.com/nixuuu)
- Added resolving git directory in project ([33741f5](https://github.com/nixcodeit/nixcode-ai/commit/33741f5)) [@nixuuu](https://github.com/nixuuu)
- Command popup UI improvements ([10be2ff](https://github.com/nixcodeit/nixcode-ai/commit/10be2ff)) [@nixuuu](https://github.com/nixuuu)
- Updated system prompt for better user experience ([3aa6c06](https://github.com/nixcodeit/nixcode-ai/commit/3aa6c06)) [@nixuuu](https://github.com/nixuuu)
- Added configuration from external file ([85c2119](https://github.com/nixcodeit/nixcode-ai/commit/85c2119)) [@nixuuu](https://github.com/nixuuu)
- Changed sync tools into async ([72209f0](https://github.com/nixcodeit/nixcode-ai/commit/72209f0)) [@nixuuu](https://github.com/nixuuu)
- Added project analysis tool ([ba11dd0](https://github.com/nixcodeit/nixcode-ai/commit/ba11dd0)) [@nixuuu](https://github.com/nixuuu)
- Added thinking by default ([0f47d3f](https://github.com/nixcodeit/nixcode-ai/commit/0f47d3f)) [@nixuuu](https://github.com/nixuuu)
- Added function calling mechanism ([0f65f47](https://github.com/nixcodeit/nixcode-ai/commit/0f65f47)) [@nixuuu](https://github.com/nixuuu)

### Bug Fixes
- Added openssl-sys with vendored feature ([f380f97](https://github.com/nixcodeit/nixcode-ai/commit/f380f97)) [@nixuuu](https://github.com/nixuuu)
- Updated glob tests ([561cc7b](https://github.com/nixcodeit/nixcode-ai/commit/561cc7b)) [@nixuuu](https://github.com/nixuuu)
- Fixed scrolling via buttons not working ([cbd2233](https://github.com/nixcodeit/nixcode-ai/commit/cbd2233)) [@nixuuu](https://github.com/nixuuu)
- Removed quotes from glob response ([1e2264b](https://github.com/nixcodeit/nixcode-ai/commit/1e2264b)) [@nixuuu](https://github.com/nixuuu)
- Fixed calculating content height for viewport ([baa5030](https://github.com/nixcodeit/nixcode-ai/commit/baa5030)) [@nixuuu](https://github.com/nixuuu)