#!/bin/bash
# Setup script for LLM Pool models

echo "Installing required models for LLM Pool..."
echo "This will download ~10GB total. Ensure you have enough space."
echo ""

# Model 1: Phi-3 Mini (4B) - Fast and lightweight
echo "1/3 Downloading Phi-3 Mini (4B)..."
ollama pull phi3:mini

# Model 2: Llama 3.1 8B - General purpose judge
echo "2/3 Downloading Llama 3.1 8B..."
ollama pull llama3.1:8b

# Model 3: Gemma 2 9B - Reranking and refinement
echo "3/3 Downloading Gemma 2 9B..."
ollama pull gemma2:9b

echo ""
echo "✅ All models installed successfully!"
echo ""
echo "Verifying installation..."
ollama list
