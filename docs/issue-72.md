title:	feature request: Tab complishing on 'gwt sw' command
state:	OPEN
author:	troydai
labels:	
comments:	1
assignees:	
projects:	
milestone:	
number:	72
--
The gwt sw command require the user to type in the full branch name today. Enable tab completion on the gwt sw command improve user experience. Here's the requirement: Type in `gwt sw` and tap once, the command list all the available worktree branches that user can type through to choose. Once user pick one, they type the command is filled with the branch name. Tab completion is a common feature among CLIs, use the most common implementation method.

## Execution Plan

**Repository Analysis:**
- **Language:** Rust
- **CLI Framework:** `clap` (v4.5.53)
- **Current State:** No `clap_complete` dependency found in `Cargo.toml`.

### Execution Plan: Add Tab Completion for `gwt sw`

**Objective:** Enable shell tab completion for the `gwt sw` command to suggest available git worktree branches dynamically.

**Phase 1: Dependencies & Setup**
1.  **Update `Cargo.toml`:** Add `clap_complete` to dependencies.
2.  **Completion Command:** Add a `completion` subcommand to `gwt` that generates static shell scripts using `clap_complete`.

**Phase 2: Implementation (Dynamic Completion)**
3.  **Implement Raw Listing in `gwt`:** 
    - Modify the `gwt ls` command or add a hidden flag (e.g., `gwt ls --raw`) that outputs only the branch names, one per line, without styling or markers.
    - **Why call `gwt` and not `git`?** 
        - **Consistency:** Ensures suggestions align with `gwt`'s specific configuration and `worktree_root`.
        - **Simplicity:** Keeps the parsing logic in Rust; the shell script only needs to consume a simple list.
        - **Performance:** Rust's startup time is negligible for this use case.

4.  **Integration with Shell Scripts:**
    - The generated completion script must be "bridged" to call `gwt ls --raw` when the user is completing the `branch` argument for `gwt sw`.
    - Use `ValueHint::Other` in `clap` or manually inject the call into the generated completion function logic.

**Phase 3: Verification & Documentation**
5.  **Test:** Generate, source, and verify the script in Bash and Zsh.
6.  **Documentation:** Update `README.md` with installation instructions (e.g., adding `source <(gwt completion bash)` to `.bashrc`).

