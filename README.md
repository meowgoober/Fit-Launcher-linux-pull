# Fit Launcher

Fit Launcher is a 🚀 **blazingly fast** 🚀 (sorry it's made in rust, I have to write that) game launcher designed specifically for cracked games from FitGirl Repack. Built with Rust, Tauri, and SolidJS, this launcher delivers the best performance and a sleek, modern design.

## Features
- **Lightning Fast**: Powered by Rust and Tauri for better speed and efficiency.
- **Cross-Platform**: Works on **Windows** and **Linux**.
- **Easy to Use**: Simple setup and easy to run.
- **Quick Download**: Uses **aria2c** for torrenting and DDL for optimal speed.
- **Wine/Proton Launch Support (Linux)**: When running on Linux, Fit Launcher detects installed Wine/Proton runners and asks which one to use when launching a game.

## Getting Started

### Images of the launcher

![Launcher Screenshot 1](images/1.png)
![Launcher Screenshot 2](images/2.png)
![Launcher Screenshot 3](images/3.png)
![Launcher Screenshot 4](images/4.png)
![Launcher Screenshot 5](images/5.png)
![Launcher Screenshot 6](images/6.png)
![Launcher Screenshot 7](images/7.png)
![Launcher Screenshot 8](images/8.png)

## Installation Guide

### Standard Mode

1. **Visit the Release Page**  
   Go to the [release page here](https://github.com/CarrotRub/Fit-Launcher/releases/latest).

2. **Download the correct package for your OS**
   - **Windows:** `.msi` or `.exe` installer (e.g. `Fit.Launcher_<version>_x64_en-US.msi`)
   - **Linux:** `.AppImage`, `.deb`, or `.rpm` package (depending on your distro)

3. **Install and run**
   - **Windows:** run the installer and launch Fit Launcher.
   - **Linux (AppImage):** make it executable and run it:
     ```bash
     chmod +x Fit.Launcher_*.AppImage
     ./Fit.Launcher_*.AppImage
     ```
   - **Linux (.deb/.rpm):** install with your package manager and launch Fit Launcher from your app menu or terminal.

### Linux runtime notes
- **Downloads:** `aria2c` must be installed and available in `PATH` (Linux builds do not bundle the Windows sidecar binaries).
- **Running FitGirl games/installers:** you need **Wine** and/or **Proton** installed. On Linux, Fit Launcher detects available runners and asks which one to use when launching a game.
- **Feature parity:** some Windows-specific features (e.g. automated installer UI automation / Windows Defender folder exclusion integration) may be unavailable or limited on Linux.

### Development Mode

If you want to run the launcher in development mode, follow these steps:

1. Ensure you have **Rust/Cargo**, **Node.js/npm**, and the required Tauri system dependencies for your OS installed.
2. Clone this repository.
3. Install frontend dependencies and start the app:

```bash
npm install
npm run tauri dev
```

### Building from Source on Windows

If you'd like to build Fit Launcher yourself on Windows, follow these steps:

1. Ensure you have a **C++ compiler**, **Rust/Cargo**, and **Node.js/npm** installed.
2. Clone this repository.
3. Download and extract [aria2c.exe](https://github.com/mokurin000/aria2c-windows-hide-console/releases/) to `src-tauri/binaries/aria2c-x86_64-pc-windows-msvc.exe`.
4. Build the Windows automation sidecar binary:
```pwsh
cd src-tauri; cargo build --release -p fit-launcher-automation 2>&1 | Select-Object -Last 5; Copy-Item -Force "target\release\FitLauncherService.exe" "binaries\FitLauncherService-x86_64-pc-windows-msvc.exe"
```
5. Build the app:

```bash
npm run tauri build
```

### Building on Linux

Linux builds use `src-tauri/tauri.linux.conf.json` (merged on Linux) which disables bundled Windows sidecar binaries. Install system dependencies, `aria2c`, and at least one Wine/Proton runner for launching games.

#### Option 1: Use the setup script (Debian/Ubuntu, Fedora, Arch)

```bash
chmod +x install_deps.sh
./install_deps.sh
npm run tauri dev
```

#### Option 2: Manual setup

1. Install [Tauri v2 Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux) for your distro.
2. Install **Rust/Cargo** and **Node.js/npm**.
3. Install runtime tools (recommended): `aria2c` and `wine` (and/or Steam/Proton).
4. Install dependencies and run/build:

```bash
npm install
npm run tauri dev     # development mode
npm run tauri build   # production build (.AppImage/.deb/.rpm depending on your environment)
```

## Create your own themes !
Now you can even customize the launcher to your liking, you can go check the tutorial on how to create a theme in doc/theme_creation.md

![Launcher Screenshot 1](images/themes/1.png)
![Launcher Screenshot 2](images/themes/2.png)
![Launcher Screenshot 3](images/themes/3.png)
![Launcher Screenshot 4](images/themes/4.png)
![Launcher Screenshot 5](images/themes/5.png)

*The last one is a special theme, come on Discord to get it*

Here are a few of the default themes !
You can then create as much as you want.
Good luck !!

## Join Our Community 

Got questions or want to hang out with our users? Join us on Discord! Big thanks to **Kürst** for moderating our Discord server, to **mokurin** for helping me tremendously with the many and most issues in the backend and to **SimplyStoned** for maintaining the old launcher while I was developing this one.

[Join the Discord](https://discord.gg/cXaBWdcUSF)

## Credits

- **CarrotRub** - Me, Developer of this app (Design now too) :3.
- **mokurin000** - Main Contributor.
- **Kürst** - Discord Moderator and Technical and Visual Co-Contributor
- **Vintage_Soldier** - First App Design (Old design).
- **SimplyStoned** - Maintenance of the Old Launcher and Contributor.

---

Enjoy your games with Fit Launcher!

## TODO List 

- **Improve Linux Feature Parity**: Continue improving Linux support (installer automation flows, parity with Windows-specific integrations).
- **Filtering by Genres and Sizes**: Implement filters to sort games by genres and file sizes.
- **Control CPU Usage**: Allow the user to limit the CPU usage of the setup.
