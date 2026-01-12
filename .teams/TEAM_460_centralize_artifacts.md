# TEAM_460: Centralize Artifacts and Clean Up .gitignore

## **Context**
The project has many scattered `.gitignore` files, especially in the `toolchain/` directory. Artifact generation should be centralized, and `.gitignore` files should be consolidated to the root where possible.

## **Status**
- [ ] Research artifact generation locations
- [ ] Identify redundant `.gitignore` files
- [ ] Consolidate artifact paths into root `.gitignore`
- [ ] Remove scattered `.gitignore` files
- [ ] Ensure project builds and tests pass

## **Notes**
- The `toolchain/` directory seems to have many subprojects (coreutils, brush, dash, busybox) with their own `.gitignore` files.
- Many of these might be from upstream repositories if they are submodules or vendored code.
