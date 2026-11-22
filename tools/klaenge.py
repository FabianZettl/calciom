#!/usr/bin/env python3
# Makes the match and reward sounds from nothing but maths, so they carry no outside licence:
# tor.wav (the crowd rises), karte.wav (two short whistle blasts), elfmeter_vorbei.wav (a disappointed murmur),
# anpfiff.wav / abpfiff.wav (referee whistle), sieg.wav (chord and crowd), niederlage.wav (falling chord),
# meilenstein.wav (rising chord), pressefrage.wav (camera shutters).
# Usage: klaenge.py  (writes into assets/sfx/; the same file every time, the noise has a fixed seed)
import math, os, random, struct, wave
APP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RATE = 22050

def schreiben(name, samples):
    peak = max(1e-9, max(abs(x) for x in samples))
    with wave.open(os.path.join(APP, "assets/sfx", name), "wb") as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(RATE)
        w.writeframes(b"".join(struct.pack("<h", int(x / peak * 0.8 * 32767)) for x in samples))

def rauschen(n, glatt, seed):
    # low-passed noise: a crowd is many voices, no single tone
    r = random.Random(seed); y = 0.0; out = []
    for _ in range(n):
        y += glatt * (r.uniform(-1, 1) - y); out.append(y)
    return out

def tor():
    n = int(RATE * 1.8); z = rauschen(n, 0.12, 1)
    return [z[i] * min(1.0, i / (RATE * 0.35)) * (1.0 if i < RATE * 1.2 else max(0.0, 1 - (i - RATE * 1.2) / (RATE * 0.6)))
            * (1 + 0.25 * math.sin(2 * math.pi * 3.0 * i / RATE)) for i in range(n)]

def karte():
    out = []
    for von, bis in [(0.0, 0.18), (0.26, 0.62)]:
        while len(out) < int(RATE * von): out.append(0.0)
        for i in range(int(RATE * (bis - von))):
            t = i / RATE
            f = 2900 + 120 * math.sin(2 * math.pi * 28 * t)     # the pea in the whistle makes it trill
            hull = min(1.0, t / 0.01, (bis - von - t) / 0.02)
            out.append(hull * math.sin(2 * math.pi * f * t))
    return out

def elfmeter_vorbei():
    n = int(RATE * 1.4); z = rauschen(n, 0.05, 2)
    return [z[i] * math.sin(math.pi * min(1.0, i / n)) + 0.15 * math.sin(2 * math.pi * (180 - 60 * i / n) * i / RATE) * (1 - i / n) for i in range(n)]

def pfiff(laengen):
    # referee whistle blasts (anpfiff: one long; abpfiff: two short, one long)
    out = []
    for von, dauer in laengen:
        while len(out) < int(RATE * von): out.append(0.0)
        for i in range(int(RATE * dauer)):
            t = i / RATE
            f = 3000 + 140 * math.sin(2 * math.pi * 30 * t)
            out.append(min(1.0, t / 0.01, (dauer - t) / 0.03) * math.sin(2 * math.pi * f * t))
    return out

def anpfiff(): return pfiff([(0.0, 0.7)])
def abpfiff(): return pfiff([(0.0, 0.2), (0.3, 0.2), (0.6, 0.9)])

def akkord(toene, dauer, abstand=0.0):
    # a short chord of sine tones with soft overtones; with abstand the notes come one after another (arpeggio)
    n = int(RATE * (dauer + abstand * len(toene))); out = [0.0] * n
    for k, f in enumerate(toene):
        s0 = int(RATE * abstand * k)
        for i in range(n - s0):
            t = i / RATE
            hull = min(1.0, t / 0.015) * math.exp(-2.2 * t / dauer)
            out[s0 + i] += hull * (math.sin(2 * math.pi * f * t) + 0.3 * math.sin(4 * math.pi * f * t))
    return out

def sieg(): return [a + 0.35 * b for a, b in zip(akkord([523.25, 659.25, 783.99, 1046.5], 0.9, 0.09), tor()[:int(RATE * 1.26)] + [0.0] * 99999)]
def niederlage(): return akkord([392.0, 311.13, 261.63], 0.8, 0.16)
def meilenstein(): return akkord([659.25, 783.99, 987.77, 1318.5], 0.7, 0.07)
def pressefrage():
    # camera shutters: short clicks of filtered noise
    z = rauschen(int(RATE * 0.9), 0.6, 3); out = []
    for i in range(len(z)):
        t = i / RATE; k = min(abs(t - x) for x in (0.05, 0.22, 0.31, 0.55))
        out.append(z[i] * math.exp(-k * 90))
    return out

if __name__ == "__main__":
    for name, f in [("tor.wav", tor), ("karte.wav", karte), ("elfmeter_vorbei.wav", elfmeter_vorbei), ("anpfiff.wav", anpfiff), ("abpfiff.wav", abpfiff),
                    ("sieg.wav", sieg), ("niederlage.wav", niederlage), ("meilenstein.wav", meilenstein), ("pressefrage.wav", pressefrage)]:
        s = f(); assert len(s) > RATE // 2, name
        schreiben(name, s); print(name, round(len(s) / RATE, 2), "s")
