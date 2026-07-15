#!/usr/bin/env bash
# LEGACY EXAMPLE. Verify flags against the installed Aider version before use.
# This script is not an input to Harness Guard and must never be executed by a scan.

aider --analytics-disable \
      --no-analytics \
      --model ollama_chat/llama3.2   # or your preferred local model
