#!/usr/bin/env python3
from __future__ import annotations
import json

print(json.dumps({
    "additionalContext": "Titan Incarnation Spine available: summon named custom agents Atlas/Sentinel/Mneme, not generic architect/reviewer/memory-keeper. Forge requires mutation payload gate; Delta requires judgment payload gate. No autospawn.",
    "titan_names": ["Atlas", "Sentinel", "Mneme", "Forge", "Delta"],
    "autospawn": False,
}, ensure_ascii=False))
