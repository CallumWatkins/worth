# Worth Brand Assets

This folder contains the source artwork and exported icon assets for Worth.

## Brand asset notice
Forks are welcome, but the Worth branding is separate from the source code license.

The Worth™ name, logo, app icons, wordmarks, combination marks, source artwork, exported images, and copies of those assets elsewhere in this repo are brand assets and source identifiers of Callum Watkins. This includes assets under this folder and copied/exported files such as `src-tauri/icons/` and `app/assets/`.

These brand assets are **not** licensed under `AGPL-3.0-only`. Copyright © 2026 Callum Watkins. All rights reserved, except for the limited permissions below.

You may use the Worth brand assets to view, clone, fork, build, and run the original Worth project as-is, and to make factual references to Worth, such as saying that another project is "based on Worth" or "forked from Worth".

Please use your own name and branding for modified or unofficial versions that you publish, distribute, or present as a separate project. Public packaged builds or other uses of the Worth branding outside the original project need permission from Callum Watkins. For permission, contact `callum@callumwatkins.com`.

## Source file

`logo-system.ai` is the Adobe Illustrator source project for the Worth logo system.

It contains the editable master artboards for the core symbol, app icon tiles, macOS icon tiles, and combination marks.

Changes to the wordmark in `logo-system.ai` will require the [Readex Pro](https://fontsource.org/fonts/readex-pro) font.

## Tauri icon exports

> [!NOTE]
> See the [Tauri App Icons documentation](https://v2.tauri.app/develop/icons).

The top-level files in `brand/tauri/` are exported as:

- `32x32.png` and `128x128.png` from the medium app tile artboard
- `128x128@2x.png` from the large app tile artboard, exported at 256×256
- `icon.png` from the large app tile artboard, exported at 512×512

### Windows ICO

The PNG layers for the Windows icon are exported to `brand/tauri/ico-src/` as:

- `16.png`, `24.png`, `32.png`, `48.png`, and `64.png` from the medium app tile artboard
- `256.png` from the large app tile artboard

`png-to-ico` is used to combine the PNG layers into a single ICO file, placing the 32x32px icon first, since Tauri always uses the first icon for the taskbar instead of choosing intelligently ([tauri-apps/tauri#14596](https://github.com/tauri-apps/tauri/issues/14596)):

```bash
bun run brand:generate-ico
```

### macOS ICNS

The PNG layers for the macOS icon are exported to `brand/tauri/icns-src/` as:

- `16.png`, `32.png`, `64.png`, `128.png`, and `256.png` from the medium macOS app tile artboard
- `512.png` and `1024.png` from the large macOS app tile artboard

`icon-gen` is used to combine the PNG layers into a single ICNS file:

```bash
bunx icon-gen -i ./brand/tauri/icns-src -o ./brand/tauri/ --icns --icns-name icon --icns-sizes 16,32,64,128,256,512,1024 -r
```

### Copying into Tauri

After exporting and generating, the following files are copied into `src-tauri/icons/`: `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.ico`, and `icon.icns`.

Tauri then references these icons from `src-tauri/tauri.conf.json5`.
