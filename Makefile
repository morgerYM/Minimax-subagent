.PHONY: version bump-patch bump-minor bump-major tag

VERSION_FILE := VERSION

version:
	@echo "Current version: $$(cat $(VERSION_FILE))"

bump-patch:
	@./scripts/bump.sh patch $(VERSION_FILE)

bump-minor:
	@./scripts/bump.sh minor $(VERSION_FILE)

bump-major:
	@./scripts/bump.sh major $(VERSION_FILE)

tag:
	@ver=$$(cat $(VERSION_FILE)); \
	git tag -a "v$$ver" -m "Release v$$ver"; \
	echo "Tagged v$$ver"
