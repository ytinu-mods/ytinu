# ytinu Mod Manager

A Mod Manager for Unity Mods using the [BepInEx](https://github.com/BepInEx/BepInEx) Modding framework.

## How to use

Either watch this video or follow the instructions below.

[![Video with installation instructions](https://img.youtube.com/vi/2VjVBvPL5d8/0.jpg)](https://www.youtube.com/watch?v=2VjVBvPL5d8)

1. Download the the correct ytinu version for your operating system:
   - Windows: https://github.com/ytinu-mods/ytinu/releases/download/v0.1.0/ytinu.exe
   - Linux: https://github.com/ytinu-mods/ytinu/releases/download/v0.1.0/ytinu (**Note**: The Linux version is still untested and has some known issues. In particular, after ytinu installed the Mod Loader, you need to follow the instructions listed [here](https://docs.bepinex.dev/articles/advanced/steam_interop.html#2-set-up-permissions) if you're playing on Steam or do step 3 from [here](https://docs.bepinex.dev/articles/user_guide/installation/index.html?tabs=tabid-nix#installing-bepinex-1) if otherwise.)
   - MacOS: There currently aren't any pre-built executables for MacOS. Sorry! You can either compile ytinu yourself or install mods manually. 
2. Run the downloaded executable.

**Note:** For the basic setup ytinu requires you to have a working Chromium based browser like Google Chrome or the new Microsoft Edge installed.
On newer versions of Windows 10 the correct Microsoft Edge should be installed by default. For other systems see the section below.

### Usage without a Chromium based browser

If you don't have or want to install a Chromium browser you can also use ytinu with any other regular browser with ytinu acting as the web server.

To do this you need to create or modify the configuration file
at `%appdata%\ytinu\config.json` (i.e. `C:\Users\<user>\AppData\Roaming\ytinu\config.json`) on Windows
or at `$HOME/Library/Application Support/ytinu/config.json` on MacOS
or at `$HOME/.config/ytinu/config.json` on other Unix systems and add the following:

```jsonc
{
    "open_ui": "browser"
}
```

This will attempt to automatically open ytinu in your default browser on startup.

Alternatively you can just launch the ytinu web server part, fixate the port on which it is opened and
use your own methods to view the UI. To do this change your configuration to this:

```jsonc
{
    "open_ui": "none",
    "port": 1337        // Port on which ytinu should serve it's UI
}
```

If you launch ytinu with such a configuration you will also find an additional shutdown button in the menu bar to stop the ytinu server.
Remember that simply closing the browser window will do nothing to the program running in the background and ytinu currently only
refreshes mod metadata on startup.

## Compile ytinu

If you want to compile ytinu yourself, you need a decently up-to-date version of [Rust](https://rust-lang.org/) and [Node.js](https://nodejs.org/).

After that, it's fairly simple:

1. Build the frontend:
   1. Go into the `sevelte` directory
   2. Run `npm install` to install all dependencies
   3. Run `npm run build`
2. Build the executable with `cargo build --release`
