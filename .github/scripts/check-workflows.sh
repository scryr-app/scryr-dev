#!/usr/bin/env bash
set -euo pipefail
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
curl --fail --silent --show-error --location \
  https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_linux_amd64.tar.gz \
  --output "$scratch/actionlint.tar.gz"
printf '%s  %s\n' 8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8 "$scratch/actionlint.tar.gz" | sha256sum --check
tar -xzf "$scratch/actionlint.tar.gz" -C "$scratch" actionlint
"$scratch/actionlint"
curl --fail --silent --show-error --location \
  https://github.com/gitleaks/gitleaks/releases/download/v8.30.1/gitleaks_8.30.1_linux_x64.tar.gz \
  --output "$scratch/gitleaks.tar.gz"
printf '%s  %s\n' 551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb "$scratch/gitleaks.tar.gz" | sha256sum --check
tar -xzf "$scratch/gitleaks.tar.gz" -C "$scratch" gitleaks
"$scratch/gitleaks" git --redact --no-banner --log-opts=--all .
