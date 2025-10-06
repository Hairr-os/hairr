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

## Components

### Core Kernel
- **Microkernel**: Process and thread management with capability-based security
- **Priority Scheduling**: Support for real-time, interactive, and batch workloads
- **Memory Management**: Isolated address spaces with secure memory allocation

### Libraries
- **IPC**: High-performance inter-process communication with message passing
- **HAL**: Protocol-centric Hardware Abstraction Layer with vendor-independent traits
- **Capability System**: Fine-grained access control for resources

### Services
- **Device Manager**: Hardware device lifecycle and driver management
- **AI Scheduler**: AI-aware workload scheduling with GPU acceleration support
- **Keystore**: Hardware-backed cryptographic key management and decentralized identity

### Applications
- **Desktop Shell**: Modern windowing system with multi-tasking support
- **Package Manager**: Native package installation and dependency management
- **App Store**: Graphical application discovery and management interface
- **Chrysalis**: Virtualization-based Linux and Android compatibility layer

## Features

### Security
- Capability-based access control
- Hardware-backed keystore
- Process isolation via microkernel design
- Sandboxed compatibility layer

### AI-Native
- AI-aware scheduler for optimal ML workload placement
- First-class support for GPU acceleration
- Dedicated AI inference and training workload types

### Compatibility
- Run unmodified Linux applications via Chrysalis
- Android app support through virtualization
- Docker daemon support in Linux containers
- Automatic foreign binary detection

### Package Management
- Native package manager with CLI
- Graphical App Store interface
- Support for third-party app stores via API
- Automatic dependency resolution

## Building

```bash
# Build all components
cargo build --release

# Build a specific component
cargo build -p kernel --release
cargo build -p shell --release
```

## Testing

```bash
# Run all tests
cargo test

# Run tests for a specific component
cargo test -p kernel
cargo test -p ipc
```

## Running

### Desktop Shell
```bash
cargo run -p shell
```

### Package Manager
```bash
cargo run -p pkg-manager
```

### App Store
```bash
cargo run -p app-store
```

## Development Status

This is an early-stage prototype implementing core functionality outlined in the PRD. Current status:

✅ Microkernel with process management  
✅ IPC system with message passing  
✅ Capability-based security framework  
✅ Hardware Abstraction Layer  
✅ Device manager service  
✅ AI-aware scheduler  
✅ Hardware-backed keystore  
✅ Desktop shell prototype  
✅ Package manager with CLI  
✅ App Store interface  
✅ Chrysalis compatibility layer foundation  

### Roadmap
- [ ] Full hardware driver implementations
- [ ] Complete AI acceleration API
- [ ] Enhanced real-time scheduling guarantees
- [ ] Graphical UI for App Store
- [ ] Full Docker support in Chrysalis
- [ ] Android app compatibility in Chrysalis
- [ ] A/B partition update system
- [ ] Smart glasses form factor support

## Contributing

hairr OS is an open-source project. Contributions are welcome! Please see the PRD (prd.md) for design guidelines and architectural principles.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
