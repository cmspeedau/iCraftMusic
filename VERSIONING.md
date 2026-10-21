# Documentation versioning

This repository versions its documentation with semantic versioning:

- **MAJOR** — incompatible restructuring or a substantial change in the documented contract, scope, or interpretation.
- **MINOR** — backward-compatible additions, such as a new stage, section, or clarified plan that does not invalidate existing guidance.
- **PATCH** — backward-compatible corrections, wording fixes, link repairs, and formatting changes.

## Bumping a version

1. Update the version and date in the document's version header.
2. Add a dated entry to `CHANGELOG.md` using Keep a Changelog sections such as Added, Changed, and Fixed.
3. Review the table of contents, internal links, status wording, and list-item preservation.
4. Commit the documentation changes with a descriptive message.
5. Create an annotated Git tag using the `vMAJOR.MINOR.PATCH` convention (for example, `v1.0.0`), then verify the tag points to the intended commit.

Roadmap feature versions (for example, `v0.10`) identify product increments and are separate from the roadmap document version. The document version and repository tag should agree: this release is document version `1.4.0` with Git tag `v1.4.0`. A release tag is immutable: corrections after a release receive a new version rather than rewriting the old tag.
