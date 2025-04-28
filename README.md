# Amethyst 🚀

**A High-Performance Minecraft: Bedrock Edition Server in Rust** 🦀
---

## ⚠️ Important Notice
This project is **NOT** a port of PocketMine-MP. Amethyst is a new server implementation built from the ground up in Rust to leverage its unique capabilities. Please be aware that this software is in the **early stages of development** and is **not recommended for production use.**

## 🧑‍🤝‍🧑 Join the Community
Connect with fellow contributors and stay up-to-date on our progress by joining our Discord server: [Discord](https://discord.gg/hSTcSRcNcQ)

## 🏗️ Development Status

Amethyst is actively being developed. Here's a look at our current focus:

- 🔧 **Core Network Layer**: Implementing the foundational binary handling and RakNet protocol support. This is our current priority!

## ✨ Key Features (Our Focus)

- ⚡ **Exceptional Performance**: Designed for minimal overhead and high Transactions Per Second (TPS).
- 🛡️ **Robust Memory Safety**: Leveraging Rust's guarantees to prevent common memory-related errors, leading to a more stable server.
- ⚙️ **Efficient Concurrency**: Utilizing Tokio for non-blocking, asynchronous input/output operations, enabling high scalability.

## 🛠️ Getting Started (Development Builds)

```bash
# Clone the repository and navigate to the project directory
git clone [https://github.com/sauoro/amethyst.git](https://github.com/sauoro/amethyst.git)
cd amethyst

# Build a release-optimized binary
cargo build --release

# Run the Amethyst server
./target/release/amethyst
```

## 🗺️ Future Plans (Roadmap)
Our immediate next steps include:

- [ ] Implementing the Minecraft: Bedrock Edition communication protocol.

## 💖 Contributing to Amethyst
We welcome contributions from the community! Here's how you can get involved:

Fork the Repository: Create your own copy of the Amethyst repository.
Create a Feature Branch: Organize your changes in a dedicated branch.
Develop and Test: Implement your features or bug fixes and ensure they are working correctly.
Submit a Pull Request: Propose your changes to the main repository.

## 📜 Licensing
Amethyst is released under the MIT License. For more details, please see the https://www.google.com/search?q=LICENSE file.