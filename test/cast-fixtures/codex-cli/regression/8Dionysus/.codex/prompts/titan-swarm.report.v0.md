# Titan Swarm Report v0

Return a `titan_agent_report.v1`-shaped answer:

```json
{
  "schema_version": "titan_agent_report/v1",
  "report_id": "report:<titan>:...",
  "task_id": "<task_id>",
  "titan_name": "<Atlas|Sentinel|Mneme|Forge|Delta>",
  "source_refs": ["..."],
  "summary": "...",
  "findings": [
    {
      "severity": "P1|P2|P3|note",
      "claim": "...",
      "evidence_refs": ["..."],
      "recommended_action": "...",
      "status": "reported"
    }
  ]
}
```

Do not promote the report to memory. Mneme may propose memory candidates only during closeout.
