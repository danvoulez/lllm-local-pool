#!/bin/bash
# Test script for HTTP API

echo "🧪 Testing LLM Pool HTTP API"
echo ""

# Health check
echo "1️⃣  Health Check"
curl -s http://localhost:7071/health | jq .
echo ""
echo ""

# Test expand_queries
echo "2️⃣  Test expand_queries (FASTEST)"
curl -s -X POST http://localhost:7071/v1/infer \
  -H "Content-Type: application/json" \
  -d '{
    "task": "expand_queries",
    "prompt": "Generate search terms for ambient cinematic videos with warm colors",
    "max_tokens": 256,
    "deadline_ms": 1500
  }' | jq .
echo ""
echo ""

# Test enrich_metadata
echo "3️⃣  Test enrich_metadata (FASTEST)"
curl -s -X POST http://localhost:7071/v1/infer \
  -H "Content-Type: application/json" \
  -d '{
    "task": "enrich_metadata",
    "prompt": "Create a title and tags for a 10-minute cinematic video with bokeh and warm tones",
    "max_tokens": 128,
    "deadline_ms": 1000
  }' | jq .
echo ""
echo ""

# Test judge
echo "4️⃣  Test judge (JUDGE strategy)"
curl -s -X POST http://localhost:7071/v1/infer \
  -H "Content-Type: application/json" \
  -d '{
    "task": "judge",
    "prompt": "Select the best video: Option A (720p, 8min, studio), Option B (1080p, 10min, cinematic)",
    "max_tokens": 128,
    "deadline_ms": 1200,
    "strategy": "JUDGE"
  }' | jq .
echo ""
echo ""

# Test cache hit
echo "5️⃣  Test cache (repeat previous request)"
curl -s -X POST http://localhost:7071/v1/infer \
  -H "Content-Type: application/json" \
  -d '{
    "task": "enrich_metadata",
    "prompt": "Create a title and tags for a 10-minute cinematic video with bokeh and warm tones",
    "max_tokens": 128,
    "deadline_ms": 1000
  }' | jq .
echo ""

echo "✅ Tests complete!"
