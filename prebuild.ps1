$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$verFile = Join-Path $root "version.txt"
$numFile = Join-Path $root "build_number.txt"
$infoFile = Join-Path $root "source\public\build_info.json"

$ver = if (Test-Path $verFile) { (Get-Content $verFile -Raw).Trim() } else { "0.0.0" }
$n = if (Test-Path $numFile) { [int]((Get-Content $numFile -Raw).Trim()) } else { 0 }
$n += 1

Set-Content -Path $numFile -Value $n -NoNewline

$json = '{"version":"' + $ver + '","buildNumber":' + $n + '}'
Set-Content -Path $infoFile -Value $json -NoNewline

Write-Host "$ver.$n"
