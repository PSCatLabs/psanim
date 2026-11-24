# psanim — Project Initialization

## Mission

Build psanim, an open-source, programmable animation engine powered by PostScript and the PSCat graphics engine. It should enable humans and AI agents to author expressive animations, motion graphics, mathematical visualizations, short films, and dynamic presentations as executable programs rather than through a graphical timeline editor.

This is a greenfield project. You have substantial freedom to investigate, experiment, choose abstractions, and implement the system. Treat this document as a statement of intent, not a prescribed architecture. Challenge assumptions when evidence warrants it, and document consequential decisions.

## 1. Vision

PostScript is already a rich programmable graphics language. Extend its expressive power into the temporal domain while preserving its procedural nature, composability, graphics semantics, and existing ecosystem of drawing, typography, and painting capabilities.

We want a system that is:

- Expressive: capable of sophisticated, aesthetically compelling animation, not merely moving geometric primitives.
- Programmable: animation is source code that can be generated, inspected, composed, versioned, tested, and revised.
- Performant: leverage Rust, efficient rendering, caching, and parallelism where appropriate. Measure performance rather than assuming it.
- Portable: design for native execution and WebAssembly, with a credible path to browser playback and embedding.
- Agent-native: make authoring, previewing, diagnosing, evaluating, and refining animations straightforward for coding agents.
- Extensible: support future applications and rendering models without forcing speculative abstractions into the initial implementation.

The ambition is not simply to reproduce an existing animation package in Rust. Explore what becomes possible when a general-purpose graphics language, an optimized rendering engine, and an agentic creative workflow are designed to work together.

## 2. Relationship to PSCat

psanim is a separate repository that depends on reusable libraries exported by PSCat (https://github.com/PSCatLabs/pscat).

Begin by investigating the actual PSCat repository, its workspace, public interfaces, interpreter, rendering pipeline, painting libraries, existing tests, documentation, and conventions. Locate any existing local checkout before cloning another copy. Do not assume its architecture from this brief.

A major objective of this project is to help evolve PSCat from a standalone interpreter and rendering application into a reusable graphics platform for multiple independent applications.

### PSCat feature branch and upstream integration

Before making any changes to PSCat, create and check out a dedicated feature branch, suggested name `feature/psanim-library-api`, based on the appropriate current upstream development branch. Inspect the repository status and branch conventions first. If the existing checkout contains unrelated work, use a separate Git worktree or checkout rather than disturbing it.

- Make all PSCat API additions, refactoring, and supporting tests on this feature branch, not directly on the default or release branch.
- Develop psanim against this branch using a reproducible local development dependency arrangement. Record the upstream base, branch name, and tested commit revisions, and document how to build both repositories together.
- Commit PSCat changes separately from psanim changes, in coherent, reviewable increments. Preserve compatibility and run PSCat's existing regression tests alongside new API and integration tests.
- Treat the PSCat changes as upstream-quality contributions intended to be reviewed and merged into a future PSCat release. Avoid permanent forks, animation-specific coupling, or temporary hacks that would obstruct that merge.
- Document the proposed public API, compatibility implications, and eventual migration of psanim to a released PSCat dependency. Leave the feature branch available for review; do not merge it into the default branch, push it remotely, or publish a release without explicit authorization.


You are authorized to make appropriate changes to PSCat in support of this objective. In particular:

- Prefer direct Rust library integration over invoking the PSCat CLI as a subprocess.
- Identify and improve public APIs for interpreter lifecycle, execution, graphics state, rendering, extension registration, resource management, and diagnostics where needed.
- Consider separation of interpretation, graphics semantics, rendering, and host integration where justified by the existing implementation.
- Keep animation-specific policy in psanim. PSCat should not acquire a dependency on psanim.
- Make reusable abstractions general enough for other applications, including applications with external inputs, alternative coordinate systems, rendering targets, or execution models.
- Preserve existing PSCat functionality and compatibility. Add regression tests and avoid unrelated rewrites.
- Prefer incremental, evidence-driven refactoring over a speculative proliferation of crates or a wholesale architectural rewrite.

Use psanim as a real consumer that validates the quality of PSCat's public APIs. Document integration points, dependency versions, and how to build both repositories together. Keep changes to each repository independently reviewable. Do not publish crates, push changes, or modify unrelated work without authorization.

## 3. Animation model

Investigate the best way to express time, motion, composition, and animation in PostScript. Consider existing animation systems such as Manim, Processing, Motion Canvas, and relevant shader or procedural graphics systems as architectural references, not templates to copy.

A simple conceptual starting point is a PostScript procedure that renders a scene at a supplied time. This is an example, not a mandated API:

```postscript
/scene {
    /t exch def
    gsave
        320 240 translate
        t 90 mul rotate
        0.9 0.3 0.1 setrgbcolor
        -50 -50 100 100 rectfill
    grestore
} def
```

Explore appropriate abstractions for timelines, sequencing, interpolation, easing, keyframes, path-following, transformations, transitions, composition, and reusable animation procedures. Decide which capabilities belong in ordinary PostScript libraries and which require native extensions.

Favor deterministic, time-addressable rendering where practical. An agent should be able to inspect an arbitrary frame without rendering an entire film first. However, do not exclude stateful simulations, feedback effects, persistent drawing, or other history-dependent animations. Make their execution semantics explicit, and consider replay or checkpointing when appropriate.

Think about external time-varying signals and discrete events, host-supplied clocks, live inputs, and interactive playback. Avoid hardwiring the engine to a fixed offline frame loop. Define clear semantics for time, frame indices, sampling, random seeds, and state.

Do not require a retained scene graph unless it earns its complexity. Preserve the ability to use procedural PostScript directly.

## 4. Runtime and outputs

Develop a coherent Rust library and a practical CLI. Establish an end-to-end workflow for authoring an animation, previewing selected frames, and producing a playable movie.

Desired capabilities include:

- Rendering a frame at a requested timestamp.
- Rendering a range of frames and exporting a standard video format, preferably MP4 or WebM.
- Generating contact sheets or storyboards of selected frames.
- Configurable duration, resolution, frame rate, and random seed.
- Useful progress reporting, structured errors, and machine-readable metadata.
- A credible WASM integration path, ideally demonstrated through a working browser player or preview.

Choose appropriate encoding and rendering dependencies. Keep platform-specific video encoding separate from the portable rendering core. Investigate efficient resource reuse, caching, frame parallelism, and GPU acceleration where warranted. Benchmark representative workloads and document limitations.

Do not claim support for an output format or platform unless it has actually been exercised.

## 5. Agent-native workflows

AI agents are primary users of this project, not an afterthought. Design the authoring and evaluation loop around their needs:

1. Understand the animation API and available drawing libraries.
2. Generate or modify a PostScript animation program.
3. Validate the program and render inexpensive previews.
4. Inspect representative frames, contact sheets, diagnostics, and motion.
5. Refine the program and render the final artifact.

Favor stable, discoverable interfaces, concise diagnostics, reproducible commands, and bounded execution. Consider machine-readable animation metadata, source locations for errors, visual assertions, and efficient inspection of long animations. Explore an MCP integration if it adds value, but do not let it block the core engine.

Treat generated PostScript as potentially untrusted. Investigate execution budgets, memory limits, cancellation, filesystem and network access, and safe handling of external resources. Document the actual security guarantees and limitations, especially for embedded or hosted use.

## 6. Build an impressive animation gallery

The gallery is a first-class deliverable and an architectural test suite. Do not stop at bouncing balls, rotating squares, or elementary demonstrations. Create a curated collection of genuinely compelling animations that exercise different aspects of the system.

Possible directions (use your own artistic judgment):

- Cinematic kinetic typography and animated title sequences.
- A mathematical or scientific explanation that unfolds visually over time.
- Procedural generative art with intricate, evolving geometry and color.
- An animated painting using PSCat's existing artistic media and brush capabilities.
- A short visual narrative with multiple scenes, transitions, and deliberate pacing.
- A sophisticated looping animation or particle-based composition.
- A dynamic technical presentation that communicates a complex idea through motion.

Aim for variety in visual style, motion, composition, and technical complexity. Use the gallery to discover missing abstractions and improve the engine. Prioritize a smaller collection of excellent pieces over a large collection of trivial examples.

Each gallery piece should include its editable PostScript source, reproducible rendering instructions, relevant metadata, representative stills or contact sheets, and a playable animation where practical. Explain the techniques used. Use original or appropriately licensed assets.

Actually render and inspect the results. Evaluate motion and temporal continuity, not just attractive individual frames. Iterate on the artistic quality as well as technical correctness. The gallery should demonstrate what psanim can do today, not what we hope it will do later.

## 7. Documentation for humans and agents

Documentation is part of the product. Build it alongside the implementation.

### Human-facing HTML documentation

Create a polished, browsable documentation and gallery site. Include architecture diagrams, conceptual explanations, syntax-highlighted examples, getting-started instructions, API guidance, and embedded animation previews. Use diagrams to explain the relationship between PSCat and psanim, the execution model, the animation lifecycle, and rendering/export paths.

Where practical, demonstrate interactive examples or a browser-based player. Make the site easy to serve locally and deploy as static documentation. It should be useful to someone encountering the project for the first time.

### Markdown documentation

Maintain version-controlled Markdown documentation suitable for repository browsing and agent consumption. Include:

- README with the project's purpose, quick start, and compelling examples.
- Architecture and design rationale, including significant tradeoffs and rejected alternatives.
- Animation authoring guide and language/extension reference.
- Rust integration and embedding guide.
- CLI reference, build instructions, and WASM status.
- Gallery index with reproducible commands.
- Testing, benchmarking, and contribution guidance.
- Roadmap and known limitations.

Avoid duplicating large bodies of documentation across formats unnecessarily. Prefer a maintainable source of truth and generated views where appropriate.

### Agent skills

Create reusable, discoverable agent skills using SKILL.md files and supporting references/examples. They should teach agents how to author psanim programs, select appropriate animation abstractions, render and inspect previews, troubleshoot failures, and produce polished final animations.

Include practical workflows, tested commands, common mistakes, and guidance on artistic quality. Keep the main skill instructions concise and move detailed reference material into supporting files. Validate the skills by using them to create or refine gallery pieces. Do not merely repackage the README as a skill.

## 8. Engineering approach

Start with repository reconnaissance and a concise design/implementation plan. Then proceed autonomously through implementation and experimentation; do not stop after producing a proposal unless genuinely blocked.

Establish an early vertical slice that exercises PSCat as a library, executes a time-dependent PostScript program, renders frames, and produces a playable animation. Use that slice to validate API boundaries before investing heavily in abstractions.

Work iteratively. Add unit, integration, and regression tests for animation semantics, deterministic rendering, PSCat integration, and failure cases. Use gallery examples as end-to-end tests where practical. Include performance measurements and inspect rendered artifacts.

Follow existing PSCat conventions where appropriate and establish sensible conventions for this new repository. Initialize version control, keep changes focused, and record decisions and progress so another human or agent can resume the work. Avoid destructive operations and preserve unrelated local changes.

Make reasonable decisions independently. Ask for clarification only when a genuinely consequential ambiguity or external blocker prevents meaningful progress. If a dependency, platform capability, or assumption proves unsuitable, investigate alternatives and explain the tradeoff.

## 9. Initial success criteria

The initial project should leave us with:

- A working psanim Rust library and CLI, integrated with PSCat through reusable APIs.
- Appropriate, tested PSCat API improvements that benefit independent consumers.
- A coherent, documented animation programming model.
- Reproducible frame rendering, contact sheets, and playable video output.
- A demonstrated WASM/browser integration where feasible, or a concrete, tested account of remaining blockers.
- An impressive gallery of original animations with source code and reproducible outputs.
- Human-facing HTML documentation with diagrams and animation previews.
- Markdown technical documentation and practical agent skills.
- Automated tests, representative benchmarks, and an honest account of limitations and next steps.

These are outcome-oriented goals, not a fixed implementation checklist. Adjust the scope intelligently as you learn, but preserve the core intent: programmable animation through PostScript, a reusable PSCat graphics platform, high-quality creative output, and an excellent experience for both humans and AI agents.

Build something that makes us excited about what this platform can become.
