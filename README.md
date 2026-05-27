# Telegram MCP

A fast, pure-Rust [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server that connects AI agents to Telegram. 

## Overview

This server provides over 40 tools that let an LLM act as your Telegram client. It supports:
- **Messaging:** Send, edit, forward, delete, and react to messages.
- **Media:** Download and upload photos, files, and voice notes.
- **Contacts & Chats:** Manage groups, ban users, update profiles, and read chat history.

Because it's written in Rust, it has no Node.js dependencies, even when running the HTTP/SSE transport.

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

### Option A: Local Use (Stdio)
This is the standard way to use MCP with desktop apps like Claude or Cursor.

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

### Option B: Network Service (HTTP/SSE)
If you are building a custom integration or want to access the server over a local network, you can start the built-in web server.

```bash
./target/release/telegram-mcp-rs --sse 8000 --host 0.0.0.0
```
Your clients can now connect to `http://localhost:8000/sse`.

## Security

By default, the server will block attempts to read or write files to your hard drive. 

If you want the AI to be able to upload or download media, you **must** specify an allowed root directory. Any paths outside this directory will be rejected. You can configure this via the `TELEGRAM_ALLOWED_ROOTS` variable in your `.env` file or by passing it as a positional argument to the binary.

## License

MIT
