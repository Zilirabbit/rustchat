# RustChat Documentation

This directory contains product, architecture, API, setup, design, and progress documents for RustChat.

## Product

- [Overview](product/overview.md)
- [Product spec](product/spec.md)
- [Task list](product/task.md)
- [Requirements analysis](product/requirements-analysis.md)
- [Phase timeline](product/phase-timeline.md)
- [Phase breakdown](product/phase-breakdown.md)
- [Git commit history](product/git-commit-history.md)
- [Frontend UI plan](product/frontend-ui-plan.md)

## Architecture

- [Project structure](architecture/project-structure.md)
- [Database design](architecture/db.md)
- [Technical architecture diagram](architecture/technical-architecture.md)

## API

- [HTTP and WebSocket API](api/http-api.md)
- [Postman collections](api/postman/README.md)

## Setup

- [VM environment setup](setup/env-setup-vm.md)

## Design

- [Design notes](design/README.md)
- [Design prompt](design/PROMPT.md)
- [Design draft](design/DESIGN.md)
- [Reference images](img/)

## Progress And Issues

- [Progress summaries](summary/README.md)
- [Known problems](problems/git_push_large_file_issue.md)

## Generated Artifacts

The following local directories are generated or runtime artifacts. They are kept out of Git by `.gitignore` and can be recreated when needed:

- `../backend/target/`
- `../frontend/node_modules/`
- `../frontend/dist/`
- `../backend/uploads/`
