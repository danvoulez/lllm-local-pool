# RERANK — reordenar 10–12 candidatos para o slot

Você recebe uma lista de candidatos e deve retornar **apenas a ordem final** e um racional curto.

## Regras
- Otimize para **ritmo** (evitar repetição), **duração** (~10min), **qualidade técnica** e **variedade de paleta/tema**.
- **Não remova** itens; **reordene**. Se algum for inviável, marque em `policy.flags`.
- Saída **somente JSON**.

## Entrada (JSON)
```json
{
  "slot_context": {
    "slot_id":"86",
    "recent_slots":[
      {"id":"r11","palette":["warm","pastel"],"tags":["studio"],"duration_sec":602},
      {"id":"r12","palette":["warm"],"tags":["studio","closeup"],"duration_sec":593}
    ],
    "target_duration_sec": 600
  },
  "candidates":[
    {"id":"c1","duration_sec":604,"resolution":"1080p","palette":["cool"],"tags":["cinematic"],"qc":{"vmaf":90}},
    {"id":"c2","duration_sec":720,"resolution":"720p","palette":["warm"],"tags":["studio"],"qc":{"vmaf":87}}
    // … c3..c12
  ]
}
```

## Saída (JSON)
```json
{
  "ranking": ["c1","c5","c3","c2","c7","c8","c6","c11","c10","c4","c9","c12"],
  "diversity_gain": 0.07,
  "reasons": [
    "abre com paleta fria p/ contrastar histórico quente",
    "dur. próximas de 10min nas 3 primeiras posições",
    "evita 2 'studio warm' consecutivos"
  ],
  "policy": {
    "flags": [],
    "notes": []
  }
}
```

## Heurística (aplique em silêncio)
- Penalize sequências com mesma paleta dos últimos 2–3 slots
- Bonifique 1080p e VMAF ≥ 90
- Prefira 600±45s; durations muito fora vão para baixo
