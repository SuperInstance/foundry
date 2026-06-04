# Future Integration: SuperInstance-foundry

## Current State
A fork of Foundry (the Ethereum development framework) repurposed for SuperInstance fleet infrastructure. Provides build, test, and deployment tooling.

> **Note:** This is a fork of the Foundry project. We respect their work and repurpose the infrastructure tooling for our fleet.

## Integration Opportunities

### With fleet build infrastructure
Foundry's build system (Forge) becomes the fleet's build system. Every crate in the fleet is built, tested, and deployed through Foundry's pipeline. The same tooling that builds Ethereum smart contracts builds ternary crates.

### With room deployment
Foundry's deployment scripts become room deployment scripts: build the room's crates, compile its strategies, deploy to the target hardware (Codespace, Jetson, ESP32). One command to deploy a room.

### With CI/CD
Foundry's CI integration (test on every push, deploy on merge) becomes the fleet's CI/CD: every ternary crate is tested on push, every room is deployed on merge. The fleet's quality is enforced automatically.

## Potential in Mature Systems
SuperInstance-foundry is the fleet's build and deployment infrastructure. From code to production in one pipeline. Every crate, every room, every strategy is built, tested, and deployed through Foundry.

## Cross-Pollination Ideas
- **conservation-verify**: Verification runs through Foundry's test pipeline
- **git-agent-codespace**: Codespaces use Foundry for room setup
- **agent-template**: Template includes Foundry configuration

## Dependencies for Next Steps
- Adapt Foundry's build system for Rust/cargo workloads
- Room deployment scripts
- Fleet-wide CI integration
