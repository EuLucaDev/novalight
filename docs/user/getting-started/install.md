---
title: Install Nova Light
description: Download Nova Light for Windows 11, macOS or Linux, install it, and open your photo library.
order: 10
verified: 0.7.1
---

Download the file for your system from the [latest release](https://github.com/EuLucaDev/novalight/releases/latest) and install it. In the file names, `<version>` stands for the version number, for example `0.7.1`. The [Nova Light page on workflowind.agcunae.com/apps](https://workflowind.agcunae.com/apps/novalight) also links the download.

## Windows 11

1. Open the [latest release](https://github.com/EuLucaDev/novalight/releases/latest).
2. Download `novalight-<version>-windows-x64.msi`. It fits most PCs (Intel and AMD, 64-bit).
3. Run the file. The installer asks where to install. The default is `C:\Program Files\Nova Light`, and upgrades stay in the folder you chose.
4. At the end, choose the option to start Nova Light.

For a Snapdragon or other Arm PC, download `novalight-<version>-windows-arm64.msi` instead. For a 32-bit Windows, download `novalight-<version>-windows-x86.msi`.

Upgrading from an installer older than 0.6.0 also keeps Nova Light in the folder it was installed in. If that folder was deleted, the upgrade installs to `C:\Program Files\Nova Light`.

To run it without installing, download `novalight-<version>-windows-x64-portable.zip`. Portable builds for `arm64` and `x86` are on the release page too.

## macOS

1. Download `novalight-<version>-macos-universal.dmg`. It works on Apple silicon and Intel Macs, and is signed and notarized.
2. Open the file and install Nova Light.

## Linux

The release has these files for `x86_64` and `aarch64` (Arm). Replace `x86_64` with `aarch64` on Arm.

| File                                         | Notes                                                    |
|----------------------------------------------|----------------------------------------------------------|
| `novalight-<version>-linux-x86_64.AppImage` | Runs anywhere.                                           |
| `novalight-<version>-linux-x86_64.flatpak`  | Sandboxed. Install with `flatpak install --user <file>`. |
| `novalight-<version>-linux-x86_64.deb`      | For Debian and Ubuntu.                                   |
| `novalight-<version>-linux-x86_64.rpm`      | For Fedora, RHEL and openSUSE.                           |
| `novalight-<version>-linux-x86_64.tar.gz`   | Unpack anywhere.                                         |

RISC-V and FreeBSD builds are on the release page too. On a Wayland desktop, Nova Light runs through XWayland when it is available, because only X11 gives it file drag-and-drop.

## Install with Nova Foods Launcher

[Nova Foods Launcher](https://workflowind.agcunae.com/apps/docs/novafoods-launcher) lists Nova Light and can install and update it for you. On Windows it installs the release `.msi`.

## In a web browser

The release also has `novalight-web-<version>.zip`, a static site that runs Nova Light in a modern browser. Host it on any static web server.

## First run

1. Start Nova Light. It opens your library at `~/Pictures/Nova Light Library`. A new library starts with demo photos.
2. To bring in your own photos, see [Import photos](../guides/import-photos.md).

## Still stuck?

Ask in [Suporte](https://workflowind.agcunae.com/inbox) or open an issue in the [issue tracker](https://github.com/EuLucaDev/novalight/issues).
