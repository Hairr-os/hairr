# hairr OS

A next-generation, secure, and AI-native operating system built from the ground up.

## Overview

hairr OS is a modern operating system architected on four core principles:

- **Security by Design**: Rust-based microkernel architecture with capability-based security
- **Universal Adaptability**: Modular design scaling from smart glasses to servers
- **Future-Native Integration**: First-class support for AI, robotics, and decentralized identity
- **Seamless Compatibility**: Broad application compatibility through the Chrysalis compatibility suite

## Architecture

The project follows a monorepo structure with Cargo workspaces:

```
hairr-os/
├── kernel/             # The microkernel
├── libs/               # Shared libraries (IPC, HAL traits, etc.)
├── services/           # Core userspace services
├── drivers/            # Userspace device drivers
├── shell/              # Desktop shell and utilities
├── compatibility/      # Linux/Android compatibility suite (Chrysalis)
├── pkg-manager/        # Native package management
└── apps/               # First-party App Store and native apps
```

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## Contributing

hairr OS is an open-source project. Contributions are welcome!

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
