#!/bin/bash
export PATH=$PATH:/usr/local/bin

#
# White Whale contracts pre-commit hook, used to perform static analysis checks on changed files.
#
# Install the hook with the --install option.
#

project_toplevel="$(git rev-parse --show-toplevel)"
git_directory=$(git rev-parse --git-dir)

install_hook() {
  mkdir -p "$git_directory/hooks"
  ln -sfv "$project_toplevel/scripts/git_hooks/pre-commit.sh" "$git_directory/hooks/pre-commit"
}

if [ "$1" = "--install" ]; then
  if [ -f "$git_directory/hooks/pre-commit" ]; then
    read -r -p "There's an existing pre-commit hook. Do you want to overwrite it? [y/N] " response
    case "$response" in
    [yY][eE][sS] | [yY])
      install_hook
      ;;
    *)
      echo "Skipping hook installation :("
      exit $?
      ;;
    esac
  else
    install_hook
  fi
  exit $?
fi

# check for formatting

has_issues=0
first_file=1

for file in $(git diff --name-only --staged -- '*.rs'); do
    format_check_result="$(rustfmt --skip-children --force --write-mode diff $file 2>/dev/null || true)"
    if [ "$format_check_result" != "" ]; then
        if [ $first_file -eq 0 ]; then
            echo -n ", "
        fi
        echo -n "$file"
        has_issues=1
        first_file=0
    fi
done

if [ $has_issues -eq 0 ]; then
    exit 0
fi

echo ". Formatting issues were found in files listed above. Format your code with cargo fmt."
exit 1