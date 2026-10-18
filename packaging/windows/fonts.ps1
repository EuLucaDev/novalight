# Font setup for local packaging; dot-sourced by package.ps1.
function Initialize-CraftFonts([string] $WorkspaceRoot) {
  $fontDir = $env:NOVAFOODS_FONTS_DIR
  if (-not $fontDir) { $fontDir = Join-Path $WorkspaceRoot '../novafoods-fonts' }
  if (-not [IO.Path]::IsPathRooted($fontDir)) { $fontDir = Join-Path $WorkspaceRoot $fontDir }

  if (-not $env:NOVAFOODS_FONTS_DIR -and -not (Test-Path -LiteralPath $fontDir)) {
    # Same revision as release.yml and packaging/fonts.sh. Existing checkouts stay untouched.
    $revision = '3d9ab3a132ed0eff457e424226a435a901da30d3'
    Write-Output "==> downloading novafoods-fonts ($revision) to $fontDir"
    & git clone --no-checkout https://github.com/EuLucaDev/novafoods-fonts $fontDir
    if ($LASTEXITCODE -ne 0) { throw 'Could not download novafoods-fonts; provide a complete checkout with NOVAFOODS_FONTS_DIR' }
    & git -C $fontDir checkout --detach $revision
    if ($LASTEXITCODE -ne 0) { throw 'Could not check out novafoods-fonts; provide a complete checkout with NOVAFOODS_FONTS_DIR' }
  }

  $manifest = Join-Path $fontDir 'fonts/manifest.txt'
  if (-not (Test-Path -LiteralPath $manifest -PathType Leaf)) {
    throw "Missing $manifest; set NOVAFOODS_FONTS_DIR to a complete novafoods-fonts checkout"
  }
  $env:NOVAFOODS_FONTS_DIR = (Resolve-Path -LiteralPath $fontDir).Path
  $env:NOVAFOODS_FONTS_REQUIRED = '1'
  Write-Output "==> embedding fonts from $env:NOVAFOODS_FONTS_DIR"
}
