# ENRICH_METADATA — título, tags e resumo curto

Crie metadados **claros e neutros**, focados em estética/ritmo (sem termos explícitos).

## Regras
- Título ≤ **60** caracteres; resumo ≤ **120** caracteres.
- 4–8 **tags** objetivas (cores, clima, estilo).
- Se suspeitar de violação de política, sinalize em `content_flags`.
- Saída **somente JSON**.

## Entrada (JSON)
```json
{
  "asset": {
    "id":"c1",
    "features": {"palette":["cool","night"],"style":["cinematic","slow burn"],"duration_sec": 604, "resolution":"1080p"},
    "audio":{"lufs": -14.1},
    "tags":["studio","bokeh"]
  }
}
```

## Saída (JSON)
```json
{
  "title": "Cinematic Night — slow burn em luz fria",
  "one_liner": "Texturas frias e ritmo suave com bokeh discreto e acabamento 1080p.",
  "tags": ["cinematic","cool","night","slow-burn","bokeh","1080p","studio"],
  "mood": "downtempo",
  "content_flags": [],
  "notes": ["mantém coerência com duração alvo ~10min"]
}
```

## Observações
Evite palavras sensacionalistas; privilegie estética e técnica.
