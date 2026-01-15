# music-manager

TODO: readme

## Atlas Migrations

> Atlas workflows are not mutually exclusive. Many teams mix declarative and versioned techniques to benefit from both.
>
> During local development, engineers often work declaratively. They edit HCL or SQL schema files and run atlas schema apply against a local database to bring it up to the desired state. This allows quick iterations without manually writing SQL, and Atlas handles planning and execution.
>
> When the schema change is ready to be shared, the team switches to the versioned workflow. Instead of applying the change directly on shared environments, they run atlas migrate diff against the updated desired state. Atlas computes the difference between the current migration history and the new schema and writes a migration file into the migrations directory. The file is then committed to source control as part of a pull request. Standard processes such as code review, migration linting, and CI/CD pipelines apply the migration using atlas migrate apply.
>
> This hybrid flow keeps local development flexible and fast while ensuring that production changes are explicit, reviewable, and reproducible.

See `migrations.just` for usage here.

### Resources

- https://atlasgo.io/concepts/declarative-vs-versioned#combining-declarative-and-versioned-workflows
