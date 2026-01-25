# Contributing to MindFry

Thank you for your interest in contributing to MindFry! This document provides guidelines for contributing.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Set up the development environment**:

   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Build the project
   cargo build

   # Run tests
   cargo test

   # Run the server
   cargo run --bin mindfry-server
   ```

## Development Workflow

### Branch Naming

- `feat/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation changes
- `refactor/description` - Code refactoring
- `test/description` - Test additions/modifications

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add QUERY_PATTERN opcode support
fix: handle TCP fragmentation in large responses
docs: update README with new API examples
test: add integration tests for bond operations
```

### Code Standards

- **Zero Warnings**: All code must compile without warnings
- **Tests Required**: New features must include tests
- **Documentation**: Public APIs must be documented
- **Formatting**: Run `cargo fmt` before committing
- **Linting**: Run `cargo clippy` and address any issues

## Pull Request Process

1. **Create a feature branch** from `main`
2. **Make your changes** with clear, atomic commits
3. **Write or update tests** as needed
4. **Update documentation** if applicable
5. **Open a Pull Request** with a clear description
6. **Address review feedback** promptly

### PR Checklist

- [ ] Code compiles without warnings (`cargo build`)
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy is clean (`cargo clippy`)
- [ ] Documentation is updated (if applicable)
- [ ] CHANGELOG.md is updated (for user-facing changes)

## Areas for Contribution

### Good First Issues

Look for issues labeled `good first issue` - these are suitable for newcomers.

### High-Impact Areas

- **Performance**: Optimizing decay calculations, graph traversal
- **Documentation**: API examples, architecture guides
- **Testing**: Integration tests, property-based tests
- **Protocol**: New MFBP opcodes, client implementations

## Questions?

- Open a [Discussion](https://github.com/cluster-127/mindfry/discussions) for questions
- Check existing [Issues](https://github.com/cluster-127/mindfry/issues) before opening a new one

## License

By contributing, you agree that your contributions will be licensed under the project's Apache-2.0 license.
