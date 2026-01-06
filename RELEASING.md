# Releasing rdt

## Publishing to crates.io

1. Create account at https://crates.io and login:
   ```bash
   cargo login
   ```

2. Verify the package:
   ```bash
   cargo publish --dry-run
   ```

3. Publish:
   ```bash
   cargo publish
   ```

## Creating a GitHub Release

1. Update version in `Cargo.toml`

2. Commit and tag:
   ```bash
   git add -A
   git commit -m "Release v0.1.0"
   git tag v0.1.0
   git push origin main --tags
   ```

3. The GitHub Action will automatically build and create a release with binaries

## Setting up Homebrew Tap

1. Create a new GitHub repo called `homebrew-tap`

2. Add the formula file `Formula/rdt.rb` (template in `homebrew/rdt.rb`)

3. After a release, update the formula:
   - Update the `version`
   - Download each binary and compute SHA256:
     ```bash
     curl -L https://github.com/sergical/rdt/releases/download/v0.1.0/rdt-aarch64-apple-darwin.tar.gz | shasum -a 256
     ```
   - Update the `sha256` values in the formula

4. Users can then install with:
   ```bash
   brew install sergical/tap/rdt
   ```

## Version Checklist

- [ ] Update `version` in `Cargo.toml`
- [ ] Update `version` in `homebrew/rdt.rb`
- [ ] Commit changes
- [ ] Create and push git tag
- [ ] Wait for GitHub Action to complete
- [ ] Publish to crates.io
- [ ] Update Homebrew formula with SHA256 hashes
