## rpy

Do you deal with lots of virtual conda environments? `rpy` is for you!

Before `rpy`:

```
~/dev/some/project$ env PYTHONPATH=src/py path/to/my/interpreter path/to/my/script.py --my --args here
```

After:

```
~/dev/some/project$ rpy path/to/my/script.py  --my --args here
```

### The magic

`rpy` looks for a `pyproject.toml` file relating to the script, and then looks for a `tool.py` section of the form:

```toml
[tool.rpy]
# All paths are relative to the project root (which is wherever we found the pyproject.toml
interpreter = 'out/env/bin/python'  # path relative to the project root
source_root = 'src/py'              # PYTHONPATH to set up
pre_run = 'make -q deps'            # Optional command to run in the project root first
```
