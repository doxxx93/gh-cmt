# gh-cmt

AI-powered commit message generator using [GitHub Models API](https://github.com/marketplace/models).

A [GitHub CLI](https://cli.github.com/) extension that analyzes your staged changes and generates [Conventional Commits](https://www.conventionalcommits.org/) messages.

## Install

```bash
gh extension install doxxx93/gh-cmt
```

> Requires [GitHub CLI](https://cli.github.com/) with `gh auth login` completed.

## Usage

Stage your changes and run:

```bash
gh cmt
```

This will:
1. Read your staged changes (`git diff --staged`)
2. Send them to GitHub Models API
3. Generate a commit message
4. Prompt you to **commit**, **edit**, **regenerate**, or **abort**

### Options

```
gh cmt [OPTIONS] [COMMAND]

Commands:
  config  Manage configuration

Options:
  -l, --language <LANG>      Commit message language (default: english)
  -m, --model <MODEL>        GitHub Models model (default: openai/gpt-4o)
  -e, --examples <N>         Previous commits as context (default: 3)
  -y, --yes                  Auto-commit without confirmation
```

### Examples

```bash
# Generate and interactively commit
gh cmt

# Auto-commit without confirmation
gh cmt -y

# Generate in Korean
gh cmt -l korean

# Use a different model
gh cmt -m openai/gpt-4o-mini
```

## Configuration

Set defaults so you don't have to pass flags every time:

```bash
# Interactive setup
gh cmt config

# View current config
gh cmt config --show

# Reset to defaults
gh cmt config --reset
```

Config is saved to `~/.config/gh-cmt/config.yml`:

```yaml
language: english
model: openai/gpt-4o
examples: 3
auto_commit: false
```

CLI flags always override config values.

## How It Works

- Uses your existing GitHub token (`gh auth token`) — no extra API keys needed
- Calls [GitHub Models API](https://github.com/marketplace/models) (OpenAI-compatible chat completions)
- Includes recent commit messages as style reference for the LLM
- Large diffs are automatically truncated to fit API token limits

## Credits

Inspired by [hazadus/gh-commitmsg](https://github.com/hazadus/gh-commitmsg) (Go), reimplemented in Rust with interactive commit flow and configuration support.

## License

[MIT](LICENSE)
