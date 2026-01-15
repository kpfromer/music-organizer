sqlite_database := "sqlite://music/library.db"
migrations_dir := "file://migrations"
schema_file := "file://schema.sql"

# Uses Atlas to sync the database schema with the schema.hcl file for development purposes.
dev-migrate-sync:
    atlas schema apply --url "{{sqlite_database}}" --to "{{schema_file}}" --dev-url "sqlite://dev?mode=memory"

# Creates a new migration file with the given name using Atlas.
create-migration name:
    atlas migrate diff "{{name}}" --dir "{{migrations_dir}}" --to "{{schema_file}}" --dev-url "sqlite://dev?mode=memory"

# Checks for differences between the migrations directory and the schema file.
diff-migration:
    atlas schema diff  --from "{{migrations_dir}}" --to "{{schema_file}}" --dev-url "sqlite://dev?mode=memory"

# Checks for differences between the migrations directory and the schema file and errors if there are any.
check-migration-diff:
    #!/usr/bin/env -S uv run --script

    import os
    import subprocess
    import sys

    cmd = [
        "atlas",
        "schema",
        "diff",
        "--from", "file://migrations",
        "--to", "file://schema.sql",
        "--dev-url", "sqlite://dev?mode=memory",
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)
    output = result.stdout + result.stderr

    github_output = os.environ.get("GITHUB_OUTPUT")
    if github_output:
        with open(github_output, "a") as f:
            f.write("diff_output<<EOF\n")
            f.write(output)
            f.write("\nEOF\n")

    if output.strip() != "Schemas are synced, no changes to be made.":
        print("❌ Schema differences detected:")
        print(output)
        sys.exit(1)
    else:
        print("✅ Schemas are synced, no changes to be made.")