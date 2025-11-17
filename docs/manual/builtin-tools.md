# Built-in Tools: Browser and Calculator

This note documents the initial scope for the two core tools we plan to embed inside the agent runtime. Keep it updated as the implementations evolve.

## Browser Tool

- **Purpose**: Fetch read-only textual data (HTML, JSON, plain text) over HTTPS and summarize key facts for the agent.
- **Capabilities**:
  - Accepts explicit URLs or entries from a trusted domain allowlist.
  - Issues bounded GET requests with strict limits on total download size, timeout, and redirect depth.
  - Strips scripts/embedded binary content, extracts title, main text snippet(s), and HTTP metadata, and emits a short summary plus citation info.
  - Returns error metadata (timeout, disallowed scheme, status >= 400, etc.) so the agent can recover.
- **Non-goals**:
  - No cookies, JavaScript execution, resource rendering, or login flows.
  - No downloading of large binaries, media files, or arbitrary attachments.
  - No attempt to infer or hallucinate content when the response is truncated; the tool must disclose truncation.
- **Integration**: Expose as a `ToolSchema` implementation that the session can enable/disable per configuration. Configuration knobs include allowlist domains, max body size, max redirects, and request timeout.

## Calculator Tool

- **Purpose**: Provide deterministic numeric evaluation for arithmetic and standard math functions, freeing the LLM from handling raw expressions.
- **Capabilities**:
  - Parses short infix expressions (addition, subtraction, multiplication, division, power, parentheses).
  - Supports elementary functions (sqrt, log, exp, sin/cos/tan and inverses) plus optional unit conversions.
  - Emits precise numeric results (f64) along with formatting helpers (scientific vs fixed) and detailed error diagnostics for malformed expressions.
- **Non-goals**:
  - No symbolic algebra, solving of equations, calculus, or long-running numerical optimization.
  - No plotting or stateful computation history.
- **Integration**: Provide a library API such as `CalculatorTool::execute(expr: &str) -> Result<CalculatorResult>` that implements `ToolSchema`. Configurable limits cover maximum expression length, evaluation depth, and precision mode.

## Documentation Requirements

- All documentation in this directory stays in English and is versioned with the code.
- Generated API references remain ignored (`target/doc` or temporary output directories) to avoid churn in git history.
