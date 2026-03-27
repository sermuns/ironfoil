release force-version="":
	#!/bin/sh
	set -e

	if [ -n "{{force-version}}" ]; then
		VERSION="{{force-version}}"
	else
		VERSION=$(git cliff --bumped-version | cut -d'v' -f2)
	fi
	cargo release -x $VERSION
	git cliff -o CHANGELOG.md --tag $VERSION
	git add CHANGELOG.md
	git commit --amend --no-edit
	git tag v$VERSION -f

push:
	git push
	git push --tags
