#!/usr/bin/env bash
# App-specific installer and package assertions retained alongside shared packaging lint.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

for want in '<UIRef Id="NovalightUI" />' 'Value="InstallDirDlg"' '<Property Id="WIXUI_INSTALLDIR" Value="INSTALLFOLDER" />' \
    'Key="Software\NovaLight" Name="InstallFolder"' 'Value="LaunchNovalight"'; do
  grep -qF "$want" packaging/windows/novalight.wxs || { echo "::error::novalight.wxs lost: $want"; exit 1; }
done
grep -qF '<DialogRef Id="ExitDialog" />' packaging/windows/novalight.wxs \
  || { echo "::error::ExitDialog is not referenced; installs and removes end without a completion dialog"; exit 1; }

python3 - <<'PYEOF'
import yaml
manifests = []
for name in ("br.com.novafoods.novalight.yml", "br.com.novafoods.novalight.bundle.yml"):
    with open("packaging/linux/flatpak/" + name) as source:
        manifest = yaml.safe_load(source)
    assert manifest["id"] == "br.com.novafoods.novalight" and manifest["command"] == "novalight", name
    manifests.append(manifest)
for key in ("runtime", "runtime-version", "sdk", "finish-args"):
    assert manifests[0][key] == manifests[1][key], f"Flatpak manifests disagree on {key}"
print("NovaLight package identity ok")
PYEOF
