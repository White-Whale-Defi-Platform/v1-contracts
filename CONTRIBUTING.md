# How to contribute to this repository

We are thrilled that you want to contribute to our contracts! The best way to contribute to the project is to get familiarized with the code, fork it, make your contributions and then create a pull request to be merged into the project.

The first thing you need to do is to fork the repository to your GitHub account and clone it to your local machine.

1. Fork the repository.
    - Go to our repository, i.e. [White Whale Defi Platform](https://github.com/White-Whale-Defi-Platform/contracts).
    - Click on the ["fork" button](https://github.com/White-Whale-Defi-Platform/contracts/fork) located on the top right corner of the page. 
    - Click on your GitHub account where you want to fork the repo.
2. Clone your fork to your local machine, preferably using the SSH URL. If you have issues cloning this repo, look at the [GitHub docs](https://docs.github.com/en/repositories/creating-and-managing-repositories/cloning-a-repository).
    - `git clone git@github.com:$USER/contracts.git` or `git clone https://github.com/$USER/contracts.git`
3. Set up your git user locally if you haven't already.
    - `git config --global user.name "your name or alias"`
    - `git config --global user.email "your email address"`
4. Install our pre-commit hook. **Do not skip this step**. This will make sure that your code doesn't have any issues and is formatted correctly before you even commit. If you don't install this, you risk your future pull request to fail on CI.
    - `./scripts/git_hooks/pre-commit.sh --install`
5. Make your contributions locally. The following are recommendations so that it is easier for anyone to understand what you are trying to achieve:
    - Please make sure to use clear commit messages.
    - Please favor small commits instead of large ones.
6. Make sure to update the schemas if you have modified the messages.
    - `cargo schema`
7. Make sure your code compiles, both for debug and production.
   - `cargo build`
   - `cargo wasm`
   - `docker run --rm -v "$(pwd)":/code \
     --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
     --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
     cosmwasm/workspace-optimizer:0.12.3`
8. Test your code. Any changes you introduce need to be tested. **Untested code will be rejected**. If you are not sure how to create tests, please refer to existing ones.
    - `cargo test`
9. Push your changes to your repository.
    - `git push --set-upstream $YOUR_ORIGIN $YOUR_BRANCH_NAME"`
10. Create a pull request. Go to your [repository](https://github.com/$USER/contracts.git) and create a pull request against White Whale's repository main branch as base.
     - Please fill in the template presented to you when creating the pull request. Follow the instructions on the template.
     - Pull request that doesn't follow the template or is not filled in properly *will be considered incomplete*. 
11. Follow up the discussions on the PR as there might be requests from other members.
12. Wait for your PR to be approved and merged.

Thank you so much for taking the time to review our contracts and help decentralizing the enforcement the peg! #BeTheWhale

## Docs
When in doubt, please take a look at our [Litepaper](https://whitewhale.money/Litepaper.pdf) or our [contract's documentation](https://white-whale-defi-platform.github.io/docs/).
