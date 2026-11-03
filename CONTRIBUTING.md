# Contributing to Débarras

Thank you for taking the time. Débarras is a small app with a narrow goal: show what fills a disk, and help free what can go, safely. Contributions that keep it that way are welcome.

By taking part you agree to the [code of conduct](CODE_OF_CONDUCT.md). For a security problem, follow the [security policy](SECURITY.md) instead of opening an issue.

## Before you write code

- **A bug**: open an issue with the version, the system and the steps to reproduce.
- **A feature**: open an issue first and wait for an answer. Many ideas are declined to keep the app small; it is better to know before the work is done.
- **A new rule** (something that can be freed): say what creates the folder, what rebuilds it, and whether anything is lost when it goes.
- **A typo, a small fix**: a pull request is enough.

## Building and testing

```bash
cargo run -- --demo       # debug build, on the made-up disk
cargo run -- ~/projects   # on a real folder
cargo run --release       # what gets installed
cargo test --locked       # or: make test
make docs                 # rewrite the shortcuts page, build the user guide (needs mdbook)
```

The system packages needed on each platform are listed in the [install guide](book/src/install.md). The tests need no display: they run on GPUI's test platform and drive the app with simulated keystrokes and clicks.

Measures are ignored tests: `cargo test --release --locked -- --ignored --nocapture`. Their results are in the guide's [Performance](book/src/performance.md) page.

## Rules of the code base

- **Read [ARCHITECTURE.md](ARCHITECTURE.md) first**: the layers (`logic`, `platform`, `ui`, `views`, `shell`), which way they depend, and where each feature lives. A file that passes 1,500 lines gets split by subject.
- **GPUI is pinned to 0.2.2** and its API moves. Check a signature in the source of that version, not in the documentation of `main`.
- **`Cargo.lock` is fragile**: it started as the one GPUI 0.2.2 was published with. No global `cargo update`; one dependency at a time. No new dependency without an agreement in the issue.
- **Code, comments**, commit messages, the README and issues are in English.
- **Interface texts** always go through `tr("English", "Français")`. The tests run in French; the French interface says « vous ».
- **Nothing is deleted without the user's say.** A rule that marks something « without loss » must be true for every user.
- **A deliberate shortcut** carries a `// ponytail:` comment that names its limit and what would come next.

## Tests

- Pure functions are tested in their own module, edge values included (0, 1, the first value that no longer fits, a huge one).
- Every user path is tested in `src/shell/tests/`. A new feature adds its path there.

## Documentation

| Reader | Place |
|---|---|
| someone deciding whether to try Débarras | `README.md` and the website, `docs/index.html`. Short. |
| a user | the user guide, `book/src/` (an [mdBook](https://rust-lang.github.io/mdBook/)), one page per subject |
| a contributor | `CONTRIBUTING.md`, `ARCHITECTURE.md` |

A visible feature updates the help panel (`help_sections` in `src/shell/keymap.rs`, then `make docs`: the test `shortcuts_page_is_current` fails until the page matches), the page of the guide on its subject, and the README or the website only for a feature worth a line there. A new rule updates [What can be freed](book/src/rules.md).

Do not add screenshots: there is one, `docs/screenshots/debarras.png` (light and dark side by side), copied to `docs/site/`.

CI builds the guide and checks every link (`.github/workflows/docs.yml`); pushes to `main` publish it with the website.

## Commits and pull requests

- One branch per change: `feat/…` or `fix/…`.
- Subjects start with `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `build:` or `chore:`, in the imperative; the body explains why.
- One commit per coherent step, tests passing at each commit. The history stays linear.
- CI builds and tests on Linux, macOS and Windows; it has to pass.

## Releasing (maintainer)

`scripts/release.sh X.Y.Z` prepares a draft and starts the native packaging workflow. The release is published only after every download has been built, attached and checked. See [RELEASING.md](RELEASING.md).
