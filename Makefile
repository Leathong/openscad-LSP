README_MD := out/README.md

$(README_MD): README.rst
	pandoc $< -o $@

.PHONY: check-repo
check-repo:
	@# Untracked files (like the generated README) count as dirty, so manually check for changes and use `--allow-dirty`.
	git diff --quiet && git diff --quiet --cached

package: $(README_MD)
	$(MAKE) check-repo
	cargo package --allow-dirty

publish: $(README_MD)
	$(MAKE) check-repo
	cargo publish --allow-dirty
