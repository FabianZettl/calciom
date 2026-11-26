#!/usr/bin/env python3
# Prüfbild: pruefbild.py out.png KEY=VAL ...  (CALCIOM_PAGE, CALCIOM_THEME, CALCIOM_SPRACHE, CALCIOM_WELT, CALCIOM_VEREIN …)
# The CalcioM window is made floating at its own size (1400x900) right after it opens, because a full, tiled
# Hyprland workspace would squeeze it; retries while the picture still comes out narrow. Careers go to a fresh
# temporary folder, never the person's.
import json, os, struct, subprocess, sys, tempfile, time
out = sys.argv[1]
env = dict(os.environ)
for kv in sys.argv[2:]:
    k, v = kv.split("=", 1); env[k] = v
env["CALCIOM_KARRIEREN"] = tempfile.mkdtemp(prefix="pesm-shot-")
app = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
exe = os.path.join(app, "target/release/calciom")
def hc(*a): return subprocess.run(["hyprctl", *a], capture_output=True, text=True).stdout
for versuch in range(6):
    if os.path.exists(out): os.unlink(out)
    p = subprocess.Popen([exe, "--screenshot", out], env=env, cwd=app, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    t0 = time.time()
    while p.poll() is None and time.time() - t0 < 300:
        try: cl = json.loads(hc("clients", "-j"))
        except Exception: cl = []
        w = [c for c in cl if c.get("pid") == p.pid]
        if w:
            if not w[0].get("floating"): hc("dispatch", 'hl.dsp.window.float({window="address:%s"})' % w[0]["address"])
            time.sleep(0.3)  # the resize only holds once the window is floating
            fx, fy = env.get("CALCIOM_FENSTER", "1400x900").split("x")
            hc("dispatch", 'hl.dsp.window.resize({window="address:%s", x=%s, y=%s})' % (w[0]["address"], fx, fy))
            break
        time.sleep(0.05)
    p.wait(timeout=600)
    if os.path.exists(out) and struct.unpack(">I", open(out, "rb").read()[16:20])[0] >= int(env.get("CALCIOM_FENSTER", "1001x").split("x")[0]) - 1: print(out); sys.exit(0)
print("kein breites Bild", out); sys.exit(1)
