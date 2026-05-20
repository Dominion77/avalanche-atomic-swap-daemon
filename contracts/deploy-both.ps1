Write-Host ""
Write-Host "🚀 Starting deployment to both Fuji and Echo testnets..." -ForegroundColor Cyan
Write-Host ""

# Deploy to Fuji
Write-Host "============================================================" -ForegroundColor Yellow
Write-Host "DEPLOYING TO FUJI C-CHAIN..." -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Yellow
npm run deploy:fuji
$FujiExit = $LASTEXITCODE

Write-Host ""

# Deploy to Echo
Write-Host "============================================================" -ForegroundColor Yellow
Write-Host "DEPLOYING TO ECHO SUBNET..." -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Yellow
npm run deploy:echo
$EchoExit = $LASTEXITCODE

Write-Host ""
Write-Host "============================================================" -ForegroundColor Green
Write-Host "DEPLOYMENT SUMMARY" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green

if ($FujiExit -eq 0) {
    Write-Host "✅ Fuji C-Chain: Deployment successful" -ForegroundColor Green
} else {
    Write-Host "❌ Fuji C-Chain: Deployment failed" -ForegroundColor Red
}

if ($EchoExit -eq 0) {
    Write-Host "✅ Echo Subnet: Deployment successful" -ForegroundColor Green
} else {
    Write-Host "❌ Echo Subnet: Deployment failed" -ForegroundColor Red
}

Write-Host ""
Write-Host "Check the output above for contract addresses." -ForegroundColor Cyan
Write-Host "Add them to your daemon .env file as HTLC_CCHAIN and HTLC_SUBNET" -ForegroundColor Cyan
Write-Host ""
