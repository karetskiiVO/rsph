#Requires -Version 7.5
<#
.SYNOPSIS
    Configures D highlighting in the default VS Code user profile on Windows x64.
.DESCRIPTION
    Installs the verified dmd-lsp build and merges a D-specific palette into settings.json.
    A backup retains the original JSONC, including comments, when settings change.
#>

$ErrorActionPreference = 'Stop'

if (-not $IsWindows -or [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne 'X64') {
    throw 'This script requires Windows x64.'
}

$code = Get-Command code -ErrorAction SilentlyContinue
if (-not $code) {
    throw 'Install Visual Studio Code and make its code command available on PATH.'
}

$userDir = Join-Path $env:APPDATA 'Code\User'
$settingsPath = Join-Path $userDir 'settings.json'
$serverPath = Join-Path $userDir 'globalStorage\xoxorwr.dmd-lsp\bin\dmd-lsp.exe'

$original = if (Test-Path -LiteralPath $settingsPath) { [IO.File]::ReadAllText($settingsPath) }
$settings = if ([string]::IsNullOrWhiteSpace($original)) {
    [ordered]@{}
} else {
    ConvertFrom-Json -InputObject $original -AsHashtable -NoEnumerate -DateKind String
}
if ($settings -isnot [System.Collections.IDictionary]) {
    throw 'VS Code settings.json must contain a JSON object.'
}

function Get-SettingsObject($parent, [string]$key) {
    if (-not $parent.Contains($key)) { $parent[$key] = [ordered]@{} }
    if ($parent[$key] -isnot [System.Collections.IDictionary]) {
        throw "Expected a JSON object in VS Code settings for '$key'."
    }
    $parent[$key]
}

$semanticRules = [ordered]@{
    'type:d'          = '#4EC9B0'
    'class:d'         = '#4EC9B0'
    'struct:d'        = '#4EC9B0'
    'interface:d'     = '#4EC9B0'
    'enum:d'          = '#4EC9B0'
    'typeParameter:d' = '#4EC9B0'
    'function:d'      = '#DCDCAA'
    'method:d'        = '#DCDCAA'
    'parameter:d'     = '#808080'
}
$textMateRules = @(
    [ordered]@{ scope = @('source.d entity.name.function', 'source.d support.function'); settings = @{ foreground = '#DCDCAA' } }
    [ordered]@{ scope = @('source.d entity.name.type', 'source.d entity.name.class', 'source.d support.type.built-ins'); settings = @{ foreground = '#4EC9B0' } }
    [ordered]@{ scope = @('storage.type.basic-type.d', 'storage.type.d', 'support.type.void.d'); settings = @{ foreground = '#569CD6' } }
)

$language = Get-SettingsObject $settings '[d]'
$language['editor.semanticHighlighting.enabled'] = $true

$semantic = Get-SettingsObject $settings 'editor.semanticTokenColorCustomizations'
$semanticGroups = @($semantic) + @($semantic.Keys | Where-Object { $_ -like '[[]*' } | ForEach-Object { $semantic[$_] })
foreach ($group in $semanticGroups) {
    $rules = Get-SettingsObject $group 'rules'
    foreach ($key in $semanticRules.Keys) { $rules[$key] = $semanticRules[$key] }
}

$textMate = Get-SettingsObject $settings 'editor.tokenColorCustomizations'
$textMateGroups = @($textMate) + @($textMate.Keys | Where-Object { $_ -like '[[]*' } | ForEach-Object { $textMate[$_] })
foreach ($group in $textMateGroups) {
    $rules = @($group.textMateRules | Where-Object { $null -ne $_ })
    foreach ($rule in $textMateRules) {
        $scope = $rule.scope -join ', '
        $rules = @($rules | Where-Object { ($_.scope -join ', ') -ne $scope }) + @($rule)
    }
    $group['textMateRules'] = $rules
}

# dmd-lsp can classify template predicates as types. An editor decoration
# gives isXxx! its own color while leaving the semantic type rule in place.
$highlightRules = Get-SettingsObject $settings 'highlight.regexes'
$predicatePattern = '//[^\r\n]*|/\*[\s\S]*?\*/|/\+[\s\S]*?\+/|\br"[^"]*"|"(?:\\[\s\S]|[^"\\])*"|''(?:\\[\s\S]|[^''\\])*''|`[^`]*`|\b(is[A-Z][A-Za-z0-9_]*)(?=\s*!(?!=|(?:is|in)\b))'
$highlightRules[$predicatePattern] = [ordered]@{
    filterLanguageRegex = '^d$'
    regexFlags          = 'g'
    decorations         = @(@{ color = '#C586C0' })
}

$settings['dmdLsp.autoUpdate'] = $false
$settings['dmdLsp.autoCloseBrackets'] = $false
$settings['dmdLsp.serverPath'] = $serverPath
$updated = ($settings | ConvertTo-Json -Depth 100 -WarningAction Stop) + [Environment]::NewLine

$extension = 'xoxorwr.dmd-lsp@0.0.0'
$installed = & $code --list-extensions --show-versions
if ($LASTEXITCODE -ne 0) { throw 'Could not list VS Code extensions.' }
$installExtension = $extension -notin $installed
$highlightExtension = 'fabiospampinato.vscode-highlight'
$installHighlight = $highlightExtension -notin ($installed -replace '@.*$', '')

# The nightly tag rolls forward. These hashes identify the Windows build tested with this palette.
$releaseUrl = 'https://github.com/xoxorwr/dmd-lsp/releases/download/nightly'
$serverHash = 'DE7F535504B16B629321F9394F68BB8B000F74542E3526650350CAF6E32C268D'
$installServer = -not (Test-Path -LiteralPath $serverPath -PathType Leaf) -or
    (Get-FileHash -LiteralPath $serverPath).Hash -ne $serverHash

function Get-ReleaseAsset([string]$name, [string]$hash) {
    $path = Join-Path $staging.FullName $name
    Invoke-WebRequest -Uri "$releaseUrl/$name" -OutFile $path -TimeoutSec 60
    if ((Get-FileHash -LiteralPath $path).Hash -ne $hash) {
        throw "SHA-256 mismatch for $name. The upstream nightly release may have changed."
    }
    $path
}

function Save-Settings {
    if ($updated -ceq $original) { return }

    $null = New-Item -ItemType Directory -Path $userDir -Force
    $suffix = [guid]::NewGuid().ToString('N')
    $pending = "$settingsPath.$suffix.tmp"
    try {
        [IO.File]::WriteAllText($pending, $updated)
        if ($null -ne $original) {
            if ([IO.File]::ReadAllText($settingsPath) -cne $original) { throw 'VS Code settings changed. Run the script again.' }
            $backup = "$settingsPath.$suffix.bak"
            [IO.File]::Replace($pending, $settingsPath, $backup)
            Write-Host "Settings backup: $backup"
        } else {
            [IO.File]::Move($pending, $settingsPath)
        }
    } finally {
        if (Test-Path -LiteralPath $pending) { Remove-Item -LiteralPath $pending -Force }
    }
}

$staging = $null
try {
    if ($installServer -or $installExtension) {
        $staging = New-Item -ItemType Directory -Path (Join-Path ([IO.Path]::GetTempPath()) ('d-highlighting-' + [guid]::NewGuid()))
        if ($installExtension) {
            $vsix = Get-ReleaseAsset 'dmd-lsp.vsix' 'E539C7BE5ADA3336BD4AEBB0BC04D572CF8E2DBD10A0F57C0F1513AC15C12DF0'
        }
        if ($installServer) {
            $zip = Get-ReleaseAsset 'dmd-lsp-win-x64.zip' '102B14A8326DE132B9733C5B0E5A7AC4D793C9A288437BBD800BC7C492E6EA9F'
            Expand-Archive -LiteralPath $zip -DestinationPath $staging.FullName
            $exe = Join-Path $staging.FullName 'dmd-lsp.exe'
            if ((Get-FileHash -LiteralPath $exe).Hash -ne $serverHash) { throw 'Unexpected dmd-lsp.exe hash.' }
            $null = New-Item -ItemType Directory -Path (Split-Path $serverPath) -Force
            Copy-Item -LiteralPath $exe -Destination $serverPath
        }

    }

    Save-Settings
    if ($installExtension) {
        & $code --install-extension $vsix --do-not-sync
        if ($LASTEXITCODE -ne 0) { throw 'Could not install dmd-lsp in VS Code.' }
    }
    if ($installHighlight) {
        & $code --install-extension $highlightExtension --do-not-sync
        if ($LASTEXITCODE -ne 0) { throw 'Could not install Highlight in VS Code.' }
    }
} finally {
    if ($staging) {
        $stagingPath = $staging.FullName
        if ((Split-Path $stagingPath) -eq (Get-Item -LiteralPath ([IO.Path]::GetTempPath())).FullName -and
            (Split-Path $stagingPath -Leaf) -match '^d-highlighting-[0-9a-f-]{36}$') {
            Remove-Item -LiteralPath $stagingPath -Recurse -Force
        }
    }
}

Write-Host 'D highlighting configured. Reload the VS Code window if it is open.'
