# Token Vault

A multi-chain wallet manager built with Tauri, React, and TypeScript.

## Features

- Multi-chain support (Ethereum, BNB Chain, Polygon, Arbitrum, Optimism, Avalanche)
- Secure wallet generation and management
- Balance checking across multiple chains
- Transaction signing and broadcasting

## Development

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Tauri CLI

### Setup

```bash
# Install dependencies
npm install

# Run development server
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

## Tech Stack

- **Frontend**: React 18, TypeScript, Vite
- **Backend**: Rust, Tauri 2
- **Blockchain**: Alloy (EVM compatibility)