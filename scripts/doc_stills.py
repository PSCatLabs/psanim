#!/usr/bin/env python3
"""Render every runnable `postscript` fence of a Markdown file to a contact
sheet, to look at what the documentation's examples actually draw.

  scripts/doc_stills.py skills/psanim/references/stage.md out/doc-stills [--count 4]

Writes out/doc-stills/<file>-<line>.png, one sheet per fence. Uses the
same wrapping as crates/psanim/tests/docs.rs.
"""
import pathlib, subprocess, sys

LIBS = "pkstage pkmotion pktype pkdiagram pkcode pkchart pkink pkfigure pkpixel".split()

def fences(text):
    out, cur = [], None
    for n, line in enumerate(text.splitlines()):
        t = line.lstrip()
        if cur is None:
            if t.startswith("```postscript") and "partial" not in t:
                cur = (n + 2, [])
        elif t.startswith("```"):
            out.append((cur[0], "\n".join(cur[1]) + "\n"))
            cur = None
        else:
            cur[1].append(line)
    return out

def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    count = "4"
    if "--count" in sys.argv:
        count = sys.argv[sys.argv.index("--count") + 1]
        args.remove(count)
    src, out = pathlib.Path(args[0]), pathlib.Path(args[1])
    out.mkdir(parents=True, exist_ok=True)
    root = pathlib.Path(__file__).resolve().parent.parent
    psanim = root / "target/release/psanim"
    for line, body in fences(src.read_text()):
        if not body.lstrip().startswith("%!PS"):
            use = " ".join(f"/{l} use" for l in LIBS)
            body = (
                "<< /Size [640 360] /Duration 4 /FPS 30 /Background [0.05 0.06 0.09] >> animation\n"
                f"{use}\n/scene {{\n{body}\n}} def\n"
            )
        ps = out / f"{src.stem}-{line}.ps"
        ps.write_text(body)
        png = out / f"{src.stem}-{line}.png"
        r = subprocess.run(
            [str(psanim), "sheet", str(ps), "--count", count, "--cols", count, "-o", str(png)],
            capture_output=True, text=True,
        )
        print(png if r.returncode == 0 else f"FAILED {src}:{line}\n{r.stderr}")
        ps.unlink()

if __name__ == "__main__":
    main()
