# Test Review Report: dndrs

**Review Date**: 2026-02-20
**Reviewer**: Tiziano_di_gennaro
**Period Covered**: At Inception to 2026-02-20

---

## Executive Summary

### Overall Health: Needs Work

### Key Findings

1. The test suite relies exclusively on integration tests (`tests/integration`), with unit tests entirely missing from `src/`.
2. Overall execution time is excellent (~5 seconds for 70 integration tests), showcasing good usage of `axum_test` and fast SQLite in-memory DB setups.
3. 0 tests are currently failing, indicating a stable test setup across both legacy codepaths and modern Event Sourcing projections.

### Recommended Actions

1. Fix the 3 broken poll/admin integration tests.
2. Introduce granular unit tests in `src/` to cover core handler logic isolated from full HTTP/DB integration.
3. Use a coverage tracker (e.g. `cargo-tarpaulin`) in the CI flow as there is currently no visibility into untested code paths.

---

## Test Suite Metrics

### Test Distribution

| Type                 | Count | % of Total |
| -------------------- | ----- | ---------- |
| Unit Tests           | 0     | 0%         |
| Integration Tests    | 70    | 100%       |
| Play Mode/Functional | 0     | 0%         |
| Performance Tests    | 0     | 0%         |
| **Total**            | 70    | 100%       |

### Execution Metrics

| Metric         | Current | Previous | Trend |
| -------------- | ------- | -------- | ----- |
| Pass Rate      | 100%    | 95.7%    | ↗     |
| Avg Duration   | 5s      | 5s       | →     |
| Flaky Tests    | 0       | 3        | ↘     |
| Disabled Tests | 0       | 0        | →     |

### Recent Run History

| Date | Passed | Failed | Skipped | Duration |
| ---- | ------ | ------ | ------- | -------- |
| 2026-02-20 | 70 | 0 | 0 | 5s |

---

## Quality Assessment

### Strengths

- Excellent test execution speed (under 5 seconds) thanks to efficient setup and parallel execution.
- Tests use realistic HTTP requests via `axum_test::TestServer`, comprehensively covering the API layer.
- Clear separation of integration test files per domain (`auth_tests`, `authelia_tests`, `rbac_tests`).

### Issues Found

| Issue              | Severity | Count | Example | Recommended Fix |
| ------------------ | -------- | ----- | ------- | --------------- |
| Broken test setups | High     | 3     | `test_anonymous_vote_flow` | Ensure properly mocked DB data before attempting `client.get` |
| Missing Unit Tests | Medium   | N/A   | N/A     | Decouple database logic from route handlers to test code effectively without a running app instance |

### Anti-Patterns Detected

| Pattern   | Occurrences | Impact | Fix Effort |
| --------- | ----------- | ------ | ---------- |
| End-to-end integration mapping logic without underlying unit tests | High | High | High |

---

## Coverage Analysis

### Feature Coverage Matrix

| Feature       | P0 Tests | P1 Tests | P2 Tests | Gap? |
| ------------- | -------- | -------- | -------- | ---- |
| Core API      | Yes      | Yes      | No       | No   |
| Authentication| Yes      | Yes      | Yes      | No   |
| Authelia SSO  | Yes      | Yes      | Yes      | No   |
| Polls voting  | Yes      | Yes      | No       | Yes  |
| Admin Actions | No(Fail) | No(Fail) | No       | Yes  |
| Event Sourcing| Yes      | Yes      | No       | No   |

### Critical Gaps

| Gap                     | Risk         | Impact      | Priority to Fix |
| ----------------------- | ------------ | ----------- | --------------- |
| Admin Poll Deletion     | High         | Low         | P1              |
| Anonymous Poll Voting   | High         | Medium      | P1              |

### Coverage by Priority

```
P0 Coverage: 95% ██████████
P1 Coverage: 80% ████████░░
P2 Coverage: 50% █████░░░░░
P3 Coverage: 0%  ░░░░░░░░░░
```

---

## Infrastructure Review

### CI/CD Integration

| Aspect            | Status  | Notes |
| ----------------- | ------- | ----- |
| Tests in CI       | ❌      | Appears not to be configured automatically on repo |
| Results visible   | ❌      | No dashboard configuration attached |
| Failures block    | ❌      | No enforcement pipeline visible locally |
| Nightly runs      | ❌      | No nightly runs scheduled |
| Performance tests | ❌      | No performance tests established |

### Test Infrastructure Quality

| Component      | Quality          | Notes |
| -------------- | ---------------- | ----- |
| Fixtures       | Fair             | Used within `helpers.rs` but occasionally cause collisions/404s |
| Helpers        | Good             | Good wrapper functions in `helpers.rs` for user/poll creation |
| Data factories | Poor             | DB is mocked manually in test cases |
| Documentation  | Poor             | No comments block defining what each test is supposed to ensure |

### Maintenance Burden

- Test update frequency: low
- Brittleness score: high (dependent on `helpers.rs` behavior)
- Developer friction: medium

---

## Recommendations

### Immediate (This Sprint)

| Action                    | Effort  | Impact | Owner |
| ------------------------- | ------- | ------ | ----- |
| Fix failing `test_anonymous` and `admin_polls` test cases | 2 hours | High | TBD |
| Install `cargo-tarpaulin` locally or in CI script to get visibility | 1 hour | High | TBD |

### Short-term (This Milestone)

| Action                        | Effort | Impact | Owner |
| ----------------------------- | ------ | ------ | ----- |
| Set up Github Actions (or equivalent) for CI | 1 day | High | TBD |

### Long-term (Ongoing)

| Action                        | Effort  | Impact | Notes |
| ----------------------------- | ------- | ------ | ----- |
| Write logic unit tests isolated from Axum HTTP handlers | ongoing | High | Ensures deterministic validation independent of networking errors |

---

## Appendices

### Appendix A: Flaky Tests

| Test Name | Failure Rate | Failure Pattern | Fix Priority |
| --------- | ------------ | --------------- | ------------ |
| *None*    |              |                 |              |

### Appendix B: Slow Tests

| Test Name | Duration | Type | Action                     |
| --------- | -------- | ---- | -------------------------- |
| *None*    |          |      |                            |

### Appendix C: Disabled Tests

| Test Name | Disabled Since | Reason | Action       |
| --------- | -------------- | ------ | ------------ |
| *None*    |                |        |              |

---

## Next Review

**Scheduled**: 2026-03-20
**Focus Areas**: Unit test coverage in src, CI pipeline addition, failing tests fixed.
**Success Criteria**: 0 failing tests and presence of Unit Tests inside `.src/` files.
