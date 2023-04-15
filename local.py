
import subprocess, os

MODULE = "r_segs"
local_only = set()
rust_only = set()
print("Shared:")
for name in """
boolean		segtextured;	
boolean		markfloor;	
boolean		markceiling;
boolean		maskedtexture;
int		toptexture;
int		bottomtexture;
int		midtexture;
angle_t		rw_normalangle;
int		rw_angle1;	
int		rw_x;
int		rw_stopx;
angle_t		rw_centerangle;
fixed_t		rw_offset;
fixed_t		rw_distance;
fixed_t		rw_scale;
fixed_t		rw_scalestep;
fixed_t		rw_midtexturemid;
fixed_t		rw_toptexturemid;
fixed_t		rw_bottomtexturemid;
int		worldtop;
int		worldbottom;
int		worldhigh;
int		worldlow;
fixed_t		pixhigh;
fixed_t		pixlow;
fixed_t		pixhighstep;
fixed_t		pixlowstep;
fixed_t		topfrac;
fixed_t		topstep;
fixed_t		bottomfrac;
fixed_t		bottomstep;
lighttable_t**	walllights;
short*		maskedtexturecol;

""".split():
    if not name.endswith(";"):
        continue
    name = name.strip(";")
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











