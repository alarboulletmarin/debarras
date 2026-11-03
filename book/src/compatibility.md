# Compatibility

| System | Status |
|---|---|
| Linux, Wayland or X11 | Built and tested by the CI on every change; developed on Arch Linux. |
| macOS 11 or later, Apple Silicon and Intel | Built and tested by the CI; not notarized. |
| Windows 10 or 11, 64-bit | Built and tested by the CI; not signed. |

Débarras draws with the GPU, through Vulkan on Linux and Windows and Metal on macOS. A machine without a working driver cannot open the window: see [Troubleshooting](troubleshooting.md).

## File systems

- The scan stays on the file system of the folder you chose. Another disk mounted inside it is skipped, so a network share does not get walked by accident.
- Symbolic links are not followed: a link counts for itself, not for what it points to.
- Sizes are the space files take on the disk, not their length. A sparse file counts for what it really uses.
- A file with several hard links counts once, in the folder where the scan meets it first. Package stores (pnpm, uv, Nix…) link the same files into many folders: counting each link would inflate the total.
- The blocks a folder takes for its own list of names are not counted: a few kilobytes per folder.
- On Windows, sizes are file lengths, and a hard-linked file counts once per link.
