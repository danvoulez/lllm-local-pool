# RECOVERY_PLAN — plano de ação em até 3 passos

Quando algo falhar (ex.: baixa qualidade técnica, buffer crítico, fonte sem HD), descreva **um plano sucinto e seguro**.

## Regras
- Máximo **3 passos**. Sem gambiarras, sem burlar ToS, sem instruções invasivas.
- Se o cenário indicar DRM/ABR indisponível, **recomende abortar** com razão.
- Saída **somente JSON**.

## Entrada (JSON)
```json
{
  "incident": "low_vmaf_playout",
  "context": {
    "slot_id":"86",
    "current_asset":{"id":"a17","vmaf":81,"resolution":"720p","bitrate_kbps":1200},
    "buffer_minutes": 0.7,
    "alternatives":[{"id":"b02","vmaf":92,"resolution":"1080p"}]
  }
}
```

## Saída (JSON)
```json
{
  "severity": "medium",
  "steps": [
    {"action":"swap_asset","args":{"use_id":"b02"},"guardrails":["verify vmaf>=90","duration within 540..720"]},
    {"action":"bump_music_ratio","args":{"next_slots":2,"mood":"downtempo"},"guardrails":["keep queue FIFO within bucket"]},
    {"action":"note_incident","args":{"tag":"slow-HD-source"},"guardrails":["do not blacklist domain without audit"]}
  ],
  "rollback": {"if":"swap fails","do":"revert_to a17 and inject emergency loop 1 item"},
  "eta_min": 3,
  "reason": "asset b02 atende 1080p/VMAF92; troca rápida reduz risco de churn"
}
```

## Notas
- Use preferência por alternativas já baixadas
- Se tudo abaixo do limiar, recomende emergency loop temporário
