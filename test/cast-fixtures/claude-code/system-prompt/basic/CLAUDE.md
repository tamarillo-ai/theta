## MindGraph Knowledge Base

This project has a MindGraph knowledge base at `.mindgraph/`.

### Search before reading
Before exploring code with Grep/Glob, search the knowledge graph first:
```bash
python3 -m tools search . "your query"
```

### After modifying wiki files
The watcher daemon auto-updates fingerprints. If not running:
```bash
python3 -m tools fingerprint .
```

### Available commands
| Command | Usage |
|---------|-------|
| search | `python3 -m tools search . "query"` |
| fingerprint | `python3 -m tools fingerprint .` |
| ingest | `python3 -m tools ingest . <source>` |
| lint | `python3 -m tools lint .` |
| watch | `python3 -m tools watch . start` |
