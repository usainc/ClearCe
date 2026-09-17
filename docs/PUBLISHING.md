# Publishing ClearCe safely

The original development history contains local-only paths and outdated screenshots. Keep it private. Publish a fresh snapshot of the reviewed current tree; do not run `git push --all`, `--mirror` or push the original phase branch to the public repository.

The Phase 4.2 workflow prepares an independent `main` repository in ignored `.qa/ClearCe-public`. It contains only the current committed source/assets/docs, with no development ancestry, Git configuration, engine weights, installer, logs or QA output.

If creating the public repository was not possible automatically, run from the reviewed, committed original checkout (use a fresh destination):

```powershell
New-Item -ItemType Directory -Force .qa | Out-Null
git archive HEAD -o .qa/ClearCe-source.zip
Expand-Archive -LiteralPath .qa/ClearCe-source.zip -DestinationPath .qa/ClearCe-public
git -C .qa/ClearCe-public init -b main
git -C .qa/ClearCe-public add .
git -C .qa/ClearCe-public commit -m "feat: introduce ClearCe Windows MVP"
Push-Location .qa/ClearCe-public
gh repo create ClearCe --public --description "ClearCe — Local AI Image Enhancer for Windows" --source . --remote origin --push
Pop-Location
```

If the public repository already exists, clone it for future public work instead of replaying these creation commands. Do not merge the original private phase history into it. No project license was selected; choose one explicitly before claiming an open-source release. Installer binaries can be attached separately only after release review; external AI engines are not included.

For Phase 4.3 and later Windows packages, use `npm run release:windows`, `node scripts/check-release.mjs`, and the [release trust checklist](WINDOWS_TRUST.md). Include the generated `SHA256SUMS.txt` and `release-manifest.json` with the exact final artifacts. The unsigned test workflow does not publish to GitHub, Vercel or any hosting service.
