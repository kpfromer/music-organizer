# Frontend development server
[working-directory: 'frontend']
frontend-dev-server:
    # bun run rspack dev --mode development --port 3001
    bun --hot src/index.ts

# Watch GraphQL codegen
[working-directory: 'frontend']
frontend-dev-codegen:
    @echo "Waiting for backend server to be ready on port 3000..."
    until nc -z 127.0.0.1 3000; do sleep 1; done
    @echo "Backend server is ready on port 3000!"
    bun run graphql-code-generator --require dotenv/config --config codegen.ts --watch

# Run dev server and codegen concurrently
[working-directory: 'frontend']
[parallel]
frontend-dev: frontend-dev-server frontend-dev-codegen

# Start production server
[working-directory: 'frontend']
frontend-start:
    NODE_ENV=production bun src/index.ts

# Build for production
[working-directory: 'frontend']
frontend-build:
    bun run rspack build --mode production

# Run GraphQL codegen once
[working-directory: 'frontend']
frontend-codegen:
    bun run graphql-code-generator --require dotenv/config --config codegen.ts

# Format code with biome
[working-directory: 'frontend']
frontend-format:
    bun run biome format --write ./

# Lint code with biome
[working-directory: 'frontend']
frontend-lint:
    bun run biome check ./

# Type check TypeScript
[working-directory: 'frontend']
frontend-check-typescript:
    bun run tsc --noEmit --project tsconfig.check.json

# Run all frontend checks (lint + typescript)
[parallel]
frontend-check: frontend-lint frontend-check-typescript

# Fix biome issues (format + check with write)
[working-directory: 'frontend']
frontend-fix:
    bun run biome format --write ./
    bun run biome check --write ./

