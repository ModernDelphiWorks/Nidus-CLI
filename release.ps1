# Release script for nest4d-cli
Write-Host 'Building release...' -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host 'Build successful! Preparing release...' -ForegroundColor Green
    git add .
    
    # Extract version from Cargo.toml
    $cargoContent = Get-Content 'Cargo.toml' -Raw
    $versionMatch = [regex]::Match($cargoContent, 'version\s*=\s*"([^"]+)"')
    if ($versionMatch.Success) {
        $version = $versionMatch.Groups[1].Value
        Write-Host "Detected version: v$version" -ForegroundColor Cyan
    } else {
        Write-Host 'Could not detect version from Cargo.toml, please enter manually:' -ForegroundColor Yellow
        $version = Read-Host 'Enter version number'
    }
    
    git commit -m "Release v$version"
    git tag "v$version"
    
    Write-Host 'Pushing to https://github.com/ModernDelphiWorks/nest4d-cli...' -ForegroundColor Cyan
    git push origin main --tags
    
    Write-Host "Release v$version published successfully!" -ForegroundColor Green
} else {
    Write-Host 'Build failed, aborting release' -ForegroundColor Red
}