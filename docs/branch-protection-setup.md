# Branch Protection Setup — SEC-04 Runbook

## Context
Branch protection prevents force-pushes, requires CI to pass, and enforces code review.

## Steps (GitHub UI)

### 1. Enable for `main` branch

1. Go to: `https://github.com/mapleDevJS/netspeed-cli/settings/branches`
2. Click **"Add rule"** for branch: `main` (or `master`)
3. Enable these settings:
   - ✅ **Require a pull request before merging**
     - Required approvals: `1`
     - ✅ Require approvals
     - ✅ Dismiss stale pull request approvals when new commits are pushed
   - ✅ **Require status checks to pass before merging**
     - ✅ Require branches to be up to date before merging
     - Required status checks: `CI` (or `quick-checks`, `build`, `test`)
   - ✅ **Require conversation resolution before merging**
   - ✅ **Restrict who can push to matching branches** (optional — only admins)
   - ✅ **Do not allow bypassing the above settings**
   - ✅ **Require linear history** (prevents merge commits — use rebase/squash)
   - ✅ **Restrict pushes that create new branches** (optional)
4. Click **"Save changes"**

### 2. Enable for `staging` branch

Repeat step 1 for branch `staging` with the same settings.

### 3. Enable for `develop` branch

Repeat step 1 for branch `develop` with the same settings.

## Verification

After enabling, test the rules:

```bash
# Should fail — direct push to main blocked
git checkout main
echo "test" >> README.md
git commit -am "test: direct push"
git push origin main  # Should be rejected by GitHub

# Should succeed — via PR
git checkout -b test/pr-flow
echo "test" >> README.md
git commit -am "test: PR flow"
git push origin test/pr-flow
# Create PR: test/pr-flow → main
# Wait for CI to pass, then merge
```

## Expected Result
- Direct pushes to `main`, `staging`, `develop` are blocked
- All changes must go through PRs with passing CI
- No force-pushes allowed
- Linear git history (no merge bubbles)
