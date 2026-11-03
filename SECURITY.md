# Security policy

## Supported versions

Only the latest release receives fixes. Please check that you run it before reporting (`debarras --version`, or the settings).

## Reporting a vulnerability

Do not open a public issue. Use the [private report form](https://github.com/alarboulletmarin/debarras/security/advisories/new) (*Security* tab, *Report a vulnerability*): only the maintainer sees it.

Please include the version, the operating system, and the steps or the shape of the folder that trigger the problem.

You can expect a first answer within a week. A confirmed vulnerability is fixed in a new release, and the advisory is published with it, crediting you unless you prefer otherwise.

## What matters most

- Anything that deletes, moves or overwrites a file, or that could act on another file than the one shown.
- The scan: links, mounted disks and folders it must not follow, names it must not misread.
- Anything that writes outside the configuration folder.
