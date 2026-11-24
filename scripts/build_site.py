#!/usr/bin/env python3
"""Build the documentation and gallery site into site/.

    scripts/build_gallery.sh        # renders the films the gallery page embeds (slow; once)
    scripts/build_wasm.sh           # the engine for the live player (once)
    scripts/build_site.py           # this: fast, rerun after editing docs
    python3 -m http.server -d site 8000

Standard library only. The site is static files: deploy the directory.

Sources of truth, none duplicated here:
  docs/*.md and skills/psanim/references/*.md   prose (rendered as written)
  crates/psanim/src/api.rs via `psanim api --json`   the reference
  gallery/*/                                      pieces: source, README, rendered outputs
  scripts/site/*.svg.html                        diagrams
"""
import html
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SITE = ROOT / "site"
PSANIM = ROOT / "target/release/psanim"

# --------------------------------------------------------------------------
# PostScript highlighting
# --------------------------------------------------------------------------

def vocabulary():
    out = subprocess.run([str(PSANIM), "api", "--json"], capture_output=True, text=True, check=True).stdout
    return json.loads(out)

API = vocabulary()
WORDS = {e["name"] for e in API}
TOKEN = re.compile(
    r"(?P<comment>%[^\n]*)"
    r"|(?P<string>\((?:\\.|[^()\\]|\((?:\\.|[^()\\])*\))*\))"
    r"|(?P<name>/[^\s()<>\[\]{}/%]*)"
    r"|(?P<number>[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?(?![^\s()<>\[\]{}/%]))"
    r"|(?P<brace>[{}\[\]]|<<|>>)"
    r"|(?P<word>[^\s()<>\[\]{}/%]+)"
)

def highlight_ps(code):
    out, pos = [], 0
    for m in TOKEN.finditer(code):
        out.append(html.escape(code[pos:m.start()]))
        kind, text = m.lastgroup, html.escape(m.group())
        if kind == "word":
            if m.group() in WORDS:
                out.append(f'<span class="t-anim" title="psanim">{text}</span>')
            else:
                out.append(text)
        else:
            out.append(f'<span class="t-{kind}">{text}</span>')
        pos = m.end()
    out.append(html.escape(code[pos:]))
    return "".join(out)

# --------------------------------------------------------------------------
# A small Markdown renderer: what these documents use, nothing more
# --------------------------------------------------------------------------

def slug(text):
    return re.sub(r"[^a-z0-9]+", "-", re.sub(r"<[^>]+>|`", "", text).lower()).strip("-")

def inline(text, link):
    codes = []
    def stash(m):
        codes.append(m.group(1))
        return f"\0{len(codes) - 1}\0"
    text = re.sub(r"`([^`]+)`", stash, text)
    text = html.escape(text, quote=False)
    text = re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", lambda m: f'<a href="{link(m.group(2))}">{m.group(1)}</a>', text)
    text = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", text)
    text = re.sub(r"(?<![\w*])\*([^*\n]+)\*(?![\w*])", r"<em>\1</em>", text)
    return re.sub(r"\0(\d+)\0", lambda m: f"<code>{html.escape(codes[int(m.group(1))])}</code>", text)

def markdown(text, link=lambda href: href):
    lines = text.split("\n")
    out, toc, i = [], [], 0
    def para(buf):
        if buf:
            out.append("<p>" + inline(" ".join(buf), link) + "</p>")
            buf.clear()
    buf = []
    while i < len(lines):
        line = lines[i]
        fence = re.match(r"^(\s*)```(.*)$", line)
        if fence:
            para(buf)
            lang = fence.group(2).strip().split()[0] if fence.group(2).strip() else ""
            body = []
            i += 1
            while i < len(lines) and not lines[i].strip().startswith("```"):
                body.append(lines[i][len(fence.group(1)):] if lines[i].startswith(fence.group(1)) else lines[i])
                i += 1
            code = "\n".join(body)
            inner = highlight_ps(code) if lang == "postscript" else html.escape(code)
            out.append(f'<pre class="code lang-{lang or "text"}"><code>{inner}</code></pre>')
        elif re.match(r"^#{1,4} ", line):
            para(buf)
            level = len(line) - len(line.lstrip("#"))
            title = line[level:].strip()
            anchor = slug(title)
            if level in (2, 3):
                toc.append((level, anchor, re.sub(r"`", "", title)))
            out.append(f'<h{level} id="{anchor}">{inline(title, link)}</h{level}>')
        elif line.startswith("|") and i + 1 < len(lines) and re.match(r"^\|[\s:|-]+\|\s*$", lines[i + 1]):
            para(buf)
            cells = lambda row: [c.strip() for c in re.split(r"(?<!\\)\|", row.strip().strip("|"))]
            head = cells(line)
            i += 2
            rows = []
            while i < len(lines) and lines[i].startswith("|"):
                rows.append(cells(lines[i]))
                i += 1
            i -= 1
            th = "".join(f"<th>{inline(c, link)}</th>" for c in head)
            trs = "".join("<tr>" + "".join(f"<td>{inline(c, link)}</td>" for c in r) + "</tr>" for r in rows)
            out.append(f'<div class="table"><table><thead><tr>{th}</tr></thead><tbody>{trs}</tbody></table></div>')
        elif re.match(r"^\s*([-*]|\d+\.) ", line):
            para(buf)
            ordered = bool(re.match(r"^\s*\d+\.", line))
            items = []
            while i < len(lines) and (re.match(r"^\s*([-*]|\d+\.) ", lines[i]) or (lines[i].startswith("  ") and lines[i].strip())):
                if re.match(r"^\s*([-*]|\d+\.) ", lines[i]):
                    items.append(re.sub(r"^\s*([-*]|\d+\.) ", "", lines[i]))
                else:
                    items[-1] += " " + lines[i].strip()
                i += 1
            i -= 1
            tag = "ol" if ordered else "ul"
            out.append(f"<{tag}>" + "".join(f"<li>{inline(x, link)}</li>" for x in items) + f"</{tag}>")
        elif line.startswith("> "):
            para(buf)
            out.append(f"<blockquote>{inline(line[2:], link)}</blockquote>")
        elif not line.strip():
            para(buf)
        else:
            buf.append(line.strip())
        i += 1
    para(buf)
    return "\n".join(out), toc

# --------------------------------------------------------------------------
# Pages
# --------------------------------------------------------------------------

# (output file, title, source Markdown relative to the repository, nav group)
DOCS = [
    ("guide/writing.html", "Writing an animation", "docs/authoring.md", "Guide"),
    ("guide/patterns.html", "Patterns", "skills/psanim/references/patterns.md", "Guide"),
    ("guide/craft.html", "Craft", "skills/psanim/references/craft.md", "Guide"),
    ("guide/pitfalls.html", "Pitfalls", "skills/psanim/references/pitfalls.md", "Guide"),
    ("guide/modes.html", "State, inputs, blur", "skills/psanim/references/modes.md", "Guide"),
    ("reference.html", "Vocabulary", None, "Reference"),
    ("guide/cli.html", "Command line", "docs/cli.md", "Reference"),
    ("guide/rust.html", "Rust API", "docs/rust.md", "Reference"),
    ("guide/browser.html", "Browser", "docs/wasm.md", "Reference"),
    ("guide/architecture.html", "Architecture", "docs/architecture.md", "Design"),
    ("guide/pscat.html", "psanim and PSCat", "docs/pscat-integration.md", "Design"),
    ("guide/security.html", "Security", "docs/security.md", "Design"),
    ("guide/benchmarks.html", "Benchmarks", "docs/benchmarks.md", "Design"),
    ("guide/testing.html", "Testing", "docs/testing.md", "Design"),
    ("guide/roadmap.html", "Limits and roadmap", "docs/roadmap.md", "Design"),
]
BY_SOURCE = {src: out for out, _, src, _ in DOCS if src}
BY_SOURCE["skills/psanim/references/vocabulary.md"] = "reference.html"
BY_SOURCE["docs/gallery.md"] = "gallery.html"

def rel(from_page, to_page):
    depth = from_page.count("/")
    return "../" * depth + to_page

def linker(page, source):
    """Rewrite a Markdown link between sources into a link between pages."""
    base = (ROOT / source).parent
    def link(href):
        if re.match(r"^[a-z]+:", href) or href.startswith("#"):
            return href
        path, _, frag = href.partition("#")
        try:
            target = str((base / path).resolve().relative_to(ROOT))
        except ValueError:
            return href
        if target in BY_SOURCE:
            return rel(page, BY_SOURCE[target]) + (f"#{frag}" if frag else "")
        return f"https://github.com/PSCatLabs/psanim/blob/main/{target}"  # a file in the repository
    return link

def shell(page, title, body, toc=None, wide=False):
    nav = []
    top = [("index.html", "psanim"), ("gallery.html", "Gallery"), ("guide/writing.html", "Guide"),
           ("reference.html", "Reference"), ("playground.html", "Playground"), ("guide/architecture.html", "Design")]
    top_html = "".join(
        f'<a class="{"brand" if i == 0 else ""}{" here" if out == page else ""}" href="{rel(page, out)}">{name}</a>'
        for i, (out, name) in enumerate(top))
    side = ""
    if not wide:
        group = None
        for out, name, _, g in DOCS:
            if g != group:
                nav.append(f"<h4>{g}</h4>")
                group = g
            nav.append(f'<a class="{"here" if out == page else ""}" href="{rel(page, out)}">{name}</a>')
        on_page = ""
        if toc:
            on_page = "<h4>On this page</h4>" + "".join(
                f'<a class="toc l{lvl}" href="#{a}">{html.escape(t)}</a>' for lvl, a, t in toc if lvl == 2)
        side = f'<aside class="side"><nav>{"".join(nav)}</nav></aside>'
        body = f'<div class="doc">{side}<main class="prose">{body}</main><aside class="onpage">{on_page}</aside></div>'
    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{html.escape(title)} · psanim</title>
<meta name="description" content="psanim: programmable animation in PostScript, on the PSCat graphics engine.">
<link rel="stylesheet" href="{rel(page, 'site.css')}">
</head>
<body>
<header class="top"><nav>{top_html}</nav></header>
{body}
<footer class="foot">psanim · animation as PostScript programs · built on <a href="https://github.com/PSCatLabs/pscat">PSCat</a> · this site is generated by <code>scripts/build_site.py</code></footer>
</body>
</html>
"""

def write(page, content):
    path = SITE / page
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)

def part(name):
    return (ROOT / "scripts/site" / name).read_text()

def build_docs():
    for out, title, source, _ in DOCS:
        if not source:
            continue
        path = ROOT / source
        if not path.exists():
            print(f"  (skipping {out}: no {source} yet)")
            continue
        text = path.read_text()
        # Diagrams: a line `<!-- diagram: name -->` in the Markdown is replaced.
        body, toc = markdown(text, linker(out, source))
        body = re.sub(r"<p>&lt;!-- diagram: ([a-z-]+) --&gt;</p>", lambda m: part(m.group(1) + ".svg.html"), body)
        if source == "docs/architecture.md":
            # The ASCII overview becomes the drawn one.
            body = re.sub(r'<pre class="code lang-text"><code>\s*┌.*?</code></pre>', part("architecture.svg.html"), body, count=1, flags=re.S)
            body = body.replace('<h3 id="purity-by-rollback-using-postscript-s-own-checkpoint">',
                                part("lifecycle.svg.html") + '<h3 id="purity-by-rollback-using-postscript-s-own-checkpoint">', 1)
            body = body.replace('<h3 id="what-is-native-and-what-is-postscript">',
                                part("timescopes.svg.html") + '<h3 id="what-is-native-and-what-is-postscript">', 1)
            body = body.replace('<h2 id="errors-and-diagnostics">', part("export.svg.html") + '<h2 id="errors-and-diagnostics">', 1)
        write(out, shell(out, title, body, toc))

def build_reference():
    cats = json.loads(subprocess.run([str(PSANIM), "api", "--json"], capture_output=True, text=True, check=True).stdout)
    listing = subprocess.run([str(PSANIM), "api"], capture_output=True, text=True, check=True).stdout
    titles = [l for l in listing.split("\n") if l and not l.startswith(" ") and not l.startswith(("Easings", "Transitions", "`"))]
    order = []
    for e in cats:
        if e["category"] not in order:
            order.append(e["category"])
    easings = re.search(r"^Easings: (.*)$", listing, re.M).group(1).split()
    transitions = re.search(r"^Transitions: (.*)$", listing, re.M).group(1).split()
    body = ['<h1>Vocabulary</h1>',
            f'<p class="lede">Every word psanim adds to PostScript: {len(cats)} of them. Generated from the table in '
            '<code>crates/psanim/src/api.rs</code>, the same one <code>psanim api</code> prints and the test suite checks '
            'against the interpreter. Stack effects read left to right, topmost operand last.</p>',
            '<input class="filter" type="search" placeholder="Filter words…" aria-label="Filter words" '
            'oninput="for (const e of document.querySelectorAll(\'.word\')) e.hidden = !e.textContent.toLowerCase().includes(this.value.toLowerCase())">']
    toc = []
    for cat, title in zip(order, titles):
        anchor = slug(title)
        toc.append((2, anchor, title.split(":")[0]))
        body.append(f'<h2 id="{anchor}">{html.escape(title)}</h2>')
        for e in [e for e in cats if e["category"] == cat]:
            stack = html.escape(e["stack"]).replace(f"  {e['name']}  ", f'  <b>{e["name"]}</b>  ')
            body.append(f'<div class="word" id="w-{e["name"]}"><pre class="stack">{stack}</pre>'
                        f'<p>{inline(e["summary"], lambda h: h)}</p></div>')
    body.append('<h2 id="easings">Easings</h2><p>For <code>ease</code>, <code>tween</code> and <code>keyframes</code>.</p>')
    # The chart is a psanim program: the engine draws its own curves.
    chart = SITE / "easings.png"
    done = subprocess.run([str(PSANIM), "frame", str(ROOT / "scripts/site/easings.ps"), "-o", str(chart), "-q", "--scale", "2"])
    if done.returncode == 0:
        body.append('<figure class="diagram"><img src="easings.png" alt="Eighteen easing curves plotted from 0 to 1" '
                    'style="width:100%;height:auto;display:block;border-radius:6px">'
                    '<figcaption>Input left to right, output bottom to top; the lines mark 0 and 1. '
                    'Drawn by <code>scripts/site/easings.ps</code>, a one-frame psanim program.</figcaption></figure>')
    body.append('<p class="chips">' + " ".join(f"<code>/{n}</code>" for n in easings) + "</p>")
    body.append('<h2 id="transitions">Transitions</h2><p>For <code>reel</code>.</p><p class="chips">'
                + " ".join(f"<code>/{n}</code>" for n in transitions) + "</p>")
    toc += [(2, "easings", "Easings"), (2, "transitions", "Transitions")]
    write("reference.html", shell("reference.html", "Vocabulary", "\n".join(body), toc))

def gallery_pieces():
    pieces = []
    for d in sorted((ROOT / "gallery").iterdir()):
        src = d / f"{d.name}.ps"
        if not src.exists():
            continue
        out = SITE / "gallery" / d.name
        info = json.loads((out / "info.json").read_text())["movie"] if (out / "info.json").exists() else None
        render = json.loads((out / "render.json").read_text()) if (out / "render.json").exists() else None
        readme = (d / "README.md").read_text() if (d / "README.md").exists() else ""
        pieces.append({"name": d.name, "src": src, "info": info, "render": render, "readme": readme,
                       "has_video": (out / f"{d.name}.mp4").exists()})
    # The order of the page: a curated sequence, then anything new.
    order = ["orrery", "afterglow", "paper-boat", "tidewater", "murmuration", "epicycles", "one-frame"]
    pieces.sort(key=lambda p: order.index(p["name"]) if p["name"] in order else len(order))
    return pieces

def build_gallery():
    pieces = gallery_pieces()
    cards, md_rows = [], []
    for p in pieces:
        n, info = p["name"], p["info"]
        title = (info or {}).get("title") or n
        desc = (info or {}).get("description") or ""
        # The README without its title line; its first heading is the page's.
        readme = re.sub(r"^# .*\n", "", p["readme"], count=1)
        readme_html, _ = markdown(readme) if readme else ("", [])
        media = (f'<video controls loop muted playsinline preload="none" poster="gallery/{n}/poster.png" '
                 f'src="gallery/{n}/{n}.mp4"></video>') if p["has_video"] else '<div class="missing">not rendered yet: scripts/build_gallery.sh</div>'
        facts = ""
        if info:
            r = p["render"] or {}
            facts = (f'<dl class="facts"><div><dt>Canvas</dt><dd>{info["width"]}×{info["height"]}</dd></div>'
                     f'<div><dt>Length</dt><dd>{info["duration"]:g} s · {info["frames"]} frames</dd></div>'
                     f'<div><dt>Mode</dt><dd>{info["mode"]}</dd></div>'
                     + (f'<div><dt>Rendered</dt><dd>{r["total_seconds"]:.1f} s on {r["threads"]} thread{"s" if r["threads"] != 1 else ""}</dd></div>' if r else "")
                     + "</dl>")
        source = highlight_ps(p["src"].read_text())
        lines = p["src"].read_text().count("\n")
        cards.append(f"""<article class="piece" id="{n}">
<div class="media">{media}</div>
<div class="about">
<h2>{html.escape(title)}</h2>
<p class="lede">{html.escape(desc)}</p>
{facts}
<div class="readme">{readme_html}</div>
<pre class="code cmd"><code>psanim render gallery/{n}/{n}.ps -o {n}.mp4</code></pre>
<p class="links"><a href="gallery/{n}/{n}.ps">source</a> · <a href="gallery/{n}/sheet.png">contact sheet</a> · <a href="gallery/{n}/poster.png">still</a> · <a href="gallery/{n}/{n}.mp4">mp4</a></p>
</div>
<details class="source"><summary>{n}.ps · {lines} lines of PostScript</summary><pre class="code lang-postscript"><code>{source}</code></pre></details>
</article>""")
        if info:
            md_rows.append(f"| [{title}](../gallery/{n}/{n}.ps) | {desc} | {info['duration']:g} s, {info['width']}x{info['height']}, {info['mode']} | `psanim render gallery/{n}/{n}.ps -o {n}.mp4` |")
    head = ('<section class="page-head"><h1>Gallery</h1><p class="lede">Seven films, each one PostScript file. '
            'Every frame you see is the program below it, run by the engine at that moment in time. '
            'Sources, stills and contact sheets are beside each film; the render command reproduces it.</p></section>')
    write("gallery.html", shell("gallery.html", "Gallery", head + "\n".join(cards), wide=True))
    # The same index as Markdown, for the repository.
    md = ["# Gallery", "",
          "Generated by `scripts/build_site.py` from each piece's declaration. The films themselves are",
          "rendered by `scripts/build_gallery.sh` into `site/gallery/` (not kept in git: they are",
          "reproducible from the sources).", "",
          "| Piece | What it is | Shape | Render |", "|---|---|---|---|", *md_rows, "",
          "Each piece's directory has a README describing its techniques.", ""]
    (ROOT / "docs/gallery.md").write_text("\n".join(md))
    return pieces

def build_home(pieces):
    body = part("home.html")
    hello = (ROOT / "examples/hello.ps").read_text()
    body = body.replace("{{HELLO}}", highlight_ps(hello))
    first = (ROOT / "docs/authoring.md").read_text()
    snippet = re.search(r"```postscript\n(%!PS\n.*?)```", first, re.S).group(1)
    body = body.replace("{{FIRST}}", highlight_ps(snippet))
    strip = "".join(
        f'<a class="tile" href="gallery.html#{p["name"]}"><img loading="lazy" src="gallery/{p["name"]}/poster.png" alt="">'
        f'<span>{html.escape((p["info"] or {}).get("title") or p["name"])}</span></a>' for p in pieces)
    body = body.replace("{{STRIP}}", strip)
    for name in ["architecture", "lifecycle", "timescopes"]:
        body = body.replace("{{" + name.upper() + "}}", part(name + ".svg.html"))
    body = body.replace("{{WORDS}}", str(len(API)))
    write("index.html", shell("index.html", "Programmable animation in PostScript", body, wide=True))

def build_playground():
    examples = {"Hello": (ROOT / "examples/hello.ps").read_text()}
    patterns = (ROOT / "skills/psanim/references/patterns.md").read_text()
    for title, block in re.findall(r"^## ([^\n]+)\n(?:(?!^## ).)*?```postscript\n(.*?)```", patterns, re.S | re.M):
        if not block.lstrip().startswith("%!PS"):
            block = ("%!PS\n<< /Size [640 360] /Duration 4 /FPS 30 /Background [0.05 0.06 0.09] >> animation\n\n/scene {\n"
                     + "\n".join("  " + l if l else l for l in block.rstrip("\n").split("\n")) + "\n} def\n")
        examples[title] = block
    examples["Pointer (move over the canvas)"] = part("pointer.ps")
    body = part("playground.html").replace("{{EXAMPLES}}", json.dumps(examples))
    write("playground.html", shell("playground.html", "Playground", body, wide=True))

def copy_assets():
    shutil.copy(ROOT / "scripts/site/site.css", SITE / "site.css")
    for f in ["psanim.js", "player.js"]:
        shutil.copy(ROOT / "web" / f, SITE / f)
    wasm = ROOT / "web/psanim.wasm"
    if wasm.exists():
        shutil.copy(wasm, SITE / "psanim.wasm")
    else:
        print("  (no web/psanim.wasm: run scripts/build_wasm.sh for the live player)")

def main():
    if not PSANIM.exists():
        sys.exit("build the CLI first: cargo build --release")
    SITE.mkdir(exist_ok=True)
    copy_assets()
    build_docs()
    build_reference()
    pieces = build_gallery()
    build_home(pieces)
    build_playground()
    pages = len(list(SITE.rglob("*.html")))
    print(f"site: {pages} pages in {SITE.relative_to(ROOT)}/ (serve with: python3 -m http.server -d site 8000)")

if __name__ == "__main__":
    main()
