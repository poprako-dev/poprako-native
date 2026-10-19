$ErrorActionPreference = 'Stop'
$nodeVersion = (Get-Content .node-version -Raw).Trim()
$pnpmVersion = (Get-Content package.json -Raw | ConvertFrom-Json).packageManager.Split('@')[1]
$installation = Join-Path $env:RUNNER_TEMP 'native-js'
if (Test-Path $installation) { throw 'Use a fresh installation directory.' }
$nodeRoot = Join-Path $installation 'node'
$pnpmRoot = Join-Path $installation 'pnpm'
New-Item -ItemType Directory -Path $nodeRoot, $pnpmRoot | Out-Null
$base = "https://nodejs.org/dist/v$nodeVersion"
Invoke-WebRequest "$base/win-x64/node.exe" -OutFile "$nodeRoot/node.exe"
$checksums = (Invoke-WebRequest "$base/SHASUMS256.txt").Content
$expected = (($checksums -split "`n" | Where-Object { $_ -match ' win-x64/node.exe$' }) -split '\s+')[0]
if (!$expected -or (Get-FileHash "$nodeRoot/node.exe" -Algorithm SHA256).Hash.ToLower() -ne $expected) { throw 'Node checksum mismatch.' }
$archive = Join-Path $installation 'pnpm.zip'
Invoke-WebRequest "https://github.com/pnpm/pnpm/releases/download/v$pnpmVersion/pnpm-win32-x64.zip" -OutFile $archive
$expectedPnpm = '66cac6b725bfc75fadac959fc435821048558adb50b89c82d0d2d5e57465a1dd'
if ((Get-FileHash $archive -Algorithm SHA256).Hash.ToLower() -ne $expectedPnpm) { throw 'pnpm checksum mismatch.' }
Expand-Archive $archive -DestinationPath $pnpmRoot
$env:PATH = "$nodeRoot;$pnpmRoot;$env:PATH"
if ((node --version) -ne "v$nodeVersion" -or (pnpm --version) -ne $pnpmVersion) { throw 'Unexpected toolchain version.' }
$nodeRoot, $pnpmRoot | Out-File -FilePath $env:GITHUB_PATH -Append -Encoding utf8
