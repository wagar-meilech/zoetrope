# Local installation and updates

Approved by the user's 27 October 2026 request to install on this device and make future project changes easy to install. Extends the already approved executable/external-launch testing seam. Tracked as [issue #14](https://github.com/wagar-meilech/zoetrope/issues/14).

- Install this customized fork as `zoe` on the user's command path. Normal launch runs a compiled release binary directly, without invoking Cargo or a background service.
- Provide `zoe-update` to rebuild and reinstall changes from the same persistent checkout. Use the locked dependency graph. Do not automatically pull, change branches, discard local changes or install the upstream crate.
- Default to a user-owned bin directory, with an explicit destination option for isolated verification and other environments. Support Linux and macOS without adding runtime dependencies to the Rust application.
- Build successfully before replacing the installed binary. Replace atomically so build failures leave the previous installed command usable and running processes can continue.
- Record the checkout location in the updater, handling spaces in paths. Explain a missing checkout and how to reinstall after moving it.
- Document the two daily commands, the source/toolchain requirement for updates, and any PATH setup required. Do not claim a Linux installation is a macOS installation.

Use vertical TDD through the public installer/updater processes and an isolated destination: initial install/launch, update from another working directory and failed-build preservation. Keep implementation small; use a shell installation helper rather than adding an application service or package manager.
