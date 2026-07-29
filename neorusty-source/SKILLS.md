---
name: neoforged-neoforge
description: "Guides agents through working with the neoforged/NeoForge codebase (Java, Shell). Use when working with or extending NeoForge, or when the user mentions 'NeoForge', 'neoforged/NeoForge', or asks about its patterns, setup, or configuration. Not for general Java, Shell questions unrelated to NeoForge."
---

# Neoforge Code Skill

## Overview

Unless noted below, NeoForge and all its parts here in this repository are licensed under the **GNU Lesser General Public License v2.1** (LGPL v2.1), as seen at http://www.gnu.org/licenses/old-licenses/lgpl-2.1.txt and reproduced in the `LICENSE-LGPLv2.1` at the root of this repository.

Primary languages: Java, Shell.

## When to Use

- Understanding the architecture and module layout of NeoForge
- Extending or modifying NeoForge consistent with its existing patterns
- Debugging issues by tracing through NeoForge's modules and dependencies
- Setting up, running, or configuring NeoForge
- Calling functions, classes, or methods in NeoForge's public API

**When NOT to use:** General Java, Shell questions, tutorials, or tasks unrelated to the NeoForge codebase.

**Related:** For general Java, Shell guidance, use language-specific skills instead.

## Core Process

### Step 1: Understand the Architecture

Read the existing code in NeoForge before making changes. Check `references/architecture.md` to understand the module layout, dependency graph, and internal import structure. The goal is to extend existing patterns, not invent new ones.

### Step 2: Locate Relevant Modules

Use `references/api.md` to find the public symbols, functions, and classes relevant to the task. Trace the call chain through NeoForge's internal imports to understand how the pieces connect.

### Step 3: Make Changes Following Existing Patterns

Implement the change consistent with NeoForge's established conventions: naming patterns, error handling style, module organization, and test structure. Consistency matters more than personal preference.

### Step 4: Verify the Change

Run the project's test suite and confirm all tests pass. If no tests exist for the changed behavior, write them first. Check that no regressions were introduced in adjacent modules.

## Common Rationalizations

| Rationalization | Reality |
|---|---|
| "I know NeoForge well enough to skip reading the existing code" | Every session starts with stale context. Re-read the architecture reference before assuming you know the current state. |
| "This change is too small to need tests" | Small changes in unfamiliar codebases cause the most subtle regressions. A test that fails without the fix and passes with it is the minimum bar. |
| "I'll follow the patterns later, let me just get it working first" | Pattern violations compound. Code that works but violates the repository's conventions creates maintenance debt for every future contributor. |

## Red Flags

- Making changes to NeoForge without reading `references/architecture.md` first
- Inventing new patterns instead of extending existing ones
- Skipping the test suite before declaring the task complete
- Modifying code outside the scope of the current task

## Verification

Before declaring this workflow complete, confirm each item with evidence:

- [ ] Changes follow NeoForge's existing patterns — evidence: diff review against `references/architecture.md`
- [ ] All tests pass — evidence: test runner output
- [ ] No regressions introduced in adjacent modules — evidence: full test suite output
- [ ] Code is consistent with the repository's naming and style conventions — evidence: code review

## References

- [Architecture & dependencies](references/architecture.md)
- [Usage examples](references/examples.md)
- [Configuration](references/config.md)
- [Full Code Digest](references/neoforged_neoforge_digest.txt)
