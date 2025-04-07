# lx - Lamina Build Tool

A build tool for Lamina projects with support for project creation, initialization, building, and running Lamina scripts.

## Features

- Create new Lamina projects
- Initialize Lamina in existing directories
- Build Lamina projects with different backends
- Run Lamina scripts

## Installation

```
cargo install lx
```

## Usage

```
# Create a new project
lx new my-project

# Initialize in current directory
lx init

# Build the project
lx build

# Build with a specific target
lx build --target huff

# Run a script
lx run script.lam
``` 