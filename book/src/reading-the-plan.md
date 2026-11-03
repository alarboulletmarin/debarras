# Reading the plan

The plan is a squarified treemap: each plot's area is its size, and plots are laid out as close to squares as the room allows, the largest first.

## Folders

A folder shows its name in a header strip, with its contents drawn inside, up to three levels deep. Click it to open it full size.

## Colours

The background colour of a plot tells its family:

| Family | What goes there |
|---|---|
| System | the operating system's own folders |
| Applications | installed apps, Flatpak, `Program Files` |
| Development | build output, dependencies, toolchains |
| Media | photos, music, videos |
| Documents | `Documents`, `Downloads`, PDF, office files |
| Caches | caches that applications rebuild |
| Archives | `.zip`, `.tar.*`, `.7z`, disk images |
| AI models | `.gguf`, `.safetensors`, local model folders |
| Other | everything else |

The legend in the panel repeats these colours. They are tints of the current [theme](settings.md), so they stay readable in every theme.

## Hatching and dots

- **Hatching**: can go without loss. It is rebuilt or downloaded again when needed.
- **Dots**: large, to review. You decide.

The [rules](rules.md) page lists every case and the reason Débarras gives for it.

## Small items

Items too small to draw are gathered into one plot, such as **12 others**. Files under 1 MB of the same folder are summed into one plot, such as **340 small files**. Neither opens: they are there so the total stays honest.

## The dimension line and the title block

Above the plan, the dimension line gives the size of the folder shown and how many items it holds. Below it, the title block names the hovered plot, its size, its share of the folder, its family, and why it can go when it can.
