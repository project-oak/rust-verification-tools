---
layout: default
regenerate: true
---

RVT is a collection of research tools/libraries to support both static verification
(formal verification) and dynamic verification (testing) of Rust.
RVT is dual-licensed (Apache/MIT) so that you can use and adapt our 
code for your own tools.

<div class="posts">
  {% for post in site.posts %}
    <article class="post">

      <h1><a href="{{ site.baseurl }}{{ post.url }}">{{ post.title }}</a></h1>

      <div class="entry">
        {{ post.excerpt }}
      </div>

      <a href="{{ site.baseurl }}{{ post.url }}" class="read-more">Read More</a>
    </article>
  {% endfor %}
</div>
