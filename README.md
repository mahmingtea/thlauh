# Thlauh (Tauri + React + TypeScript)

He project hi Tauri, React, leh TypeScript hmanga siam desktop application a ni. A hnuaiah hian he project download dan, run dan, leh build dan tarlan a ni.

## Thil pawimawh hmasate (Prerequisites)

Project i khawih (run/build) hmain a hnuaia mite hi i khawl (computer)-ah dah (install) hmasak a ngai ang:

1. **Rust** - Tauri development atan Rust a ngai. [Install Rust](https://www.rust-lang.org/tools/install)
2. **Package Manager (Bun / Node.js / npm / pnpm / yarn)** - Bun hmanga he project hi siam a nih avangin `bun.lock` a awm. Mahse Node.js/npm, pnpm, emaw yarn pawh a hman theih tho.
   - [Install Bun](https://bun.sh/)
   - [Install Node.js (npm telin)](https://nodejs.org/)
3. **Build Tools** - Tauri build nan OS a zirin build tools a ngai bawk:
   - **Mac user** nih chuan Xcode Command Line Tools i neih a ngai: `xcode-select --install`
   - **Windows user** nih chuan C++ build tools (Visual Studio hmangin) install a ngai.
   - **Linux (Debian/Ubuntu) user** nih chuan a hnuaia package ho hi apt hmanga install tur a ni:
     ```bash
     sudo apt update
     sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libappindicator3-dev librsvg2-dev patchelf build-essential curl wget file
     ```

---

## 1. Project Download Dan (Download)

Git hmangin he repository hi i computer-ah clone (download) rawh:

```bash
git clone https://github.com/mahmingtea/thlauh.git
cd thlauh
```

---

## 2. Dependency Install Dan (Install Dependencies)

I package manager hman zawk zirin dependency te hi install rawh.
*(Note: Bun aia dang i hman dawn chuan `bun.lock` file hi delete a tha ang).*

*   **Bun:**
    ```bash
    bun install
    ```
*   **npm:**
    ```bash
    npm install
    ```
*   **pnpm:**
    ```bash
    pnpm install
    ```
*   **yarn:**
    ```bash
    yarn install
    ```

---

## 3. Project Run Dan / Dev Mode (Start/Development)

Development environment tlan tir a, app hawn nan a hnuaia mi hi hmang rawh:

*   **Bun:**
    ```bash
    bun tauri dev
    ```
*   **npm:**
    ```bash
    npm run tauri dev
    ```
*   **pnpm:**
    ```bash
    pnpm tauri dev
    ```
*   **yarn:**
    ```bash
    yarn tauri dev
    ```

Hei hian i code siam danglam apiang hot-reload hmangin a thlak danglam nghal ang.

---

## 4. App Build Dan (Build for Production)

Platform hrang hrang atana installer siam nan a hnuaia command-te hi hman tur a ni:

### Bun hmangin:
*   **Mac (Apple Silicon) atan:** `bun run build:mac`
*   **Windows atan:** `bun run build:win`
*   **Linux atan:** `bun run build:linux`

### npm hmangin:
*   **Mac (Apple Silicon) atan:** `npm run build:mac`
*   **Windows atan:** `npm run build:win`
*   **Linux atan:** `npm run build:linux`

### pnpm hmangin:
*   **Mac (Apple Silicon) atan:** `pnpm build:mac`
*   **Windows atan:** `pnpm build:win`
*   **Linux atan:** `pnpm build:linux`

### yarn hmangin:
*   **Mac (Apple Silicon) atan:** `yarn build:mac`
*   **Windows atan:** `yarn build:win`
*   **Linux atan:** `yarn build:linux`

---

## IDE Setup dan tur

- **Editor:** [VS Code](https://code.visualstudio.com/)
- **Extension mamawh te:**
  - [Tauri Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
  - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)


