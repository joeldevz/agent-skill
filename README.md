# SkillCtl: The Agent Skill Manager ⬢

**SkillCtl** is the professional standard for managing AI Agent Skills. It allows you to discover, install, and synchronize robust agent behaviors across multiple AI editors (Cursor, Windsurf, Antigravity, VS Code, and more).

> **Concept**: Think of it as `npm` or `cargo`, but for your AI's brain. Instead of packages, you install **skills**—specialized prompts and instructions that give your agent new capabilities.

---

## 🚀 Key Features

- **🛡️ Secure by Design**: Integrity checks (SHA-256) ensure the skills you install are exactly what you expect. No silent changes.
- **🌐 Universal Compatibility**: One tool to rule them all. Works with:
  - Cursor (`.cursorrules`)
  - Windsurf (`.windsurfrules`)
  - Antigravity (`.agent/`)
  - VS Code, GitHub Copilot, Cline, Roo, OpenHands, and more.
- **⚡ Supercharged Workflow**:
  - `search`: Find skills from the community registry instantly.
  - `add`: Install skills directly from GitHub URLs.
  - `list`: Keep track of your installed capabilities.
- **✨ Zero Config**: Smart defaults that just work.

---

## 📦 Installation

You don't need to install anything globally. Run it directly with `npx` (Node.js) or `cargo` (Rust).

### Using Node.js (Recommended)

```bash
npx skillctl init
```

### Using Rust

```bash
cargo install skillctl
skillctl init
```

---

## 📖 Usage Guide

### 1. Initialize your Project

Sets up the necessary configuration files and detects your AI editor automatically.

```bash
npx skillctl init
```

### 2. Discover Capabilities

Search the decentralized registry for new skills.

```bash
npx skillctl search
```

> _Select a skill from the list to install it immediately._

### 3. Add a Skill Manually

Install a skill directly from a Git repository or URL.

```bash
npx skillctl add <url> --skill <name>
```

### 4. Verify Installation

See what skills are currently active in your environment.

```bash
npx skillctl list
```

### 5. Restore & Sync

Downloading a project? Restore all skills defined in `skills.json` with a single command.

```bash
npx skillctl install
```

---

## 🔧 Architecture

SkillCtl creates a `.agent/skills` (or editor-specific) directory and injects a reference into your editor's rule file.

**Directory Structure:**

```
my-project/
├── .agent/
│   ├── skills/
│   │   └── find-files/
│   │       └── SKILL.md  <-- The Brain
│   └── rules/
│       └── rules.md      <-- The Context Linker
├── skills.json           <-- Lockfile (Registry & Integrity)
└── src/
```

---

## 🤝 Contributing

We welcome contributions to the **Registry**!
To add your skill, submit a PR to `registry.json` with your skill's details.

## 📄 License

Proprietary Software. See [LICENSE](LICENSE) for details.
Built with ❤️ for the AI Engineering Community.
