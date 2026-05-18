#!/usr/bin/env python3
from __future__ import annotations
import json

context = {
  "hook": "aoa_titan_memory_context",
  "wave": "fourteenth_wave",
  "message": "Titan Memory Loom available. Recall is candidate-grade; verify owner repos and source seeds before treating memory as truth.",
  "default_titans": ["Atlas", "Sentinel", "Mneme"],
  "gated_titans": {"Forge": "mutation", "Delta": "judgment"}
}
print(json.dumps({"additionalContext": json.dumps(context, ensure_ascii=False)}, ensure_ascii=False))
