#!/usr/bin/env python3
# Makes the match and reward sounds from nothing but maths, so they carry no outside licence:
# tor.wav (the crowd rises), karte.wav (two short whistle blasts), elfmeter_vorbei.wav (a disappointed murmur),
# anpfiff.wav / abpfiff.wav (referee whistle), sieg.wav (chord and crowd), niederlage.wav (falling chord),
# meilenstein.wav (rising chord), pressefrage.wav (camera shutters), and the interface sounds klick, bestaetigt,
# tagesabschluss, fehler, posteingang, angebot. Tones are soft mallet notes (marimba-like: a sine that dies away
# quickly, a faint 4th partial for the wooden knock), no long pure tones, so nothing sounds like an arcade synth.
# Usage: klaenge.py  (writes into assets/sfx/; the same file every time, the noise has a fixed seed)
import math, os, random, struct, wave
APP = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RATE = 22050

def schreiben(name, samples, laut=0.8):
    peak = max(1e-9, max(abs(x) for x in samples))
    with wave.open(os.path.join(APP, "assets/sfx", name), "wb") as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(RATE)
        w.writeframes(b"".join(struct.pack("<h", int(x / peak * laut * 32767)) for x in samples))

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
    out = []; atem = rauschen(RATE, 0.5, 5)
    for von, bis in [(0.0, 0.18), (0.26, 0.62)]:
        while len(out) < int(RATE * von): out.append(0.0)
        for i in range(int(RATE * (bis - von))):
            t = i / RATE
            f = 2900 + 120 * math.sin(2 * math.pi * 28 * t)     # the pea in the whistle makes it trill
            hull = min(1.0, t / 0.01, (bis - von - t) / 0.02)
            out.append(hull * (math.sin(2 * math.pi * f * t) + 0.5 * atem[i % len(atem)]))
    return out

def elfmeter_vorbei():
    n = int(RATE * 1.4); z = rauschen(n, 0.05, 2)
    return [z[i] * math.sin(math.pi * min(1.0, i / n)) + 0.15 * math.sin(2 * math.pi * (180 - 60 * i / n) * i / RATE) * (1 - i / n) for i in range(n)]

def pfiff(laengen):
    # referee whistle blasts (anpfiff: one long; abpfiff: two short, one long)
    out = []; atem = rauschen(RATE, 0.5, 4)
    for von, dauer in laengen:
        while len(out) < int(RATE * von): out.append(0.0)
        for i in range(int(RATE * dauer)):
            t = i / RATE
            f = 3000 + 140 * math.sin(2 * math.pi * 30 * t)
            # a bit of breath noise so it sounds like a whistle, not a beep
            out.append(min(1.0, t / 0.01, (dauer - t) / 0.03) * (math.sin(2 * math.pi * f * t) + 0.5 * atem[i % len(atem)]))
    return out

def anpfiff(): return pfiff([(0.0, 0.7)])
def abpfiff(): return pfiff([(0.0, 0.2), (0.3, 0.2), (0.6, 0.9)])

def note(f, dauer=0.5, abklingen=7.0):
    # one soft mallet note: 6 ms attack, exponential decay, a faint wooden 4th partial that dies even faster
    out = []
    for i in range(int(RATE * dauer)):
        t = i / RATE
        hull = min(1.0, t / 0.006) * math.exp(-abklingen * t)
        out.append(hull * (math.sin(2 * math.pi * f * t) + 0.12 * math.exp(-30 * t) * math.sin(2 * math.pi * 4 * f * t)))
    return out

def folge(toene, abstand, dauer=0.6, abklingen=7.0):
    # notes one after another, each ringing out under the next
    n = int(RATE * (abstand * (len(toene) - 1) + dauer)); out = [0.0] * n
    for k, f in enumerate(toene):
        for i, x in enumerate(note(f, dauer, abklingen)): out[int(RATE * abstand * k) + i] += x
    return out

C4, E4, G4, A4, C5, D5, E5, G5, A5, C6 = 261.63, 329.63, 392.0, 440.0, 523.25, 587.33, 659.25, 783.99, 880.0, 1046.5

def sieg(): return [a + 0.25 * b for a, b in zip(folge([C5, E5, G5, C6], 0.11, 0.9, 4.5), tor()[:int(RATE * 1.23)] + [0.0] * 99999)]
def niederlage(): return folge([G4, E4, C4], 0.2, 0.8, 4.0)
def meilenstein(): return folge([C5, E5, G5, C6], 0.08, 0.7, 5.0)
def klick():
    # a soft wooden tap, 40 ms
    return [math.exp(-160 * i / RATE) * math.sin(2 * math.pi * 1100 * i / RATE) for i in range(int(RATE * 0.04))]
def bestaetigt(): return folge([G4, C5], 0.08, 0.4, 9.0)
def tagesabschluss(): return folge([C5, E5, G5], 0.1, 0.6, 6.0)
def fehler(): return folge([E4, C4], 0.12, 0.35, 10.0)
def posteingang(): return note(A5, 0.6, 6.0)
def angebot(): return folge([D5, G5], 0.12, 0.5, 7.0)
def pressefrage():
    # camera shutters: short clicks of filtered noise
    z = rauschen(int(RATE * 0.9), 0.6, 3); out = []
    for i in range(len(z)):
        t = i / RATE; k = min(abs(t - x) for x in (0.05, 0.22, 0.31, 0.55))
        out.append(z[i] * math.exp(-k * 90))
    return out

if __name__ == "__main__":
    # the third value is the peak volume: whistles and interface sounds quieter than the goal
    for name, f, laut in [("tor.wav", tor, 0.8), ("karte.wav", karte, 0.4), ("elfmeter_vorbei.wav", elfmeter_vorbei, 0.7), ("anpfiff.wav", anpfiff, 0.4),
                    ("abpfiff.wav", abpfiff, 0.4), ("sieg.wav", sieg, 0.7), ("niederlage.wav", niederlage, 0.6), ("meilenstein.wav", meilenstein, 0.6),
                    ("pressefrage.wav", pressefrage, 0.6), ("klick.wav", klick, 0.35), ("bestaetigt.wav", bestaetigt, 0.5),
                    ("tagesabschluss.wav", tagesabschluss, 0.55), ("fehler.wav", fehler, 0.5), ("posteingang.wav", posteingang, 0.5), ("angebot.wav", angebot, 0.5)]:
        s = f(); assert len(s) > RATE // 40, name
        schreiben(name, s, laut); print(name, round(len(s) / RATE, 2), "s")
