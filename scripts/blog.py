#!/usr/bin/env python3
"""Generate the static Grit blog from Markdown content."""
from __future__ import annotations

import email.utils
import html
import re
import shutil
import unicodedata
from dataclasses import dataclass
from datetime import date, datetime, timezone
from pathlib import Path
from xml.sax.saxutils import escape as xml_escape

ROOT = Path(__file__).resolve().parents[1]
CONTENT_DIR = ROOT / "content" / "blog"
OUT_DIR = ROOT / "docs" / "blog"
SITE_URL = "https://grit-scm.com"
SITE_TITLE = "the Grit project"
BLOG_TITLE = "project notes from Grit"
BLOG_DESCRIPTION = "Short deep dives into building a Git-compatible, library-oriented Rust implementation."
AUTHOR = "the Grit project"

FRONT_MATTER_RE = re.compile(r"\A---\s*\n(.*?)\n---\s*\n", re.DOTALL)
HEADING_RE = re.compile(r"^(#{1,6})\s+(.+?)\s*#*\s*$")
LINK_RE = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")
BOLD_RE = re.compile(r"\*\*([^*]+)\*\*")
EM_RE = re.compile(r"(?<!\*)\*([^*]+)\*(?!\*)")
CODE_RE = re.compile(r"`([^`]+)`")


@dataclass(frozen=True)
class TocItem:
    level: int
    text: str
    anchor: str


@dataclass(frozen=True)
class Post:
    slug: str
    title: str
    summary: str
    author: str
    published: date
    updated: datetime
    source: Path
    url: str
    body_html: str
    toc: list[TocItem]

    @property
    def rfc822_date(self) -> str:
        dt = datetime.combine(self.published, datetime.min.time(), tzinfo=timezone.utc)
        return email.utils.format_datetime(dt)

    @property
    def iso_datetime(self) -> str:
        return datetime.combine(self.published, datetime.min.time(), tzinfo=timezone.utc).isoformat()

    @property
    def display_date(self) -> str:
        return self.published.strftime("%B %-d, %Y")


def parse_front_matter(text: str) -> tuple[dict[str, str], str]:
    match = FRONT_MATTER_RE.match(text)
    if not match:
        return {}, text
    meta: dict[str, str] = {}
    for raw_line in match.group(1).splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        key, sep, value = line.partition(":")
        if not sep:
            raise ValueError(f"invalid front matter line: {raw_line!r}")
        meta[key.strip()] = value.strip().strip('"\'')
    return meta, text[match.end():]


def slugify(value: str) -> str:
    normalized = unicodedata.normalize("NFKD", value).encode("ascii", "ignore").decode("ascii")
    slug = re.sub(r"[^a-zA-Z0-9]+", "-", normalized.lower()).strip("-")
    return slug or "section"


def inline_md(value: str) -> str:
    protected: list[str] = []

    def protect_code(match: re.Match[str]) -> str:
        protected.append(f"<code>{html.escape(match.group(1))}</code>")
        return f"\u0000{len(protected) - 1}\u0000"

    escaped = html.escape(value)
    escaped = CODE_RE.sub(protect_code, escaped)
    escaped = LINK_RE.sub(lambda m: f'<a href="{html.escape(m.group(2), quote=True)}">{m.group(1)}</a>', escaped)
    escaped = BOLD_RE.sub(r"<strong>\1</strong>", escaped)
    escaped = EM_RE.sub(r"<em>\1</em>", escaped)
    for index, replacement in enumerate(protected):
        escaped = escaped.replace(f"\u0000{index}\u0000", replacement)
    return escaped


def markdown_to_html(markdown: str) -> tuple[str, list[TocItem]]:
    lines = markdown.splitlines()
    output: list[str] = []
    toc: list[TocItem] = []
    anchors: dict[str, int] = {}
    paragraph: list[str] = []
    list_kind: str | None = None
    in_code = False
    code_lines: list[str] = []

    def unique_anchor(text: str) -> str:
        base = slugify(re.sub(r"<[^>]+>", "", text))
        count = anchors.get(base, 0)
        anchors[base] = count + 1
        return base if count == 0 else f"{base}-{count + 1}"

    def flush_paragraph() -> None:
        nonlocal paragraph
        if paragraph:
            output.append(f"<p>{inline_md(' '.join(paragraph))}</p>")
            paragraph = []

    def close_list() -> None:
        nonlocal list_kind
        if list_kind:
            output.append(f"</{list_kind}>")
            list_kind = None

    for line in lines:
        if line.strip().startswith("```"):
            if in_code:
                output.append(f"<pre><code>{html.escape(chr(10).join(code_lines))}</code></pre>")
                code_lines = []
                in_code = False
            else:
                flush_paragraph(); close_list(); in_code = True; code_lines = []
            continue
        if in_code:
            code_lines.append(line)
            continue
        if not line.strip():
            flush_paragraph(); close_list(); continue
        heading = HEADING_RE.match(line)
        if heading:
            flush_paragraph(); close_list()
            level = len(heading.group(1))
            text = heading.group(2).strip()
            anchor = unique_anchor(text)
            if level >= 2:
                toc.append(TocItem(level, re.sub(r"[`*_]", "", text), anchor))
            output.append(f'<h{level} id="{anchor}">{inline_md(text)}</h{level}>')
            continue
        unordered = re.match(r"^[-*]\s+(.+)$", line)
        ordered = re.match(r"^\d+[.)]\s+(.+)$", line)
        if unordered or ordered:
            flush_paragraph()
            wanted = "ul" if unordered else "ol"
            if list_kind != wanted:
                close_list(); output.append(f"<{wanted}>"); list_kind = wanted
            item = unordered.group(1) if unordered else ordered.group(1)
            output.append(f"<li>{inline_md(item)}</li>")
            continue
        if line.startswith("> "):
            flush_paragraph(); close_list()
            output.append(f"<blockquote>{inline_md(line[2:].strip())}</blockquote>")
            continue
        paragraph.append(line.strip())
    flush_paragraph(); close_list()
    if in_code:
        output.append(f"<pre><code>{html.escape(chr(10).join(code_lines))}</code></pre>")
    return "\n".join(output), toc


def load_posts() -> list[Post]:
    posts: list[Post] = []
    for path in sorted(CONTENT_DIR.glob("*.md")):
        meta, body = parse_front_matter(path.read_text(encoding="utf-8"))
        title = meta.get("title")
        if not title:
            first_heading = next((HEADING_RE.match(line) for line in body.splitlines() if HEADING_RE.match(line)), None)
            title = first_heading.group(2) if first_heading else path.stem.replace("-", " ").title()
        slug = meta.get("slug") or path.stem
        published = date.fromisoformat(meta.get("date", date.today().isoformat()))
        summary = meta.get("summary", "")
        author = meta.get("author", AUTHOR)
        body_html, toc = markdown_to_html(body)
        updated = datetime.fromtimestamp(path.stat().st_mtime, tz=timezone.utc)
        posts.append(Post(slug, title, summary, author, published, updated, path, f"blog/{slug}/", body_html, toc))
    return sorted(posts, key=lambda p: (p.published, p.slug), reverse=True)


def page_shell(title: str, description: str, body: str, base: str, blog_href: str, rss_href: str, atom_href: str, extra_head: str = "") -> str:
    home_href = f"{base}/" if base != "." else "./"
    logo_href = f"{base}/grit-logo.svg"
    spec_href = f"{base}/v1-scope.md"
    return f"""<!doctype html>
<html lang=\"en\">
<head>
<meta charset=\"utf-8\" />
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
<title>{html.escape(title)}</title>
<meta name=\"description\" content=\"{html.escape(description, quote=True)}\" />
<link rel=\"alternate\" type=\"application/rss+xml\" title=\"Grit blog RSS\" href=\"{rss_href}\" />
<link rel=\"alternate\" type=\"application/atom+xml\" title=\"Grit blog Atom\" href=\"{atom_href}\" />
{extra_head}
<style>{CSS}</style>
</head>
<body>
<header class=\"site-header\">
  <a class=\"brand\" href=\"{home_href}\"><img src=\"{logo_href}\" alt=\"\" /> <span><span>the</span> <strong>Grit</strong> <span>project</span></span></a>
  <nav aria-label=\"Primary\"><a href=\"{blog_href}\">Blog</a><a href=\"{rss_href}\">RSS</a><a href=\"{spec_href}\">Spec →</a></nav>
</header>
{body}
</body>
</html>
"""


def render_index(posts: list[Post]) -> str:
    rows = "\n".join(
        f'<li><time datetime="{post.published.isoformat()}">{post.published.strftime("%b %d, %Y")}</time> <a href="{post.slug}/">{html.escape(post.title)}</a></li>'
        for post in posts
    ) or '<li><span class="empty">No posts yet.</span></li>'
    body = f"""<main class=\"blog-index\">
  <section class=\"hero\">
    <p class=\"eyebrow\">Blog</p>
    <h1>{html.escape(BLOG_TITLE)}</h1>
    <p>{html.escape(BLOG_DESCRIPTION)}</p>
  </section>
  <ol class=\"post-list\">{rows}</ol>
</main>"""
    return page_shell(f"Blog - {SITE_TITLE}", BLOG_DESCRIPTION, body, "..", "./", "rss.xml", "atom.xml", '<link rel="canonical" href="./" />')


def render_post(post: Post) -> str:
    toc = "\n".join(f'<li class="toc-level-{item.level}"><a href="#{item.anchor}">{html.escape(item.text)}</a></li>' for item in post.toc)
    if not toc:
        toc = '<li><span>No sections</span></li>'
    body = f"""<main class=\"post-layout\">
  <article class=\"post\">
    <p class=\"breadcrumb\"><a href=\"../\">blog</a> / <time datetime=\"{post.published.isoformat()}\">{post.published.isoformat()}</time> / {html.escape(post.author)}</p>
    <h1>{html.escape(post.title)}</h1>
    {f'<p class=\"dek\">{html.escape(post.summary)}</p>' if post.summary else ''}
    <div class=\"content\">{post.body_html}</div>
  </article>
  <aside class=\"toc\" aria-label=\"On this page\"><h2>On this page</h2><ol>{toc}</ol></aside>
</main>"""
    extra = f'<link rel="canonical" href="./" />\n<meta property="og:title" content="{html.escape(post.title, quote=True)}" />'
    return page_shell(f"{post.title} - {SITE_TITLE}", post.summary or BLOG_DESCRIPTION, body, "../..", "../", "../rss.xml", "../atom.xml", extra)


def render_rss(posts: list[Post]) -> str:
    items = "\n".join(f"""  <item>
    <title>{xml_escape(post.title)}</title>
    <link>{SITE_URL}/{post.url}</link>
    <guid>{SITE_URL}/{post.url}</guid>
    <pubDate>{post.rfc822_date}</pubDate>
    <description>{xml_escape(post.summary)}</description>
  </item>""" for post in posts)
    latest = posts[0].rfc822_date if posts else email.utils.format_datetime(datetime.now(timezone.utc))
    return f"""<?xml version=\"1.0\" encoding=\"utf-8\"?>
<rss version=\"2.0\"><channel>
  <title>{xml_escape(BLOG_TITLE)}</title>
  <link>{SITE_URL}/blog/</link>
  <description>{xml_escape(BLOG_DESCRIPTION)}</description>
  <lastBuildDate>{latest}</lastBuildDate>
{items}
</channel></rss>
"""


def render_atom(posts: list[Post]) -> str:
    updated = posts[0].iso_datetime if posts else datetime.now(timezone.utc).isoformat()
    entries = "\n".join(f"""  <entry>
    <title>{xml_escape(post.title)}</title>
    <link href=\"{SITE_URL}/{post.url}\" />
    <id>{SITE_URL}/{post.url}</id>
    <updated>{post.iso_datetime}</updated>
    <published>{post.iso_datetime}</published>
    <author><name>{xml_escape(post.author)}</name></author>
    <summary>{xml_escape(post.summary)}</summary>
  </entry>""" for post in posts)
    return f"""<?xml version=\"1.0\" encoding=\"utf-8\"?>
<feed xmlns=\"http://www.w3.org/2005/Atom\">
  <title>{xml_escape(BLOG_TITLE)}</title>
  <link href=\"{SITE_URL}/blog/\" />
  <link rel=\"self\" href=\"{SITE_URL}/blog/atom.xml\" />
  <id>{SITE_URL}/blog/</id>
  <updated>{updated}</updated>
{entries}
</feed>
"""


CSS = r'''
:root{--bg:#f8f2e8;--ink:#211711;--muted:#736961;--line:#e2d6c7;--accent:#b64729;--paper:#fff9f1;--code:#f3e8de}*{box-sizing:border-box}html{scroll-behavior:smooth}body{margin:0;background:var(--bg);color:var(--ink);font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;font-size:16px;line-height:1.65}.site-header{height:4.8rem;border-bottom:1px solid var(--line);display:flex;align-items:center;justify-content:space-between;padding:0 2.4rem}.brand{display:flex;align-items:center;gap:.65rem;color:var(--ink);font-size:1rem;font-weight:720;text-decoration:none}.brand img{width:2.25rem;height:2.25rem}.brand span span{color:#645a52;font-weight:650}.brand strong{color:var(--ink)}nav{display:flex;gap:1.55rem;font-size:.95rem}nav a,.content a,.breadcrumb a{color:var(--accent);text-decoration:none}.blog-index{max-width:66rem;margin:0 auto;padding:5.2rem 2rem}.hero{text-align:center;max-width:45rem;margin:0 auto 3rem}.eyebrow,.toc h2{text-transform:uppercase;letter-spacing:.24em;color:var(--accent);font:700 .72rem/1 ui-monospace,SFMono-Regular,Menlo,monospace}.hero h1{font:800 clamp(2.4rem,5.2vw,4.25rem)/.98 ui-monospace,SFMono-Regular,Menlo,monospace;letter-spacing:-.05em;margin:1rem 0;color:var(--ink)}.hero p:not(.eyebrow){font-size:1.08rem;color:#534941}.post-list{list-style:none;margin:0 auto;padding:0;max-width:50rem;border-top:1px solid var(--line)}.post-list li{display:flex;gap:1.2rem;border-bottom:1px solid var(--line);padding:.78rem 0;font:700 .88rem/1.4 ui-monospace,SFMono-Regular,Menlo,monospace}.post-list time{color:#837a72;min-width:8.8rem}.post-list a{color:var(--accent);text-decoration:none}.post-layout{display:grid;grid-template-columns:minmax(0,1fr) 15rem;gap:4rem;max-width:92rem;margin:0 auto;padding:5rem 4rem}.post{max-width:62rem}.breadcrumb{font:700 .86rem/1.4 ui-monospace,SFMono-Regular,Menlo,monospace;color:#837a72;margin:0 0 1.8rem}.post>h1{font:850 clamp(2.45rem,4.9vw,4.25rem)/1 ui-monospace,SFMono-Regular,Menlo,monospace;letter-spacing:-.06em;margin:0 0 2.2rem}.dek{font-size:1.08rem;color:#584d45}.content h2{font-size:1.35rem;line-height:1.25;margin:2.7rem 0 .8rem}.content h3{font-size:1.08rem;margin:2rem 0 .6rem}.content p{margin:0 0 1.25rem;color:#51473f}.content ul,.content ol{color:#51473f;margin:0 0 1.25rem 1.1rem}.content code{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;background:var(--code);border:1px solid #ded0c4;border-radius:.35rem;padding:.04rem .28rem}.content pre{background:#fff6ee;border:1px solid #ded0c4;border-radius:.7rem;padding:1rem;margin:1.4rem 0;overflow:auto}.content pre code{background:transparent;border:0;padding:0}.content blockquote{border-left:3px solid var(--accent);margin:1.4rem 0;padding:.2rem 0 .2rem 1rem;color:#5c5148}.toc{position:sticky;top:1.5rem;align-self:start;border-left:1px solid var(--line);padding-left:1.1rem;font:700 .82rem/1.45 ui-monospace,SFMono-Regular,Menlo,monospace;color:#81776f}.toc h2{color:#81776f;margin:0 0 1rem}.toc ol{list-style:none;margin:0;padding:0}.toc li{margin:0 0 .75rem}.toc a{color:#81776f;text-decoration:none}.toc-level-3{padding-left:.8rem}@media(max-width:900px){.site-header{padding:0 1.2rem}.post-layout{display:block;padding:2.5rem 1.25rem}.toc{position:static;border-left:0;border-top:1px solid var(--line);margin-top:2.5rem;padding:1.25rem 0 0}.blog-index{padding:3.25rem 1.25rem}.post-list li{display:block}.post-list time{display:block;margin-bottom:.25rem}.post>h1{font-size:2.45rem}}
'''


def main() -> None:
    posts = load_posts()
    if OUT_DIR.exists():
        shutil.rmtree(OUT_DIR)
    OUT_DIR.mkdir(parents=True)
    (OUT_DIR / "index.html").write_text(render_index(posts), encoding="utf-8")
    (OUT_DIR / "rss.xml").write_text(render_rss(posts), encoding="utf-8")
    (OUT_DIR / "atom.xml").write_text(render_atom(posts), encoding="utf-8")
    for post in posts:
        post_dir = OUT_DIR / post.slug
        post_dir.mkdir(parents=True)
        (post_dir / "index.html").write_text(render_post(post), encoding="utf-8")
    print(f"generated {len(posts)} blog post(s) in {OUT_DIR.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
