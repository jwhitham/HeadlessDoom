
import subprocess, os

MODULE = "r_bsp"
local_only = set()
rust_only = set()
print("Shared:")
for name in """
seg_t*		curline;
side_t*		sidedef;
line_t*		linedef;
sector_t*	frontsector;
sector_t*	backsector;

drawseg_t	drawsegs[MAXDRAWSEGS];
drawseg_t*	ds_p;

cliprange_t*	newend;
cliprange_t	solidsegs[MAXSEGS];

""".split():
    if not name.endswith(";"):
        continue
    name = name.strip(";")
    if name.endswith("]"):
        i = name.find("[")
        if i > 0:
            name = name[:i]
    found = set()
    for side in ["headless_doom", "src"]:
        p = subprocess.Popen(["git", "grep", "-rlE", r"\<" + name + r"\>",
                                side], stdout=subprocess.PIPE,
                            text=True)
        (stdout, _) = p.communicate()
        p.wait()
        modules = set()
        for line in stdout.strip().splitlines():
            filename = os.path.basename(line.strip())
            (module, _) = os.path.splitext(filename)
            modules.add(module)
        modules.discard(MODULE)
        modules.discard("globals")
        modules.discard("bindings")

        if modules:
            found.add(side)
            print("  {} -> {} {}".format(name, side, modules))

    if found == set(["src"]):
        rust_only.add(name)

    if len(found) == 0:
        local_only.add(name)

print("Local only:")
for name in sorted(local_only):
    print("  ", name)

print("Rust only:")
for name in sorted(rust_only):
    print("  ", name)











