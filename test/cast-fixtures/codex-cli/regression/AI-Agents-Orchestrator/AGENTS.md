# AGENTS.md — Instructions for AI Coding Agents

This file is read by Codex, Gemini CLI, and other agentic coding tools. Claude Code reads `.claude/CLAUDE.md` which imports this file.

## Project Overview
AI Coding Tools Orchestrator: Enterprise-grade agentic infrastructure with two self-contained systems coordinating AI coding assistants (Claude, Codex, Gemini, Copilot, Ollama, llama.cpp), plus specialized skills, MCP tools, and persistent graph-based context memory.

## Build & Test
```bash
pip install -r requirements.txt
python -m pytest tests/ --override-ini="addopts=" -q --timeout=30 -m "not integration and not slow"
```

## Code Style
- Python 3.8+, type hints required
- Black formatting, 120-char max line length
- isort for import ordering
- flake8 for linting

## Architecture Boundaries
- `orchestrator/` and `agentic_team/` are fully independent — zero shared imports
- Each has its own adapters/, config/, ui/, CLI
- `mcp_server/` is optional and depends on both
- `orchestrator/context/` provides graph-based memory (shared infrastructure)

## Testing
- All tests in `tests/` directory
- Use `@pytest.mark.integration` for tests requiring CLI tools (claude, codex, gemini)
- Use `@pytest.mark.slow` for long-running tests
- CI excludes integration and slow tests

## File Patterns
- Adapters: `*/adapters/<name>_adapter.py` extending `BaseAdapter`
- Config: `*/config/agents.yaml` (YAML, not JSON)
- Tests: `tests/test_<module>.py`
- Skills: `.claude/skills/<category>/<skill>.md`
- Claude Agents: `.claude/agents/<role>.md`
- Codex Agents: `.codex/agents/<role>.toml`

## Specialized Agents

### Available Specialized Agents
Use specialized agents for domain-specific tasks:

| Agent | Domain | Use For |
|-------|--------|---------|
| web-frontend | Frontend | React, Vue, CSS, accessibility |
| backend-api | Backend | REST, GraphQL, databases, Flask/FastAPI |
| security-specialist | Security | OWASP, vulnerability analysis, secure coding |
| devops-infrastructure | DevOps | Docker, K8s, CI/CD, cloud |
| ai-ml-engineer | AI/ML | ML pipelines, embeddings, LLM integration |
| database-architect | Data | Schema design, query optimization |
| mobile-developer | Mobile | iOS, Android, React Native, Flutter |
| performance-engineer | Performance | Profiling, load testing, optimization |
| documentation-writer | Docs | API docs, architecture, tutorials |

### Invoking Agents (Claude)
Reference agent by filename without extension:
```
@web-frontend Review this component for accessibility
@security-specialist Audit this authentication code
```

## Skills Library

Skills provide reusable patterns and best practices. Located in `.claude/skills/`:

### Categories
- **development/**: React components, REST API design, Python async, database queries, GraphQL, error handling
- **testing/**: Unit testing, integration testing, TDD, performance testing
- **security/**: Input validation, authentication, secure coding, vulnerability assessment
- **devops/**: Docker containerization, CI/CD pipelines, Kubernetes deployment
- **ai-ml/**: Embeddings & retrieval, LLM integration, RAG pipelines
- **documentation/**: API documentation, architecture docs, code documentation

### Using Skills
Reference skills for guidance on specific patterns:
```
See .claude/skills/security/input-validation.md for validation patterns
See .claude/skills/testing/unit-testing.md for pytest patterns
```

## MCP Tools

### Core Tools (10)
- `orchestrator_execute` - Execute task via orchestrator
- `agentic_team_execute` - Execute via agentic team
- `orchestrator_list_agents` - List available agents
- `orchestrator_health` - Check orchestrator health
- `agentic_team_config` - Get team configuration
- `list_engines` - List all engines

### Code Analysis Tools (4)
- `code_complexity` - Analyze Python code complexity
- `find_patterns` - Find code patterns/anti-patterns
- `analyze_deps` - Analyze dependency issues
- `code_summary` - Generate code file summary

### Security Tools (4)
- `secrets_scan` - Scan for hardcoded secrets
- `security_headers_check` - Check security headers
- `dependency_audit` - Audit dependencies for CVEs
- `injection_scan` - Scan for injection vulnerabilities

### Testing Tools (4)
- `suggest_tests` - Generate test case suggestions
- `test_coverage_analysis` - Analyze test coverage
- `parse_tests` - Parse test output results
- `create_test_stub` - Generate test stubs

### DevOps Tools (5)
- `dockerfile_analysis` - Analyze Dockerfile best practices
- `compose_analysis` - Analyze Docker Compose
- `ci_config_check` - Check CI/CD configuration
- `deploy_checklist` - Generate deployment checklist
- `env_config_analysis` - Analyze environment config

### Context Tools (7)
- `context_store_conversation` - Store conversation in memory
- `context_store_task` - Store task result
- `context_log_mistake` - Log mistake for future reference
- `context_store_pattern` - Store reusable code pattern
- `context_search` - Search context memory
- `context_get_relevant` - Get relevant context for task
- `context_stats` - Get memory statistics

## Graph Context System

The context system provides persistent memory for AI agents:

### Node Types
- **Conversation**: Past chat sessions with messages
- **Task**: Completed tasks with outcomes
- **Mistake**: Errors with corrections and prevention strategies
- **Pattern**: Reusable code patterns
- **Decision**: Architectural decisions with rationale
- **CodeSnippet**: Useful code fragments
- **Preference**: Learned user preferences
- **File**: Source file metadata from project scans
- **Concept**: Abstract concepts and domain knowledge
- **Project**: Registered project roots with scan metadata

### Project-Scoped Context

Both systems support project-scoped graphs. Configure `PROJECT_PATH` env var or `settings.project_path` in config YAML to point at a user project. The engine will:

1. Scan the directory with `ProjectScanner` (languages, frameworks, structure)
2. Create a `PROJECT` node plus `FILE`, `PATTERN`, and `DECISION` nodes
3. Tag all nodes with a deterministic `project_id` (SHA-256 prefix of path)
4. Isolate queries by `project_id` — no cross-project contamination

```python
from orchestrator.context import MemoryManager

manager = MemoryManager()

# Register a project (idempotent)
pid = manager.register_project("/path/to/project")

# Get project-scoped context for a task
context = manager.get_project_context(pid, task="Add authentication")

# Rescan after major changes
manager.rescan_project("/path/to/project")
```

### Using Context
```python
from orchestrator.context import MemoryManager

manager = MemoryManager("context.db")

# Log a mistake to prevent repetition
manager.log_mistake(
    description="Used string formatting in SQL query",
    correction="Changed to parameterized query",
    prevention="Always use ? placeholders",
    category="security"
)

# Get relevant context for a task
context = manager.get_relevant_context(
    task_description="Implement user authentication"
)
```

### Search Capabilities
- **BM25**: Keyword-based search
- **Semantic**: Embedding-based similarity
- **Hybrid**: Combined BM25 + semantic with RRF fusion

## Security Guidelines
- Never use `shell=True` with user input
- Always use parameterized SQL queries
- Validate all inputs with Pydantic
- Use `secrets.compare_digest()` for token comparison
- Log errors without sensitive data
- Check `.claude/skills/security/` for detailed patterns
