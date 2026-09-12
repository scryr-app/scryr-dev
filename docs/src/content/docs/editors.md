---
title: Editor highlighting
description: Treat .scry files as Python in VS Code, Zed, Neovim, JetBrains IDEs, and Ruff.
---

Because `.scry` uses Python syntax, associate the extension with Python in your editor.

## VS Code

Add a workspace setting:

```json title=".vscode/settings.json"
{
  "files.associations": {
    "*.scry": "python"
  }
}
```

Install or enable the official Python extension for syntax highlighting, language services, and completions.

## Zed

```json title=".zed/settings.json"
{
  "file_types": {
    "Python": ["py", "scry"]
  }
}
```

## Neovim

```lua title="init.lua"
vim.filetype.add({ extension = { scry = "python" } })
```

## JetBrains IDEs

Open **Settings → Editor → File Types → Python**, then add `*.scry` to the registered patterns.

## Ruff

The Scryr CLI already treats `.scry` as Python. To run Ruff directly, add:

```toml title="pyproject.toml"
[tool.ruff]
extension = { scry = "python" }
extend-include = ["*.scry"]
```
