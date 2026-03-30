# Agent-Native CLI Design — Learnings

> Research goal: find the best role model for building a CLI that AI agents call as a tool.

## Best Role Model: `gh` (GitHub CLI)

The gold standard. Every command supports `--json` with field selection and `--jq` for inline filtering. Agents use it constantly because it never surprises them.

```bash
gh pr list --json number,title,state --jq '.[] | select(.state=="OPEN")'
```

## The 10 Design Axes

| Axis | Rule |
|---|---|
| **Structured output** | `--json` flag on every command; stdout = data, stderr = human messages |
| **TTY detection** | Auto-switch to JSON when stdout is not a TTY |
| **Exit codes** | Meaningful codes, not just 0/1 — agents branch on failure type |
| **Machine-readable errors** | `{"error": "image_not_found", "code": 44}` not `"something went wrong"` |
| **Schema introspection** | `mytool schema <command>` returns accepted params + return types as JSON |
| **Noun-verb structure** | `mytool resource action` — turns discovery into a tree search |
| **Idempotence** | Agents retry; tool must handle duplicate calls gracefully |
| **NDJSON pagination** | Stream large results line-by-line, don't buffer |
| **No interactive prompts** | Never block on stdin — always provide `--yes`/`--no-input` flags |
| **Predictable output shape** | Same JSON schema every time; no surprise fields, no omitted nulls |

## Key Insight

A well-designed CLI with `--json` and schema introspection is **faster, cheaper, and more reliable** than an MCP server for most agent workflows. Switching from MCP to a well-designed CLI cut token usage by ~40% in one benchmark.

## CLI vs MCP Trade-offs

- **MCP**: persistent tool definitions in system prompt — ambient token cost every call
- **CLI**: just a bash command + stdout — zero overhead, composable with Unix pipes

For simple, deterministic operations → CLI wins.
For stateful, long-running, or conversational tools → MCP makes more sense.

## Reference Projects

- [`gh` CLI](https://github.com/cli/cli) — the template everyone copies
- [Super CLI](https://super-agentic.ai/resources/super-posts/super-cli-first-ever-agent-native-cli/) — explicitly agent-native from day one
- [CLI-Anything](https://github.com/HKUDS/CLI-Anything) — framework for wrapping any software as agent-accessible
- [nibzard — Designing CLI Tools for AI Agents](https://www.nibzard.com/ai-native)
- [Rewrite Your CLI for Agents (Or Get Replaced)](https://www.theundercurrent.dev/p/rewrite-your-cli-for-agents-or-get)
- [lnget PR — 10 agent-CLI design axes in practice](https://github.com/lightninglabs/lnget/pull/14)
