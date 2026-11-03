# Troubleshooting

## The window does not open (Linux)

Débarras needs a Vulkan driver. Run `vulkaninfo --summary` (package `vulkan-tools`): if it lists no device, install the Vulkan driver of your GPU (`vulkan-radeon`, `vulkan-intel`, `nvidia-utils`…). On a machine without a GPU, `mesa`'s software driver (lavapipe) works, slowly.

## "Not found" or "Permission denied"

The start screen shows why a folder could not be scanned. A folder that disappeared since the last scan says *not found*; a folder you may not read says *permission denied*. Choose another folder or try the demo.

## Some items were skipped

The panel says how many items the scan could not read (*unreadable*), usually system folders that belong to another user. The total does not count them. Scanning as administrator would read them; Débarras does not ask for it, and you do not need it for your own files.

## macOS asks for permission folder after folder

Give Débarras **Full Disk Access** in **System Settings**, **Privacy & Security**.

## The total differs from my file manager

Débarras counts the space files take on the disk, counts a hard-linked file once, and stays on one file system. A file manager often shows file lengths, may count each hard link, and may count other disks mounted inside the folder. On Linux and macOS, `du -sxh` should agree with Débarras, give or take a few kilobytes per folder.

## The wrong language

See [Settings, Language](settings.md#language).

## Reporting a problem

Open an issue on [GitHub](https://github.com/alarboulletmarin/debarras/issues) with your system, the output of `debarras --version`, and what you did. Starting Débarras from a terminal shows its error messages.
