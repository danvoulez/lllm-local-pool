# Contributing to LLM Pool

Thank you for your interest in contributing to LLM Pool! This document provides guidelines and instructions for contributing.

## 🚀 Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/llm-pool.git
   cd llm-pool
   ```
3. **Set up the development environment**:
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install Ollama (macOS)
   brew install ollama
   
   # Download models
   ./setup-models.sh
   ```

## 🔧 Development Workflow

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Check code without building
cargo check
```

### Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test '*'

# Test HTTP API
./test-http.sh
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check for unused dependencies
cargo machete
```

## 📝 Coding Standards

### Rust Style
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for consistent formatting
- Run `cargo clippy` and fix all warnings
- Write documentation for public APIs

### Commit Messages
Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add support for Claude provider
fix: resolve cache key collision issue
docs: update README with new examples
refactor: simplify ensemble strategy logic
test: add unit tests for orchestrator
```

### Branch Naming
- `feat/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation updates
- `refactor/description` - Code refactoring
- `test/description` - Test additions/updates

## 🎯 What to Contribute

### High Priority
- [ ] Additional LLM providers (OpenAI, Anthropic, Cohere)
- [ ] Advanced ensemble strategies (weighted voting, judge implementation)
- [ ] Redis cache backend
- [ ] Prometheus metrics implementation
- [ ] OpenTelemetry tracing
- [ ] Rate limiting and authentication
- [ ] Client SDKs (Python, TypeScript, Go)

### Medium Priority
- [ ] Circuit breaker implementation
- [ ] Hedged request support
- [ ] Advanced prompt templates
- [ ] Performance benchmarks
- [ ] Docker/Kubernetes deployment
- [ ] Monitoring dashboards

### Documentation
- [ ] API reference documentation
- [ ] Architecture deep-dive
- [ ] Performance tuning guide
- [ ] Deployment best practices
- [ ] Video tutorials

## 🐛 Reporting Bugs

When reporting bugs, please include:

1. **Environment**:
   - OS and version
   - Rust version (`rustc --version`)
   - LLM Pool version
   - Provider details (Ollama version, models)

2. **Steps to reproduce**:
   - Clear, numbered steps
   - Sample code or curl commands
   - Configuration files (sanitized)

3. **Expected vs Actual behavior**:
   - What you expected to happen
   - What actually happened
   - Error messages and logs

4. **Additional context**:
   - Screenshots if applicable
   - Related issues or PRs

## 💡 Suggesting Features

Feature requests should include:

1. **Use case**: Why is this needed?
2. **Proposed solution**: How should it work?
3. **Alternatives considered**: What other approaches did you think about?
4. **Additional context**: Examples, mockups, or references

## 🔀 Pull Request Process

1. **Create a feature branch**:
   ```bash
   git checkout -b feat/your-feature-name
   ```

2. **Make your changes**:
   - Write clean, documented code
   - Add tests for new functionality
   - Update documentation as needed

3. **Test thoroughly**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ./test-http.sh
   ```

4. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

5. **Push to your fork**:
   ```bash
   git push origin feat/your-feature-name
   ```

6. **Open a Pull Request**:
   - Use a clear, descriptive title
   - Reference any related issues
   - Describe what changed and why
   - Include screenshots/examples if applicable

### PR Checklist

Before submitting, ensure:

- [ ] Code builds without errors (`cargo build`)
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (if applicable)
- [ ] Commit messages follow conventions

## 🏗️ Project Structure

```
llm-pool/
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs            # Configuration management
│   ├── orchestrator.rs      # Request orchestration
│   ├── ensemble.rs          # Ensemble strategies
│   ├── cache.rs             # Caching layer
│   ├── providers/           # LLM provider implementations
│   ├── server/              # gRPC and HTTP servers
│   ├── qos/                 # Quality of Service features
│   ├── security/            # Auth and rate limiting
│   └── telemetry.rs         # Logging and metrics
├── proto/                   # Protocol buffer definitions
├── prompts/                 # Prompt templates
├── docs/                    # Documentation
└── tests/                   # Integration tests
```

## 🧪 Adding a New Provider

To add support for a new LLM provider:

1. **Create provider file**: `src/providers/your_provider.rs`
2. **Implement the Provider trait**:
   ```rust
   use async_trait::async_trait;
   use crate::providers::{Provider, ProviderResponse};
   
   pub struct YourProvider {
       // provider-specific fields
   }
   
   #[async_trait]
   impl Provider for YourProvider {
       fn name(&self) -> &str { "your-provider" }
       fn supports(&self, task: &str) -> bool { true }
       async fn infer(&self, prompt: &str, max_tokens: i32, deadline_ms: i32) 
           -> Result<ProviderResponse> {
           // implementation
       }
       async fn health(&self) -> bool { true }
   }
   ```
3. **Register in providers/mod.rs**
4. **Add configuration** in `llm-pool.toml`
5. **Write tests**
6. **Update documentation**

## 📚 Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Tonic gRPC Guide](https://github.com/hyperium/tonic)
- [Axum Documentation](https://docs.rs/axum/)
- [Project README](README.md)
- [Architecture Guide](docs/ARCHITECTURE.md)

## 🤝 Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of:
- Age, body size, disability, ethnicity, gender identity and expression
- Level of experience, education, socio-economic status
- Nationality, personal appearance, race, religion
- Sexual identity and orientation

### Our Standards

**Positive behavior includes:**
- Using welcoming and inclusive language
- Being respectful of differing viewpoints
- Gracefully accepting constructive criticism
- Focusing on what's best for the community
- Showing empathy towards others

**Unacceptable behavior includes:**
- Harassment, trolling, or derogatory comments
- Public or private harassment
- Publishing others' private information
- Other conduct inappropriate in a professional setting

### Enforcement

Violations may result in temporary or permanent ban from the project. Report issues to the maintainers.

## 📞 Getting Help

- **Documentation**: Check [README.md](README.md) and [docs/](docs/)
- **Issues**: Search existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Discord**: Join our community server (link in README)

## 🎉 Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Credited in documentation

Thank you for contributing to LLM Pool! 🚀
