# Layout

Layout is a rust library and a stand alone utility that draws graphs. Layout
can parse Graphviz dot files and render them.

## Getting started

Build the crate and render some dot files with the command

```bash
crate run --bin run ./inputs/bk.dot -o output.svg
```

## Gallery

This section presents a few graphs that were rendered from dot files:

A simple graph.

![](docs/graph.png)

A simple graph with multiple shapes and labels.

![](docs/graph2.png)

A graph with a few style properties.

![](docs/colors.png)

A large graph that demonstrates the edge crossing elimination optimization.

![](docs/bk.png)

Unicode, emoji and left-to-right languages:

![](docs/heb.png)

Support for Records (nested structures):

![](docs/records.png)

