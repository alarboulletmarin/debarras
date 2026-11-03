# Performance

Measured on the author's machine (Intel Core i9-13900H, 20 threads, NVMe SSD, Linux, release build).

## Start

The window is on screen 35 to 45 ms after launch, its first frame drawn by 100 ms.

On Linux, a machine without an NVIDIA card but with NVIDIA's Vulkan driver installed would wait about 2 s for that driver to give up: Débarras tells the Vulkan loader to skip it. Setting `VK_LOADER_DRIVERS_DISABLE` yourself overrides this.

## The scan

| Folder | Files | On disk | Scan |
|---|---|---|---|
| The author's home folder | 3,381,621 | 487 GB | 0.64 s with the system's caches warm |

Every folder is read by its own task, on all the processor's cores. Right after a restart, when the system has not read the disk yet, the same scan waits for the SSD and takes longer; a spinning disk is much slower.

Files under 1 MB are summed per folder, so the tree Débarras keeps stays small: the suggestions for those 3.4 million files are computed in 40 ms.

## The plan under the pointer

Moving the pointer over the plan only moves the red marker and rewrites the title block: the plots are drawn once per folder, theme and window size, then reused.

| Plan | Work per pointer move |
|---|---|
| The demo | 0.8 ms |
| 100,000 folders of ten files | 0.6 ms |
| The demo, drawn on screen by a software GPU (lavapipe) | 1.7 ms per frame |

A screen at 120 Hz leaves 8 ms per frame.

## Where the figures come from

The first two rows of the second table, and the scan, come from benchmarks; the last row from the running app. To measure on your machine:

```bash
cargo test --release --locked -- --ignored --nocapture bench
DEBARRAS_BENCH_DIR=/some/folder cargo test --release --locked -- --ignored --nocapture bench_scan
```
