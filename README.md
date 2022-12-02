## rpy

Do you deal with lots of virtual conda environments? `rpy` is for you!

Before `rpy`:

```
~/dev/some/project$ env PYTHONPATH=src/py path/to/my/interpreter path/to/my/script.py workplz 
```

After:

```
~/dev/some/project$ py path/to/my/script.py workplz
```

### The magic

`rpy` looks for a `pyproject.toml` file relating to the script, and then looks for a `tool.py` section of the form:

```toml
[tool.rpy]
interpreter = 'out/env/bin/python'  # path relative to the project root for the python interpreter to use
source_root = 'src/py'                     # PYTHONPATH to set up
pre_run = 'make -q deps'                   # Optional command to run in the project root first
```
