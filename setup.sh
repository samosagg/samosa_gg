#!/bin/sh
set -e

# Create secrets directory
mkdir -p /secrets/config

origins_yaml=""
IFS=',' read -ra ORIGINS <<< "$ALLOWED_ORIGINS"
for origin in "${ORIGINS[@]}"; do
    origins_yaml="$origins_yaml\n    - $origin"
done


# Generate the config.yaml dynamically
cat <<EOF > /secrets/config/config.yaml
aptos_base_url: ${APTOS_BASE_URL}
contract_address: ${CONTRACT_ADDRESS}
decibel_url: ${DECIBEL_URL}
terminal_url: ${TERMINAL_URL}
server_config:
  port: ${PORT}
  allowed_origins: ${origins_yaml}
jwt_config:
  secret: ${JWT_SECRET}
  expires_in: 1d
db_config: 
  url: ${DATABASE_URL}
  pool_size: 10
turnkey_config:
  organization_id: ${TURNKEY_ORG_ID}
  api_private_key: ${TURNKEY_PRIVATE_KEY}
  api_public_key: ${TURNKEY_PUBLIC_KEY}
bot_config:
  token: ${BOT_TOKEN}
  username: ${BOT_USERNAME}
admin_config:
  sponsor_private_key: ${SPONSOR_ACCOUNT_PRIVATE_KEY}
EOF

# Run the Rust binary
exec ./indexer -c /secrets/config/config.yaml