# Telegram MCP

A fast, pure-Rust [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server that connects AI agents to Telegram.

## Overview

This server provides over 40 tools that let an LLM act as your Telegram client. It supports:
- **Messaging:** Send, edit, forward, delete, and react to messages.
- **Media:** Download and upload photos, files, and voice notes.
- **Contacts & Chats:** Manage groups, ban users, update profiles, and read chat history.

Because it's written in Rust, it is extremely efficient and has zero external runtime dependencies.

## Setup

1. **Get your API Keys:** You need an `API_ID` and `API_HASH` from [my.telegram.org](https://my.telegram.org).
2. **Configure:** Copy the example config and add your keys.
   ```bash
   cp .env.example .env
   # Edit .env with your keys
   ```
3. **Login:** Run the login utility to authenticate your account and generate a `telegram.session` file.
   ```bash
   cargo run --bin login
   ```
4. **Build:**
   ```bash
   cargo build --release
   ```

## Usage

### Local Configuration
Add this to your client's config file (e.g., `claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "telegram": {
      "command": "/path/to/telegram-mcp-rs/target/release/telegram-mcp-rs",
      "args": ["/path/to/allowed/downloads"]
    }
  }
}
```

### Remote / LAN Configuration (SSH)
If your Telegram service is running on a different machine (e.g., a home server at `192.168.1.113`) and you want to connect to it from your desktop, use the SSH method. This forwards the standard I/O (stdio) over your local network securely.

**Prerequisite:** Ensure you have SSH key authentication set up so the connection doesn't prompt for a password.

Add this to your `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "telegram-remote": {
      "command": "ssh",
      "args": [
        "user@192.168.1.113",
        "/full/path/to/telegram-mcp-rs/target/release/telegram-mcp-rs",
        "/path/to/remote/downloads"
      ]
    }
  }
}
```

## Security

By default, the server will block attempts to read or write files to your hard drive. 

If you want the AI to be able to upload or download media, you **must** specify an allowed root directory. Any paths outside this directory will be rejected. You can configure this via the `TELEGRAM_ALLOWED_ROOTS` variable in your `.env` file or by passing it as a positional argument to the binary.

## License

MIT
