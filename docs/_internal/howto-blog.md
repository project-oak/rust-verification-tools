# The rust-verificatio-tools blog

## Before pushing a new post

[Install jekyll](https://jekyllrb.com/docs/):

``` shell
sudo apt install ruby ruby-dev
gem install jekyll bundler --user-install
bundle config set --local path ~/.local/share/gem/
bundle install
```

Check that the blog builds and looks ok:

``` shell
# cd docs
bundle exec jekyll serve -o -I -l
```
