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
    PUBLIC_GRAPHQL_URL=http://localhost:3000/graphql bun run rspack build --mode production

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

generate-graphql-types:
    #!/usr/bin/env -S uv run --script
    #
    # /// script
    # requires-python = ">=3.12"
    # dependencies = ["requests"]
    # ///
    import subprocess
    import sys
    import time
    import os

    import requests

    exit_code = 1 # default to failure

    # 1. Start backend server in background
    print("Starting backend server...")
    backend_process = subprocess.Popen(
        ["cargo", "run", "--", "serve", "--log-level", "info", "--directory", "downloads"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    try:
        # 2. Wait for http://localhost:3000 to return HTTP 200
        url = "http://localhost:3000/graphql"
        timeout = 60
        start_time = time.time()
        while time.time() - start_time < timeout:
            try:
                response = requests.get(url, timeout=2)
            except Exception:
                response = None
            
            if response and response.status_code == 200:
                print(f"{url} is up!")
                break
            else:
                print(f"Waiting for {url} to respond with 200...")
            time.sleep(1)
        else:
            print(f"Timeout: {url} did not become ready within {timeout} seconds")
            backend_process.terminate()
            backend_process.wait(timeout=5)
            sys.exit(1)

        # 3. Run GraphQL code generator
        print("Running GraphQL code generator...")
        os.chdir("frontend")
        result = subprocess.run([
            "bun", "run", "graphql-code-generator",
            "--require", "dotenv/config",
            "--config", "codegen.ts"
        ])
        exit_code = result.returncode

    finally:
        # 4. Bring down backend process if still running
        if backend_process.poll() is None:
            print("Stopping backend server...")
            backend_process.terminate()
            try:
                backend_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                print("Force killing backend server...")
                backend_process.kill()
                backend_process.wait()

    sys.exit(exit_code)

