# Worktree management over worktrunk (`wt`), adding the project conventions: branch from a fresh
# origin/main, and open the zellij dev layout. `wt` runs through `mise exec`, so it resolves
# without shell activation.

wt := "mise exec -- wt"

[doc("Create a worktree (and branch) from a fresh origin/main")]
new branch:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "==> Fetching origin/main"
    git fetch origin main
    echo "==> Creating worktree for '{{branch}}' from origin/main"
    {{wt}} switch --create "{{branch}}" --base origin/main --no-cd
    echo "==> Ready. 'just wk::list' to see it; 'just wk::zellij {{branch}}' for a dev tab."

[doc("Create a worktree and open it in a zellij tab running the dev layout")]
[arg("focus", long="no-focus", value="false", help="Leave focus on the current tab")]
[arg("prompt", long="prompt", help="Initial prompt for the tab's pi session")]
new_ui branch focus="true" prompt="": (new branch) (zellij branch focus prompt)

[doc("List all worktrees and their status")]
list:
    @{{wt}} list

[doc("Open an existing worktree in a zellij tab running the dev layout")]
[arg("focus", long="no-focus", value="false", help="Leave focus on the current tab")]
[arg("prompt", long="prompt", help="Initial prompt for the tab's pi session")]
[positional-arguments]
zellij branch focus="true" prompt="":
    #!/usr/bin/env bash
    set -euo pipefail
    prompt="$3"
    if [ -z "${ZELLIJ_SESSION_NAME:-}" ]; then
        echo "Error: not inside a zellij session. Start zellij first."
        exit 1
    fi
    worktree_path="$(just wk::_path "{{branch}}")"
    if [ -z "${worktree_path}" ]; then
        echo "Error: no worktree for branch '{{branch}}'. Run 'just wk::new {{branch}}' first."
        exit 1
    fi
    # The layout's pi pane takes this file as its initial prompt.
    handoff="${worktree_path}/handoff.ignore.md"
    if [ -n "${prompt}" ]; then
        printf '%s\n' "${prompt}" > "${handoff}"
    else
        rm -f "${handoff}"
    fi
    focus_flag=""
    if [ "{{focus}}" = "false" ]; then
        focus_flag="--no-focus"
    fi
    zellij action new-tab ${focus_flag} \
        --layout "${worktree_path}/.zellij/layout.kdl" \
        --cwd "${worktree_path}" \
        --name "{{branch}}"

[doc("Remove a worktree and delete its branch (pass 'force' to drop an unmerged branch)")]
rm branch=`git branch --show-current` force="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{branch}}" = "main" ]; then
        echo "Error: refusing to remove 'main'."
        exit 1
    fi
    original_dir="${PWD}"
    worktree_path="$(just wk::_path "{{branch}}")"

    echo "==> Removing worktree '{{branch}}'"

    # Removing the worktree this recipe stands in would delete its cwd.
    cd "$(git worktree list --porcelain | awk '/^worktree /{print $2; exit}')"

    # Without --force-delete, wt deletes the branch only if it is merged.
    if [ "{{force}}" = "force" ]; then
        {{wt}} remove "{{branch}}" --force --force-delete --foreground
    else
        {{wt}} remove "{{branch}}" --force --foreground
    fi

    if [ -n "${ZELLIJ_SESSION_NAME:-}" ] && [ -n "${worktree_path}" ] \
        && [[ "${original_dir}" == "${worktree_path}"* ]]; then
        zellij action close-tab
    fi

# Print BRANCH's worktree path, empty if it has none. Asks git, since the directory name can
# differ from the branch.
[private]
_path branch:
    @git worktree list --porcelain | awk -v b="refs/heads/{{branch}}" '/^worktree /{p=$2} $0=="branch "b{print p}'
