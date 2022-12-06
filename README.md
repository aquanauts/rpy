## rpy

Do you deal with lots of virtual conda environments? `rpy` is for you!

Before `rpy`:

```
~/dev/prj$ env PYTHONPATH=src/py path/to/my/interpreter src/py/my/script.py --my --args here
```

After:

```
~/dev/prj$ rpy src/py/my/script.py  --my --args here
```

### The magic

`rpy` looks for a `pyproject.toml` file relating to the script, and then looks for a `tool.py` section of the form:

```toml
[tool.rpy]
# All paths are relative to the project root (which is wherever we found the pyproject.toml
interpreter = 'out/env/bin/python'  # path relative to the project root
source_root = 'src/py'              # Optioanl PYTHONPATH to set up (defaults to project root)
pre_run = 'make -q deps'            # Optional command to run in the project root first
```
