# ADR-0015: Local classifier strategy

**Status:** Proposed

**Context.** Contextual detection (names without cues, contextual identity, confidential-business content, defence markers in prose) needs ML, but Ollama can't be assumed available (LOCAL ONLY users might not have it; cloud-only users won't run an LLM).

**Decision.** Two-tier classifier behind the `Classifier` trait:
1. **Bundled small NER/classification model(s)** in ONNX run in-process (`ort`, CPU): fast, always available, hashed and pinned in the signed manifest (candidate: a PII/NER model with permissive licence — selection and licence review is OPEN-03).
2. **Optional Ollama-hosted LLM classifier** for deeper contextual risk and transform proposals (user-chosen model; capability-checked; treated as untrusted output with strict JSON-schema parsing, size caps, and no tool use).

Both are monotonic (ADR-0005). Evaluation: labelled synthetic corpora per category, with published precision/recall gates in CI; defence markers via lexicon + classifier.

**Consequences.** + Contextual detection works without Ollama; deterministic base unaffected. − Model size (installer growth), CPU latency, licence and bias issues, multilingual coverage gaps (state supported languages honestly).

**Alternatives.** Ollama-only (availability risk, heavy); cloud classification (contradicts local-first).
