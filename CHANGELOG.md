# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- Download history search filter (filename/URL/protocol)
- Download history statistics summary (count, total size, avg speed, total duration)
- CSV export for download history
- GitHub Actions CI/CD pipeline (test + cross-platform release)
- Comprehensive test suite (348 tests)

### Changed
- Extracted `filterHistory` and `computeHistoryStats` as reusable utility functions

## [0.1.0] - 2026-05-29

### Added
- HTTP/HTTPS/FTP multi-threaded download with resume support
- BitTorrent .torrent and magnet link download (librqbit engine)
- ed2k protocol support (server/KAD/source exchange)
- HLS/DASH streaming media download
- Task management (CRUD, pause/resume, priority, concurrency control)
- SQLite persistence with crash recovery
- Browser extension (Chrome MV3) with video sniffing
- WebUI with responsive design and WebSocket real-time updates
- JSON-RPC 2.0 API (aria2 compatible)
- RSS auto-download with filter rules
- Auto-extract (ZIP/TAR/TAR.GZ/TAR.BZ2/7Z) with password management
- Batch import (text file, clipboard, wildcard, regex)
- Plugin system (WASM sandbox)
- Virtual scrolling task list
- Speed chart (Recharts)
- Dark/light/system theme support
- System tray integration
- Clipboard download link detection
- Drag & drop .torrent files
- Keyboard shortcuts (Ctrl+N, Delete, Space, Ctrl+A, Escape)
- Task search and status filtering
- Download history with clear functionality
- Settings center (general/download/connection/BT/notification/advanced)
