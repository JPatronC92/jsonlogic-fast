# Security Policy

## Supported Versions

Only the latest version of `jsonlogic-fast` is supported with security updates.

## Reporting a Vulnerability

If you discover a security vulnerability in `jsonlogic-fast`, please report it by opening an issue on the GitHub repository or by contacting the maintainers directly if a private security channel is available.

We consider the following to be valid security issues:
* **Denial of Service (DoS):** Maliciously crafted JSON rules or contexts that cause the engine to panic or enter an infinite loop.
* **Information Exposure:** Error messages leaking sensitive data from the evaluation context (we actively sanitize error messages to prevent this).
* **Memory Safety:** Any memory unsafety bug in the underlying Rust code that can be triggered from safe Python or WASM bindings.

## Threat Model

* **Rule Authoring:** Rules are assumed to be authored by trusted parties (e.g. your internal developers). While we guard against panics, executing untrusted, arbitrarily complex rules could lead to high CPU consumption.
* **Context Data:** Contexts are assumed to be untrusted (e.g. user input). The engine is designed to evaluate untrusted context data safely without side effects.
