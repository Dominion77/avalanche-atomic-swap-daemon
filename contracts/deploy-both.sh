#!/bin/bash

echo ""
echo "🚀 Starting deployment to both Fuji and Echo testnets..."
echo ""

# Deploy to Fuji
echo "============================================================"
echo "DEPLOYING TO FUJI C-CHAIN..."
echo "============================================================"
npm run deploy:fuji
FUJI_EXIT=$?

echo ""

# Deploy to Echo
echo "============================================================"
echo "DEPLOYING TO ECHO SUBNET..."
echo "============================================================"
npm run deploy:echo
ECHO_EXIT=$?

echo ""
echo "============================================================"
echo "DEPLOYMENT SUMMARY"
echo "============================================================"

if [ $FUJI_EXIT -eq 0 ]; then
  echo "✅ Fuji C-Chain: Deployment successful"
else
  echo "❌ Fuji C-Chain: Deployment failed"
fi

if [ $ECHO_EXIT -eq 0 ]; then
  echo "✅ Echo Subnet: Deployment successful"
else
  echo "❌ Echo Subnet: Deployment failed"
fi

echo ""
echo "Check the output above for contract addresses."
echo "Add them to your daemon .env file as HTLC_CCHAIN and HTLC_SUBNET"
echo ""
