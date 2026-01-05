# Phase 5 — Cleanup, Regression Protection, and Handoff

**TEAM_134** | Migrate Allocators to intrusive-collections

## Cleanup Tasks

### 1. Remove Obsolete Code

After migration is complete:
- Remove any dead code from page.rs (old next/prev accessors if any)
- Remove commented-out old implementations
- Clean up SAFETY comments that are now redundant

### 2. Update Documentation

- Add module-level docs explaining the intrusive list pattern
- Document the lazy initialization requirement in BuddyAllocator
- Update any architecture docs that reference the old pattern

### 3. Consider Slab Allocator Migration

The slab allocator (`levitate-hal/src/allocator/slab/list.rs`) has its own `SlabList<T>` implementation. Decide whether to:

**Option A:** Migrate to intrusive-collections (consistent pattern)
**Option B:** Keep custom SlabList (it's already safe enough)

Recommendation: Keep SlabList for now — it's self-contained and works.

---

## Regression Protection

### CI Verification

After migration, the following must pass:
```bash
# Unit tests
cargo xtask test unit

# Build check
cargo check --all-targets

# Unsafe audit (track count)
grep -rn "unsafe {" levitate-hal/src/allocator/ --include="*.rs" | wc -l
```

### Behavioral Tests

If any integration tests exercise the allocator:
- Boot tests
- Memory pressure tests
- Multi-allocation patterns

All must continue to pass.

---

## Handoff Checklist

- [ ] Page struct updated with LinkedListLink
- [ ] BuddyAllocator uses Option<LinkedList<PageAdapter>>
- [ ] All list operations rewritten
- [ ] All 5 buddy allocator tests pass
- [ ] Unsafe count reduced (target: 5 → 2)
- [ ] No performance regression observed
- [ ] Team file updated with completion status
- [ ] This plan marked as completed

---

## Success Metrics

| Metric | Before | Target | Actual |
|--------|--------|--------|--------|
| Unsafe blocks in buddy.rs | 5 | 2 | TBD |
| Allocator tests passing | 5/5 | 5/5 | TBD |
| New panics introduced | 0 | 0 | TBD |

---

## Future Work

After buddy allocator migration succeeds, consider:

1. **Slab allocator migration** — Apply same pattern if beneficial
2. **Remove SlabList custom impl** — If migrated to intrusive-collections
3. **Performance benchmarking** — Verify no regression in allocation hot path

---

## Summary

This plan migrates the buddy allocator from manual `NonNull<Page>` linked lists to `intrusive-collections::LinkedList<PageAdapter>` using a lazy initialization pattern to work around const initialization limitations.

**Key insight:** Use `Option<LinkedList<...>>` initialized to `None` in const `new()`, then create actual lists in `init()`.

**Expected outcome:**
- ~3 fewer unsafe blocks in buddy.rs
- Safer, more maintainable linked list operations
- Battle-tested library instead of manual pointer manipulation

**Estimated effort:** 1-2 sessions (~150 lines of changes)
