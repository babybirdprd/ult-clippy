# RustBoard

**RustBoard** is a high-performance, persistent clipboard manager built with Tauri v2 and React. It is designed to replace the native Windows clipboard manager (`Win+V`) with a faster, customizable, and "always-on" alternative.

## Features

- **⚡ Lightning Fast:** Built with Rust and Tauri for minimal resource usage.
- **♾️ Unlimited History:** Persists your clipboard history to a local SQLite database (no RAM limits).
- **📌 Pinning:** Keep important clips at the top of your list forever.
- **🔍 Search & Filter:** Instantly find clips by content, name, or category.
- **🏷️ Categorization:** Organize clips with custom categories (e.g., "Work", "Code", "Personal").
- **🖼️ Image Support:** Stores and previews images alongside text.
- **⌨️ Keyboard Centric:** Toggle with `Win+V`, navigate, and paste without lifting your hands.
- **🚀 Auto-Start:** Runs silently in the background on system boot.

## Prerequisites

Before setting up RustBoard, ensure you have the following installed:

- **Operating System:** Windows 10 or Windows 11.
- **Node.js:** (LTS recommended)
- **Rust:** (Latest stable) -> [Install Rust](https://www.rust-lang.org/tools/install)
- **pnpm:** This project exclusively uses pnpm.
  ```bash
  npm install -g pnpm
  ```
- **Build Tools:** Microsoft Visual Studio C++ Build Tools (required for Rust development on Windows).

### Important: Disable Native Windows Clipboard
RustBoard uses the global hotkey `Win+V`. To prevent conflicts, you **must** disable the native Windows clipboard history:
1. Go to **Settings** > **System** > **Clipboard**.
2. Toggle **Clipboard history** to **Off**.

## Installation (Development)

RustBoard is open-source. Follow these steps to build and run it from source.

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/rustboard.git
   cd rustboard
   ```

2. **Install dependencies:**
   ```bash
   pnpm install
   ```

3. **Run in Development Mode:**
   This will start the React frontend and the Tauri backend.
   ```bash
   pnpm tauri dev
   ```
   *The first run might take a moment to compile Rust dependencies.*

4. **Build for Production:**
   To create an optimized `.exe` installer:
   ```bash
   pnpm tauri build
   ```
   The installer will be located in `src-tauri/target/release/bundle/nsis/`.

## Usage

1. **Toggle the Window:** Press `Win+V` (or click the tray icon) to open RustBoard.
2. **Paste a Clip:** Click on any item in the list. The window will hide, and the content will be pasted into your previously active application.
3. **Pin/Unpin:** Click the pin icon to save a clip permanently.
4. **Edit:** Click the pencil icon to rename a clip or assign a category.
5. **Search:** Just start typing to filter your history.

## Troubleshooting

### `Win+V` opens the Windows Clipboard instead of RustBoard
Ensure you have disabled the native Windows Clipboard History in System Settings. If the issue persists, restart RustBoard.

### App crashes on startup
This is usually due to missing permissions or dependencies.
- Run `pnpm tauri dev` to see the error log in your terminal.
- Ensure you have the C++ Build Tools installed.

### Focus not returning to previous app
RustBoard attempts to hide itself and return focus before simulating `Ctrl+V`. If pasting fails, try increasing the delay in `src-tauri/src/lib.rs` (look for `thread::sleep`).

## Tech Stack

- **Frontend:** React 19, Vite, Tailwind CSS, Lucide React
- **Backend:** Tauri v2 (Rust)
- **Database:** SQLite (via `tauri-plugin-sql`)
- **Clipboard Monitoring:** `tauri-plugin-clipboard` (CrossCopy)
- **Input Simulation:** `enigo`
