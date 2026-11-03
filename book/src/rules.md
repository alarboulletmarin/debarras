# What can go: the rules

Débarras marks a folder or a file only when its name says, without doubt, what it is. It never looks inside files, and it never guesses from a date.

## Hatching: can go without loss

| Folder | Why |
|---|---|
| `node_modules` | npm dependencies, installed again by `npm install`. |
| `target` (not at the root of the scan) | Rust build output, rebuilt by `cargo build`. |
| `__pycache__`, `.pytest_cache`, `.mypy_cache`, `.ruff_cache` | Python cache, rebuilt on its own. |
| `.gradle` | Gradle cache, downloaded again by the next build. |
| `.next`, `.turbo`, `.svelte-kit`, `.parcel-cache` | Web build cache, rebuilt by the next build. |
| `.cargo/registry` | Downloaded crate sources, fetched again when needed. |
| `.npm` | npm download cache, filled again when needed. |
| `Xcode/DerivedData` | Xcode build output, rebuilt by the next build. |
| `.cache` | User cache: applications rebuild it. |
| `Library/Caches` | macOS caches: applications rebuild them. |
| `Local/Temp` | Windows temporary files. |
| `.Trash`, `Trash`, `$Recycle.Bin` | Trash: already deleted, not yet emptied. |

## Dots: to review

| Item | Why |
|---|---|
| `ollama`, `lm-studio`, `huggingface` | Local AI models: large and downloadable again, but slow to get back. |
| `Steam` | Installed games: remove those you no longer play. |
| `docker`, `containers` | Container images and volumes: their prune command frees the unused ones. |
| `.iso`, `.img`, `.dmg` files over 500 MB | Disk image: often kept after an install. |

## Everything else

Any other plot is left plain. Débarras only gives it a family colour, from its name (`Documents`, `Pictures`, `Steam`…) or its extension (`.mp4`, `.pdf`, `.zip`, `.gguf`…).

A rule that marks too much is worse than a missing one. To suggest a rule, open an issue with the folder's name, the tool that creates it, and the command that brings it back.
