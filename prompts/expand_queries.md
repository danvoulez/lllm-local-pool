# EXPAND_QUERIES — ampliar termos de busca (vídeos e músicas)

Gere **listas curtas e objetivas** para busca web. Foco em **diversidade controlada** e **baixo custo**.

## Regras
- Não use termos explícitos; prefira **descrições neutras/estéticas**.
- Inclua **negativos** para evitar duplicatas, baixa qualidade e termos proibidos.
- Sugira **sites genéricos** (quando aplicável), nunca invente URLs.
- Saída **somente JSON**.

## Entrada (JSON)
```json
{
  "seed_terms": ["cinematic lounge", "ambient slow burn"],
  "media": "video|music",
  "lang": ["pt","en"],
  "duration_target_sec": 600,
  "negatives_hint": ["cam rip","compilation lowres","duplicate"],
  "constraints": {"min_resolution":"720p","license_hint":"non-drm"}
}
```

## Saída (JSON)
```json
{
  "must_include": ["cinematic lounge", "ambient slow burn"],
  "optional": ["downtempo", "moody color grading", "studio soft light", "bokeh"],
  "negatives": ["cam rip","compilation","blurry","watermark","reaction","shorts"],
  "duration_range_sec": [540, 720],
  "site_hints": ["site:archive.org", "site:soundcloud.com", "site:bandcamp.com", "site:vimeo.com"],
  "language_mix": {"pt": 0.4, "en": 0.6},
  "notes": ["priorize '1080p' nas queries de vídeo", "adicionar -shorts -tiktok"]
}
```

## Observação
Reutilize resultados com TTL de 10–30 min no cache do pool.
