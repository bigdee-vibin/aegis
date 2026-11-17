# Building locally without MSVC or dlltool (Windows)

CI is the source of truth: it builds the **unpinned** dependency set with the MSVC toolchain on Windows, plus macOS and Linux.

On a Windows machine with neither Visual Studio Build Tools nor MinGW, the default MSVC target cannot link (`link.exe` not found) and the GNU target fails on `windows-sys` 0.61 (needs `dlltool.exe`). Use one of:

1. Install Visual Studio Build Tools with the C++ workload (matches CI exactly), or
2. Install a MinGW toolchain that provides `dlltool` and use `rustup toolchain install stable-x86_64-pc-windows-gnu`, or
3. As a stopgap, build in a throwaway copy with older versions that avoid the `windows-sys` 0.61 import libraries. Do **not** commit these pins:

```toml
# in the throwaway copy's workspace Cargo.toml
tokio = { version = "=1.43.1", features = [...] }
axum = "=0.8.1"
tower = { version = "=0.5.2", features = ["util"] }
```

then `cargo update -p mio --precise 1.0.3`.

Results obtained with option 3 are reported as "pinned scratch build" and never as CI-equivalent.
