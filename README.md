# PACE

## Introduction
Telegram bot & terminal suite

## Diesel setup
```sh
deisel setup
```

# Generate migration
```sh
diesel migration generate <migration_name> --migration-dir src/db_migrations/migrations 
diesel migration generate update_tokens_table --migration-dir src/db_migrations/migrations 
```

# Run migration
```sh
diesel migration run \
  --config-file="src/db_migrations/diesel.toml" \
  --database-url="postgresql://ajaythxkur@localhost:5432/samosa_gg"
```

# Revert last migration
```sh
diesel migration revert \
  --config-file="src/db_migrations/diesel.toml" \
  --database-url="postgresql://ajaythxkur@localhost:5432/samosa_gg"
```

