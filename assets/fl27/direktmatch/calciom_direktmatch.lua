-- CalcioM: Direktstart in eine Partie, ohne Hauptmenue (nur Speicher + livecpk, FL_2027.exe bleibt unveraendert).
-- Schalter: flw\direktmatch.txt mit "heim gast" (Team-IDs aus Team.bin), z. B. "396 179".
--   Zeile da   -> Ablaufdatei EditDataLoadProcess.json springt nach Match/MatchDirectSetup, die festen Teams
--                 108/52 des Knotens werden ersetzt, danach wird die Zeile zu "# benutzt: ..." (einmalig:
--                 der naechste Start ohne neue Zeile geht wieder ins Hauptmenue).
--   keine Zeile -> Ablaufdatei zeigt wie das Original auf TopMenu/TopMenu, normales Spiel.
-- Fundstelle: ProcessMatchSetup (Direct) ruft 0x141522c30(code, side, 0) mit 0x1b0000 / 0xd0000; code = id<<14 | Platz (0).
local m = { version = "1.29" }

-- Ablauf im Spiel ohne Tastendruck: Nur im Direktstart schiebt das Modul eigene Ablauf-Dateien unter
-- (content\calciom-direktmatch\flow\Match\*.json, gebaut von tools/flow_bauen.py): Halbzeit- und End-Statistik
-- sowie Halbzeit- und Ergebnis-Menue werden uebersprungen, sonst wartet das Spiel dort auf eine Taste.
local flow_dir = nil
local MENUE = nil   -- Weg "menue": {heim, gast, minuten, gesetzt}
local function menue_teams_setzen()
    -- wie ProcessMatchSetup (Direkt): 0x141522c30(code, seite, 0); Spieldauer S+0x1c
    local setze = ffi.cast("void(*)(uint32_t, uint8_t, uint8_t)", 0x141522c30)
    setze(MENUE.heim * 16384, 0, 0)
    setze(MENUE.gast * 16384, 1, 0)
    if MENUE.minuten and MENUE.minuten > 0 then
        local g = ffi.cast("uint64_t*", 0x143705e10)[0]
        local s = ffi.cast("uint64_t*", g + 0x50)[0] + 0x35960
        ffi.cast("uint8_t*", s + 0x1c)[0] = MENUE.minuten
    end
    MENUE.gesetzt = true
    log(string.format("pesm_direktmatch: MENUE Teams %d/%d gesetzt", MENUE.heim, MENUE.gast))
end
local function datei_da(p)
    local f = io.open(p, "rb")
    if f then f:close() return true end
    return false
end
-- Schnellstart: naechstes Spiel aus direktmatch.txt (von CalcioM nach dem Abpfiff geschrieben), nur Speicher-Schreiben
function naechstes_spiel()
    -- Schnellstart: neue Werte/Aufstellungen von CalcioM laden wie F7 im LivePatch-Overlay (falls das Modul das anbietet)
    if type(pesm_livepatch_laden) == "function" then
        local ok, e = pcall(pesm_livepatch_laden)
        log("pesm_direktmatch: SCHNELLSTART LivePatch geladen " .. (ok and "ok" or tostring(e)))
    end
    local p = ctx_root .. "flw\\direktmatch.txt"
    local heim, gast, kh, kg, minuten = lese_teams(p)
    if not heim then
        log("pesm_direktmatch: SCHNELLSTART keine neue Zeile in direktmatch.txt - gleiche Teams wie zuvor")
        return
    end
    MENUE.heim, MENUE.gast, MENUE.kh, MENUE.kg, MENUE.minuten = heim, gast, kh, kg, minuten
    menue_hoehlen(false)
    local f = io.open(p, "wb")
    if f then f:write(string.format("# benutzt: %d %d\r\n", heim, gast)) f:close() end
    log(string.format("pesm_direktmatch: SCHNELLSTART naechstes Spiel %d gegen %d", heim, gast))
end

local function flow_key(ctx, filename)
    local bereich, name = (filename or ""):match("script[\\/]flow[\\/](%w+)[\\/]([%w_]+)%.json$")
    if not bereich then return end
    log("pesm_direktmatch: FLOW " .. bereich .. "/" .. name)
    if MENUE and MENUE.cave and bereich == "Exhibition" and name == "ExhibiSideSetting" then
        MENUE.seitenwahl = MENUE.seitenwahl + 1
    end
    -- Schnellstart: erst beim VERLASSEN der Seitenwahl neu lesen (CalcioM schreibt das naechste Spiel oft erst Tage
    -- spaeter, kurz bevor der Spieler die Seite bestaetigt); die Team-Hoehlen dieses Zustands laufen danach
    if MENUE and MENUE.cave and MENUE.seitenwahl > 1 and bereich == "Exhibition" and name == "ExhibiChangeCompeDesignKind" then
        naechstes_spiel()
    end
    -- edit=0 (CalcioM 26.11.): ohne die Edit-Daten (EDIT00000000) starten, sie ueberschrieben die Karriere-Datenbank.
    -- Nach dem Installationsschritt direkt weiter, an Edit/Load/EditDataLoadProcess vorbei.
    if MENUE and REGELN.edit == 0 and flow_dir and bereich == "Intro" and name == "ProcInstall" and datei_da(flow_dir .. "Ohne\\ProcInstall.json") then
        log("pesm_direktmatch: OHNE EDIT-DATEN (Intro/ProcInstall -> Exhibition/ExhibiFlowInit)")
        return "pesm_flow_Ohne_ProcInstall"
    end
    if MENUE and REGELN.schnell == 1 and flow_dir and ((bereich == "Exhibition" and name == "ExhibiFlowEnd") or
            (bereich == "Match" and name == "MatchEnd")) then
        log("pesm_direktmatch: SCHNELLSTART nach dem Spiel zurueck in die Seitenwahl (" .. name .. ")")
        return "pesm_flow_Schnell_" .. name
    end
    -- Finale frueh kennzeichnen (nur ein Speicher-Schreiben, kein Funktionsaufruf), damit Einlauf/Stadion-Module es schon
    -- beim Spieltag-Bildschirm sehen; der MatchSetup-Haken schreibt es spaeter noch einmal.
    if MENUE and (REGELN.finale == 1 or REGELN.mi) and bereich == "Exhibition" and name == "ExhibiChangeCompeDesignKind" then
        local g = ffi.cast("uint64_t*", 0x143705e10)[0]
        local s = ffi.cast("uint64_t*", g + 0x50)[0] + 0x35960
        ffi.cast("uint8_t*", s + 0x10)[0] = REGELN.mi or 53
        log("pesm_direktmatch: RUNDE gekennzeichnet (S+0x10 = " .. tostring(REGELN.mi or 53) .. ")")
    end
    -- Trikotwahl aus CalcioM (26.11.): der Spieltag-Bildschirm setzt die Kit-Felder beim Aufbau auf Trikot 0 zurueck
    -- (gelesen: S+0x120 = team<<14 | 0 trotz Wahl 2). Nach dem Aufbau nachgesetzt, zeigt die Vorschau die Wahl (mit
    -- Fabian im Spiel bestaetigt): status_frame setzt sie 15 s lang wieder, sobald der Bildschirm sie zuruecksetzt.
    if MENUE and bereich == "Exhibition" and name == "ExhibiMatchMenu" then
        MENUE.trikot_bis = os.time() + 15
        trikots_nachsetzen()
    end
    if false and MENUE and not MENUE.gesetzt and bereich == "Exhibition" and name == "ExhibiChangeCompeDesignKind" then
        MENUE.faellig = true   -- nicht hier (Datei-Lader), sondern im naechsten display_frame
    end
    if flow_dir and datei_da(flow_dir .. bereich .. "\\" .. name .. ".json") then
        log("pesm_direktmatch: ABLAUF " .. bereich .. "/" .. name .. " (Ersatz ohne Menue)")
        return "pesm_flow_" .. bereich .. "_" .. name
    end
end
local function flow_pfad(ctx, filename, key)
    local bereich, name = (key or ""):match("^pesm_flow_(%w+)_([%w_]+)$")
    if bereich then return flow_dir .. bereich .. "\\" .. name .. ".json" end
end

-- Zustands-Marker fuer Testlaeufe und Launcher: flw\pesm_status.txt (Spielstand, Uhr, Periode) und eine Zeile im
-- sider.log bei jedem Wechsel von Periode oder Spielstand. Quelle: match.stats() im display_frame (gedrosselt).
local status_datei, letzt_schreiben, letzt_schluessel = nil, -10, nil
-- VERSUCH 25.11.: Im Direktstart meldet die Spielleitung die Halbzeit nie an den Ablauf-Prozess MatchIdle
-- (ProcessMatchMain, vtable 0x1428cc0b8). Steht die Uhr 8 s auf 45:00 in Periode 2, ruft das Modul dessen
-- Halbzeit-Behandlung 0x1412f5f60(obj) selbst auf (sie loest "match_halftime" im Ablauf aus).
local ht = { seit = nil, erledigt = false }
local tor = { zeit = nil, gemeldet = false }
local msg_buf = nil
local IDLE_SLOT = nil   -- Adresse, an der der Fabrik-Haken das MatchIdle-Objekt ablegt
-- Nachricht an den Ablauf-Prozess MatchIdle schicken (Handler 0x1412f6230(obj, msg), Nachrichten-Id bei msg+8)
local TEST_AKTIV = false   -- mit flw\direktmatch_test.txt: keine Anstoss-Nachrichten (unverfaelschte Beobachtung)
local function an_matchidle(id, was)
    if TEST_AKTIV or MENUE then log("pesm_direktmatch: " .. was .. " (Test/Menue-Weg: nicht gesendet)") return end
    local obj = IDLE_SLOT and tonumber(ffi.cast("uint64_t*", IDLE_SLOT)[0]) or 0
    if obj == 0 then log("pesm_direktmatch: " .. was .. " MatchIdle-Objekt nicht gefunden") return end
    msg_buf = msg_buf or ffi.new("uint32_t[16]")
    msg_buf[2] = id
    log(string.format("pesm_direktmatch: %s -> Nachricht 0x%x an MatchIdle 0x%x", was, id, obj))
    ffi.cast("void(*)(void*, void*)", 0x1412f6230)(ffi.cast("void*", obj), msg_buf)
    log("pesm_direktmatch: " .. was .. " gesendet")
end
-- Schreibt CalcioMs Trikotwahl in S+0x120..0x12c (nur Speicher, kein Funktionsaufruf), wenn dort etwas anderes steht.
function trikots_nachsetzen()
    local g = ffi.cast("uint64_t*", 0x143705e10)[0]
    if g == 0 then return end
    local sp = ffi.cast("uint64_t*", g + 0x50)[0]
    if sp == 0 then return end
    local k = ffi.cast("uint32_t*", sp + 0x35960 + 0x120)
    local h, gg = MENUE.heim * 16384 + MENUE.kh, MENUE.gast * 16384 + MENUE.kg
    if k[0] ~= h or k[1] ~= gg then
        k[0], k[1], k[2], k[3] = h, gg, h + 0x80, gg + 0x80
        log(string.format("pesm_direktmatch: TRIKOTS am Spieltag-Bildschirm gesetzt %d/%d", MENUE.kh, MENUE.kg))
    end
end
local function status_frame(ctx)
    if MENUE and MENUE.faellig and not MENUE.gesetzt then
        MENUE.faellig = false
        menue_teams_setzen()
    end
    if MENUE and MENUE.trikot_bis then
        if os.time() <= MENUE.trikot_bis then trikots_nachsetzen() else MENUE.trikot_bis = nil end
    end
    local t = os.clock()
    if t - letzt_schreiben < 1.0 then return end
    letzt_schreiben = t
    local st = match and match.stats and match.stats()
    if not st then return end
    local p, hs, as = st.period or -1, st.home_score or 0, st.away_score or 0
    local mi, se, nz = st.clock_minutes or 0, st.clock_seconds or 0, st.added_minutes or 0
    local jetzt = os.time()
    if p == 2 and mi == 45 and se == 0 and not ht.erledigt then
        ht.seit = ht.seit or jetzt
        if jetzt - ht.seit >= 5 then ht.erledigt = true an_matchidle(0x105001c, "HALBZEIT") end
    end
    local tore = hs + as
    if tor.anzahl == nil then tor.anzahl = tore end
    if tore > tor.anzahl then tor.anzahl = tore tor.zeit = jetzt tor.gemeldet = false end
    if tor.zeit and not tor.gemeldet and jetzt - tor.zeit >= 20 then
        tor.gemeldet = true
        an_matchidle(0x105000a, "TOR-WIEDERHOLUNG")
    end
    local schluessel = string.format("%d %d:%d", p, hs, as)
    if schluessel ~= letzt_schluessel then
        log(string.format("pesm_direktmatch: ZUSTAND periode=%d stand=%d:%d uhr=%d:%02d", p, hs, as, mi, se))
        letzt_schluessel = schluessel
    end
    local f = status_datei and io.open(status_datei, "wb")
    if f then
        f:write(string.format("periode=%d\r\nminute=%d\r\nsekunde=%d\r\nnachspielzeit=%d\r\nheim_tore=%d\r\ngast_tore=%d\r\nheim_elfer=%d\r\ngast_elfer=%d\r\nlua_uhr=%.1f\r\n",
            p, mi, se, nz, hs, as, st.pk_home_score or 0, st.pk_away_score or 0, t))
        f:close()
    end
end

local SIG = "\x45\x33\xc0\x33\xd2\xb9\x00\x00\x1b\x00\xe8"
local FLOW = "livecpk\\CalcioM_DirectMatch\\common\\script\\flow\\Edit\\Load\\EditDataLoadProcess.json"

local function flow_json(ziel)
    local ev = string.format('{"to":"%s","go":"backward","clear":"all","fadeStart":"","fadeEnd":"do","bgType":"none"}', ziel)
    return '{"events":{"proceed":' .. ev .. ',"0_goToTopMenu":' .. ev ..
        '},"data":{"type":"process","bgId":"","dialog":""},"binFile":{"files":[]}}'
end

local function schreibe(p, inhalt)
    local f = io.open(p, "wb")
    if not f then return false end
    f:write(inhalt)
    f:close()
    return true
end

REGELN = {}
-- Spielregeln in S (Siders MATCH_INFO): +0x10 match_info (53 = Finale), +0x2c Verlaengerung, +0x2f Elfmeterschiessen
local function regel_code()
    local function b1(x) return string.char(x) end
    local function d(x) return memory.pack("u32", x) end
    local c = ""
    if REGELN.finale == 1 then c = c .. "\xc6\x80" .. d(0x35960 + 0x10) .. b1(53) end
    -- Runde allgemein (match_info): 53 Finale, 52 Halbfinale, Ligen: Spieltag - 1
    if REGELN.mi then c = c .. "\xc6\x80" .. d(0x35960 + 0x10) .. b1(REGELN.mi % 256) end
    -- Forschung Siegerehrung: echte Turnier-ID statt 65535 (Freundschaftsspiel) in S+0x0a (u16)
    if false and REGELN.tid then c = c .. "\x66\xc7\x80" .. d(0x35960 + 0x0a) .. memory.pack("u16", REGELN.tid) end
    if REGELN.vl then c = c .. "\xc6\x80" .. d(0x35960 + 0x2c) .. b1(REGELN.vl) end
    if REGELN.em then c = c .. "\xc6\x80" .. d(0x35960 + 0x2f) .. b1(REGELN.em) end
    -- Schwierigkeit der CPU (S+0x18, u32; 0 Anfaenger ... 6 Legende), CalcioM 26.11.
    if REGELN.sk then c = c .. "\xc7\x80" .. d(0x35960 + 0x18) .. d(REGELN.sk) end
    return c
end
function lese_teams(p)
    local f = io.open(p, "r")
    if not f then return nil end
    local t = f:read("*l") or ""
    f:close()
    -- "heim gast [trikot_heim trikot_gast [minuten]]" (Trikot 0 = Heim, 1 = Auswaerts; ohne Angabe 0 und 1;
    -- minuten = echte Spieldauer, 0 = wie im Spiel eingestellt)
    local h, g, kh, kg, mi, weg = t:match("^%s*(%d+)%s+(%d+)%s*(%d*)%s*(%d*)%s*(%d*)%s*(%a*)")
    if not h then return nil end
    -- weitere Regeln als schluessel=wert: vl (Verlaengerung 0/1), em (Elfmeterschiessen 0/1), finale (0/1)
    REGELN = {}
    for k, v in t:gmatch("(%a+)=(%d+)") do REGELN[k] = tonumber(v) end
    return tonumber(h), tonumber(g), tonumber(kh) or 0, tonumber(kg) or 1, tonumber(mi) or 0, weg
end

-- Sider-Lua hat kein os.rename/os.remove: die Ablaufdatei wird umgeschrieben statt umbenannt.
-- "aus" = Inhalt wie das Original (proceed -> TopMenu/TopMenu).
-- Weg "menue": normaler Freundschaftsspiel-Ablauf ab der Seitenwahl (Controller zuordnen), Teamwahl uebersprungen,
-- Teams setzt das Modul vor dem Spieltag-Bildschirm selbst (0x141522c30, wie der Direkt-Setup).
local function schalte(flow, an, weg)
    local ziel = "TopMenu/TopMenu"
    if an then ziel = (weg == "menue" or weg == "menuefrei") and "Exhibition/ExhibiFlowInit" or "Match/MatchDirectSetup" end
    return schreibe(flow, flow_json(ziel))
end

-- Trikots: im Direktstart bleiben die Kit-Felder der Match-Setup-Struktur leer (0xffffc000 Feld, 0xffffc080 Torwart)
-- -> Spieler in Ersatz-Trikots. Struktur S = [[0x143705e10]+0x50]+0x35960; +0x120 Heim, +0x124 Gast (Feld),
-- +0x128/+0x12c (Torwart). Wert = team_id<<14 | Kit-Nummer, Torwart | 0x80 (im Spiel bestaetigt 22.11.).
-- Sider-Ereignisse feuern im Direktstart nicht (set_teams anmelden laesst das Spiel sogar abstuerzen), darum ein
-- kleiner Umweg im Code: direkt nach den zwei Team-Kopien (SIG+0x1e: mov rcx,rbp / call 0x140e525a0) springt das
-- Spiel in eine Seite von uns, die die vier Felder schreibt, den Originalaufruf macht und zuruecksprigt.
local function b(x) return string.char(x) end
local function d32(x) return memory.pack("u32", x % 4294967296) end
local function q64(x) return memory.pack("u32", x % 4294967296) .. memory.pack("u32", math.floor(x / 4294967296)) end

-- Weg "menue": Teams, Trikots, Spieldauer, Regeln und Sider-Ereignisse setzt nativer Code im Spiel-Thread
-- (aus Lua heraus zerlegt Siders set_team_id-Haken den Lua-Stapel). Zwei Stellen:
--  1. Fabrik von Exhibition/ExhibiChangeCompeDesignKind (0x1412ec270), also VOR dem Spieltag-Bildschirm.
--  2. Fabrik von Match/MatchSetup (0x1412f39ac): Spieldauer, Regeln, Sider-Ereignisse (Trikots ausser bei anstoss=selbst).
-- Die Hoehlen-Programme werden mit den Werten aus MENUE neu geschrieben; beim ersten Mal auch die Spruenge dorthin.
-- Schnellstart: vor jedem weiteren Spiel (Seitenwahl) wird direktmatch.txt neu gelesen und die Programme neu geschrieben.
function menue_hoehlen(erstmals)
    local function b(x) return string.char(x) end
    local function d32(x) return memory.pack("u32", x % 4294967296) end
    local function q64(x) return memory.pack("u32", x % 4294967296) .. memory.pack("u32", math.floor(x / 4294967296)) end
    local cave = MENUE.cave
    local h, g = MENUE.heim * 16384 + MENUE.kh, MENUE.gast * 16384 + MENUE.kg
    local function nutzlast(mit_teams, mit_trikots)
        local c = ""
        if mit_teams then
            c = c .. "\xb9" .. d32(MENUE.heim * 16384) .. "\x31\xd2\x45\x31\xc0" .. b(0x48) .. b(0xb8) .. q64(0x141522c30) .. "\xff\xd0"
                .. "\xb9" .. d32(MENUE.gast * 16384) .. "\xba\x01\x00\x00\x00\x45\x31\xc0" .. b(0x48) .. b(0xb8) .. q64(0x141522c30) .. "\xff\xd0"
        end
        c = c .. b(0x48) .. b(0xb8) .. q64(0x143705e10) .. "\x48\x8b\x00\x48\x8b\x40\x50"
        if mit_trikots then
            for _, f in ipairs({ { 0x120, h }, { 0x124, g }, { 0x128, h + 0x80 }, { 0x12c, g + 0x80 } }) do
                c = c .. "\xc7\x80" .. d32(0x35960 + f[1]) .. d32(f[2])
            end
        end
        if MENUE.minuten and MENUE.minuten > 0 then c = c .. "\xc6\x80" .. d32(0x35960 + 0x1c) .. b(MENUE.minuten % 256) end
        c = c .. regel_code()
        c = c .. "\x48\x8d\x88" .. d32(0x35960 + 0x70) .. "\x48\x89\xca"
            .. b(0x49) .. b(0xbb) .. q64(0x1414c07b0) .. "\x41\xff\xd3"
        return c
    end
    local fe = 0x1412ec270
    if erstmals then MENUE.frueh = memory.read(fe, 6) == "\x40\x53\x48\x83\xec\x30" and REGELN.nofrueh ~= 1 end
    if MENUE.frueh then
        local c4 = cave + 0x600
        local code = "\x51\x52\x48\x83\xec\x28" .. nutzlast(true, true) .. "\x48\x83\xc4\x28\x5a\x59"
            .. "\x40\x53\x48\x83\xec\x30" .. b(0x48) .. b(0xb8) .. q64(fe + 6) .. "\xff\xe0"
        ffi.copy(ffi.cast("void*", c4), code, #code)
        if erstmals then memory.write(fe, b(0xe9) .. memory.pack("i32", c4 - (fe + 5)) .. "\x90") end
        log(string.format("pesm_direktmatch: MENUE Teams %d/%d vor dem Spieltag-Bildschirm -> 0x%x", MENUE.heim, MENUE.gast, c4))
    elseif erstmals then
        log("pesm_direktmatch: ChangeCompeDesignKind-Fabrik passt nicht - Teams erst beim Anstoss")
    end
    local sj = 0x1412f39ac
    if erstmals then
        MENUE.setup_ok = memory.read(sj, 1) == "\xe9" and sj + 5 + memory.unpack("i32", memory.read(sj + 1, 4)) == 0x1412f38f0
    end
    if MENUE.setup_ok then
        local c3 = cave + 0x400
        local code = "\x51\x52\x48\x83\xec\x28" .. nutzlast(not MENUE.frueh, true)
            .. "\x48\x83\xc4\x28\x5a\x59" .. b(0x48) .. b(0xb8) .. q64(0x1412f38f0) .. "\xff\xe0"
        ffi.copy(ffi.cast("void*", c3), code, #code)
        if erstmals then memory.write(sj + 1, memory.pack("i32", c3 - (sj + 5))) end
        MENUE.gesetzt = true
    elseif erstmals then
        log("pesm_direktmatch: MatchSetup-Fabrik passt nicht - Teams im Menue-Weg nicht gesetzt")
    end
end

ctx_root = ""
local function trikots_haken(addr, heim, gast, kh, kg, minuten)
    if ffi == nil then
        log("pesm_direktmatch: ffi fehlt (luajit.ext.enabled = 1) - Trikots nicht gesetzt")
        return
    end
    -- memory.search_process liefert einen char*-Zeiger; als Zahl rechnen
    local hook = tonumber(ffi.cast("uint64_t", addr)) + 0x1e
    if memory.read(hook, 4) ~= "\x48\x8b\xcd\xe8" then
        log("pesm_direktmatch: Hakenstelle passt nicht - Trikots nicht gesetzt")
        return
    end
    local rel = memory.unpack("i32", memory.read(hook + 4, 4))
    local ziel = hook + 8 + rel                       -- 0x140e525a0
    ffi.cdef([[ void* pesm_direktmatch_VirtualAlloc(void*, size_t, uint32_t, uint32_t) __asm__("VirtualAlloc"); ]])
    local h, g = heim * 16384 + kh, gast * 16384 + kg
    -- Felder (Abstand in S, Wert): Trikots, dazu TEST 22.11.: Werte, die das normale Menue setzt und der Direktstart
    -- leer laesst (Vergleich Speicher normales Freundschaftsspiel gegen Direktstart). Bedeutung noch offen.
    local felder = {
        { 0x120, h }, { 0x124, g }, { 0x128, h + 0x80 }, { 0x12c, g + 0x80 },
        -- Liga-Wert wie im normalen Freundschaftsspiel (Test 22.11., ob er den Torjubel freischaltet; ungeprueft)
        { 0x34, 0x00d60009 },
    }
    local body = b(0x48) .. b(0xb8) .. q64(0x143705e10)                 -- mov rax, 0x143705e10
        .. "\x48\x8b\x00"                                         -- mov rax, [rax]
        .. "\x48\x8b\x40\x50"                                     -- mov rax, [rax+0x50]
    -- Forschung: zusaetzliche S-Felder aus flw\direktmatch_test.txt ("abstand wert" je Zeile, hex), nur wenn vorhanden
    local tf = io.open(ctx_root .. "flw\\direktmatch_test.txt", "r")
    if tf then
        TEST_AKTIV = true
        for line in tf:lines() do
            local o, v = line:match("^%s*0x(%x+)%s+0x(%x+)")
            if o then
                felder[#felder + 1] = { tonumber(o, 16), tonumber(v, 16) }
                log(string.format("pesm_direktmatch: TEST S+0x%x = 0x%x", tonumber(o, 16), tonumber(v, 16)))
            end
        end
        tf:close()
    end
    for _, f in ipairs(felder) do
        body = body .. "\xc7\x80" .. d32(0x35960 + f[1]) .. d32(f[2])   -- mov dword [rax+S+off], wert
    end
    -- Spieldauer: S+0x1c (u8, Minuten). Setter im Spiel 0x1414c1d00 (mov [rcx+1c],dl), den der Direktstart nicht ruft;
    -- Siders match.minutes patcht nur diesen Setter und wirkt darum im Direktstart nicht.
    body = body .. regel_code()
    if minuten and minuten > 0 then
        body = body .. "\xc6\x80" .. d32(0x35960 + 0x1c) .. b(minuten % 256)        -- mov byte [rax+S+1c], minuten
    end
    -- Sider-Ereignisse nachholen: Der Direktstart ruft die Kopierfunktion der Spiel-Einstellungen (0x1414c07b0) nie auf;
    -- an deren Ende haengt Siders set_settings-Haken (set_match_settings, set_conditions, set_stadium ...). Wir rufen
    -- sie mit Ziel = Quelle = S+0x70 auf (Selbstkopie aendert nichts), damit Stadion-, Wetter-, Scoreboard-Module wie im
    -- normalen Freundschaftsspiel arbeiten. rsp ist hier (Aufrufstelle im Original) 16-Byte-ausgerichtet.
    body = body
        .. "\x48\x8d\x88" .. d32(0x35960 + 0x70)                    -- lea rcx, [rax+S+0x70] (Teilstruktur; Sider: mi = rcx-0x70)
        .. "\x48\x89\xca"                                           -- mov rdx, rcx
        .. "\x48\x83\xec\x20"                                       -- sub rsp, 0x20
        .. b(0x49) .. b(0xbb) .. q64(0x1414c07b0) .. "\x41\xff\xd3"   -- mov r11, 0x1414c07b0 ; call r11
        .. "\x48\x83\xc4\x20"                                       -- add rsp, 0x20
    body = body
        .. "\x48\x8b\xcd"                                         -- mov rcx, rbp (Original)
        .. b(0x48) .. b(0xb8) .. q64(ziel) .. "\xff\xd0"           -- call 0x140e525a0 (Original)
        .. b(0x48) .. b(0xb8) .. q64(hook + 8) .. "\xff\xe0"       -- zurueck hinter die Hakenstelle
    local cave
    for _, pref in ipairs({ 0x13f800000, 0x13f000000, 0x13e800000, 0x15e900000, 0x15f900000 }) do
        local p = ffi.C.pesm_direktmatch_VirtualAlloc(ffi.cast("void*", pref), 0x1000, 0x3000, 0x40)  -- RWX
        if p ~= nil then
            local c = tonumber(ffi.cast("uint64_t", p))
            if math.abs(c - (hook + 5)) < 0x7fff0000 then cave = c break end
        end
    end
    if not cave then
        log("pesm_direktmatch: keine Seite in Reichweite - Trikots nicht gesetzt")
        return
    end
    ffi.copy(ffi.cast("void*", cave), body, #body)
    local j = b(0xe9) .. memory.pack("i32", cave - (hook + 5)) .. "\x90\x90\x90"
    memory.write(hook, j)
    log(string.format("pesm_direktmatch: Trikot-Haken bei %s -> 0x%x", memory.hex(hook), cave))
    -- MatchIdle-Fabrik 0x1412f5f50 endet mit "jmp 0x1412f5ed0" (bei +0xa): umlenken auf cave+0x200, das die Fabrik
    -- aufruft, das neue Objekt in cave+0x300 merkt und zurueckkehrt.
    local fj = 0x1412f5f5a
    if memory.read(fj, 1) == "\xe9" and fj + 5 + memory.unpack("i32", memory.read(fj + 1, 4)) == 0x1412f5ed0 then
        local c2 = cave + 0x200
        local code = "\x48\x83\xec\x28" .. b(0x48) .. b(0xb8) .. q64(0x1412f5ed0) .. "\xff\xd0"
            .. b(0x48) .. b(0xb9) .. q64(cave + 0x300) .. "\x48\x89\x01" .. "\x48\x83\xc4\x28\xc3"
        ffi.copy(ffi.cast("void*", c2), code, #code)
        memory.write(fj + 1, memory.pack("i32", c2 - (fj + 5)))
        IDLE_SLOT = ffi.cast("void*", cave + 0x300)
        log(string.format("pesm_direktmatch: MatchIdle-Fabrik-Haken -> 0x%x", c2))
    else
        log("pesm_direktmatch: MatchIdle-Fabrik passt nicht - kein Anstossen moeglich")
    end
    -- Weg "menue": Teams, Trikots, Spieldauer, Regeln und Sider-Ereignisse setzt nativer Code im Spiel-Thread
    -- (aus Lua heraus zerlegt Siders set_team_id-Haken den Lua-Stapel). Zwei Stellen:
    --  1. Fabrik von Exhibition/ExhibiChangeCompeDesignKind (0x1412ec270), also VOR dem Spieltag-Bildschirm: Teams und
    --     Trikots sind dort schon zu sehen (wichtig, wenn die Spieler selbst "Anstoss" druecken und Trikots aendern).
    --  2. Fabrik von Match/MatchSetup (0x1412f39ac): Spieldauer, Regeln, Sider-Ereignisse und die Trikots aus CalcioM
    --     (seit 26.11. immer: die Wahl liegt in CalcioM, der Spieltag-Bildschirm setzte sie zurueck).
    if MENUE then
        MENUE.cave = cave
        menue_hoehlen(true)
    end
end

function m.init(ctx)
    local root = ctx.sider_dir
    ctx_root = root
    local txt = root .. "flw\\direktmatch.txt"
    local flow = root .. FLOW
    local heim, gast, kh, kg, minuten, weg = lese_teams(txt)
    if not heim then
        schalte(flow, false)
        log("pesm_direktmatch: aus (keine flw\\direktmatch.txt) - normales Hauptmenue")
        return
    end
    local addr = memory.search_process(SIG)
    if not addr then
        schalte(flow, false)
        log("pesm_direktmatch: Stelle nicht gefunden - Direktstart aus, normales Hauptmenue")
        return
    end
    -- mov ecx,imm32 (Heim) bei addr+5, mov ecx,imm32 (Gast) bei addr+20
    memory.write(addr + 6, memory.pack("u32", heim * 16384))
    memory.write(addr + 21, memory.pack("u32", gast * 16384))
    local an = schalte(flow, true, weg)
    MENUE = (weg == "menue") and { heim = heim, gast = gast, kh = kh, kg = kg, minuten = minuten, gesetzt = false,
        seitenwahl = 0 } or nil
    trikots_haken(addr, heim, gast, kh, kg, minuten)
    status_datei = root .. "flw\\pesm_status.txt"
    flow_dir = root .. "content\\calciom-direktmatch\\flow\\"
    ctx.register("livecpk_make_key", flow_key)
    ctx.register("livecpk_get_filepath", flow_pfad)
    ctx.register("display_frame", status_frame)
    -- einmalig: Schalter verbrauchen, damit der naechste Start wieder ins Hauptmenue geht
    schreibe(txt, string.format("# benutzt: %d %d\r\n", heim, gast))
    log(string.format("pesm_direktmatch: Trikots Heim %d, Gast %d, Spieldauer %s", kh, kg,
        minuten > 0 and (minuten .. " min") or "wie eingestellt"))
    log(string.format("pesm_direktmatch: Heim %d, Gast %d gesetzt (%s), Ablaufdatei %s", heim, gast,
        memory.hex(addr), an and "aktiv" or "FEHLT"))
end

return m
