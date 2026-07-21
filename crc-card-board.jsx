import { useState, useEffect, useRef } from "react";

// CRC Card Board — Tier 0 prototype (chat artifact, not operated by GWF)
// Draggable index cards, visible collaborator connections, multi-select, zoom.

const T = {
  desk: "#39424E", deskEdge: "#2E3640",
  card: "#FBF9F2", cardShadow: "rgba(15,20,26,0.35)",
  rule: "#C3D3E4", redRule: "#CC5148",
  ink: "#26241F", inkSoft: "#6B675C",
  chipBg: "#EDF2F7", chipBorder: "#B9C9DB",
  btn: "#2B4A73",
  line: "rgba(233,238,244,0.4)", lineHot: "#CC5148",
};

const CARD_W = 340;
const COLORS = ["#CC5148", "#3E6FA3", "#4E8A5E", "#C98A2B", "#7C5FA8", "#6B7885"];
const uid = () => Math.random().toString(36).slice(2, 10);

const STARTER = [
  { id: uid(), name: "Order", responsibilities: ["Know its line items", "Calculate total price", "Track its status"], collaborators: ["LineItem", "Customer"], x: 60, y: 40 },
  { id: uid(), name: "LineItem", responsibilities: ["Know product and quantity", "Compute line price"], collaborators: [], x: 480, y: 320 },
  { id: uid(), name: "Customer", responsibilities: ["Know contact details", "Know order history"], collaborators: ["Order"], x: 500, y: 30 },
];

function autoLayout(cards) {
  const cols = Math.max(1, Math.ceil(Math.sqrt(cards.length)));
  return cards.map((c, i) => ({ ...c, x: 50 + (i % cols) * (CARD_W + 90), y: 40 + Math.floor(i / cols) * 330 }));
}

export default function CrcBoard() {
  const [cards, setCards] = useState(STARTER);
  const [loaded, setLoaded] = useState(false);
  const [selected, setSelected] = useState([]);       // multi-select: array of card ids
  const [marquee, setMarquee] = useState(null);       // {x1,y1,x2,y2} in board coords
  const [toast, setToast] = useState(null);
  const [sizes, setSizes] = useState({});
  const [zoom, setZoom] = useState(1);

  const cardRefs = useRef({});
  const saveTimer = useRef(null);
  const drag = useRef(null);
  const marq = useRef(null);
  const fileInput = useRef(null);
  const boardRef = useRef(null);
  const zoomRef = useRef(1); zoomRef.current = zoom;
  const cardsRef = useRef(cards); cardsRef.current = cards;
  const sizesRef = useRef(sizes); sizesRef.current = sizes;

  // ---- persistence (best effort) ----
  useEffect(() => {
    (async () => {
      try {
        const res = await window.storage.get("crc:board");
        if (res && res.value) {
          const parsed = JSON.parse(res.value);
          if (Array.isArray(parsed.cards) && parsed.cards.length) {
            const needsLayout = parsed.cards.some((c) => typeof c.x !== "number");
            setCards(needsLayout ? autoLayout(parsed.cards) : parsed.cards);
          }
        }
      } catch (e) { /* nothing saved yet */ }
      setLoaded(true);
    })();
  }, []);

  useEffect(() => {
    if (!loaded) return;
    clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(async () => {
      try { await window.storage.set("crc:board", JSON.stringify({ cards })); }
      catch (e) { /* session-only */ }
    }, 600);
    return () => clearTimeout(saveTimer.current);
  }, [cards, loaded]);

  // ---- measure card heights so lines anchor to card centers ----
  useEffect(() => {
    const next = {};
    let changed = false;
    for (const c of cards) {
      const el = cardRefs.current[c.id];
      if (el) { next[c.id] = el.offsetHeight; if (sizes[c.id] !== el.offsetHeight) changed = true; }
    }
    if (changed || Object.keys(next).length !== Object.keys(sizes).length) setSizes(next);
  });

  // ---- coordinate helper ----
  const boardCoords = (e) => {
    const el = boardRef.current;
    const r = el.getBoundingClientRect();
    const z = zoomRef.current;
    return { x: (e.clientX - r.left + el.scrollLeft) / z, y: (e.clientY - r.top + el.scrollTop) / z };
  };

  // ---- drag (moves every selected card together) ----
  const startDrag = (e, id) => {
    if (e.target.closest("input, button")) return;
    const additive = e.shiftKey || e.ctrlKey || e.metaKey;
    let sel;
    if (additive) sel = selected.includes(id) ? selected.filter((x) => x !== id) : [...selected, id];
    else sel = selected.includes(id) ? selected : [id];
    setSelected(sel);
    if (!sel.includes(id)) return; // was toggled off — nothing to drag
    const origins = {};
    for (const c of cards) if (sel.includes(c.id)) origins[c.id] = { x: c.x, y: c.y };
    drag.current = { z: zoomRef.current, sx: e.clientX, sy: e.clientY, origins };
    e.preventDefault();
  };

  // ---- marquee selection on empty board space ----
  const startMarquee = (e) => {
    if (!(e.target.getAttribute && e.target.getAttribute("data-boardbg"))) return;
    const p = boardCoords(e);
    marq.current = { x1: p.x, y1: p.y, additive: e.shiftKey, base: e.shiftKey ? [...selected] : [] };
    setMarquee({ x1: p.x, y1: p.y, x2: p.x, y2: p.y });
    if (!e.shiftKey) setSelected([]);
  };

  useEffect(() => {
    const move = (e) => {
      if (drag.current) {
        const { z, sx, sy, origins } = drag.current;
        const dx = (e.clientX - sx) / z, dy = (e.clientY - sy) / z;
        setCards((p) => p.map((c) => origins[c.id]
          ? { ...c, x: Math.max(0, origins[c.id].x + dx), y: Math.max(0, origins[c.id].y + dy) }
          : c));
      } else if (marq.current) {
        const p = boardCoords(e);
        const m = { x1: marq.current.x1, y1: marq.current.y1, x2: p.x, y2: p.y };
        setMarquee(m);
        const minx = Math.min(m.x1, m.x2), maxx = Math.max(m.x1, m.x2);
        const miny = Math.min(m.y1, m.y2), maxy = Math.max(m.y1, m.y2);
        const hits = cardsRef.current
          .filter((c) => c.x < maxx && c.x + CARD_W > minx && c.y < maxy && c.y + (sizesRef.current[c.id] || 260) > miny)
          .map((c) => c.id);
        setSelected(marq.current.additive ? [...new Set([...marq.current.base, ...hits])] : hits);
      }
    };
    const up = () => { drag.current = null; marq.current = null; setMarquee(null); };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    return () => { window.removeEventListener("pointermove", move); window.removeEventListener("pointerup", up); };
  }, []);

  // ---- operations ----
  const addCard = () => {
    const el = boardRef.current;
    const c = {
      id: uid(), name: "", responsibilities: [], collaborators: [],
      x: (el ? el.scrollLeft / zoomRef.current : 0) + 80 + (cards.length % 3) * 40,
      y: (el ? el.scrollTop / zoomRef.current : 0) + 60 + (cards.length % 3) * 40,
    };
    setCards((p) => [...p, c]);
    setSelected([c.id]);
    setTimeout(() => cardRefs.current[c.id]?.querySelector("input")?.focus(), 50);
  };
  const update = (id, patch) => setCards((p) => p.map((c) => (c.id === id ? { ...c, ...patch } : c)));
  const removeCard = (id) => { setCards((p) => p.filter((c) => c.id !== id)); setSelected((s) => s.filter((x) => x !== id)); };

  const jumpTo = (name) => {
    const t = cards.find((c) => c.name.trim().toLowerCase() === name.trim().toLowerCase());
    if (!t) return;
    setSelected([t.id]);
    const z = zoomRef.current;
    boardRef.current?.scrollTo({ left: Math.max(0, t.x * z - 100), top: Math.max(0, t.y * z - 100), behavior: "smooth" });
  };

  // ---- zoom ----
  const clampZoom = (z) => Math.min(1.5, Math.max(0.3, z));
  const zoomBy = (f) => setZoom((z) => clampZoom(Math.round(z * f * 100) / 100));
  const fitToView = () => {
    const el = boardRef.current;
    if (!el || !cards.length) return;
    const maxX = Math.max(...cards.map((c) => c.x + CARD_W)) + 60;
    const maxY = Math.max(...cards.map((c) => c.y + (sizes[c.id] || 260))) + 60;
    setZoom(clampZoom(Math.min(el.clientWidth / maxX, el.clientHeight / maxY, 1)));
    el.scrollTo({ left: 0, top: 0 });
  };
  useEffect(() => {
    const el = boardRef.current;
    if (!el) return;
    const onWheel = (e) => {
      if (!e.ctrlKey && !e.metaKey) return;
      e.preventDefault();
      setZoom((z) => clampZoom(z * (e.deltaY < 0 ? 1.08 : 0.92)));
    };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  }, [cards.length === 0]);

  const showToast = (m) => { setToast(m); setTimeout(() => setToast(null), 2200); };

  const exportMarkdown = async () => {
    const md = cards.map((c) => {
      const r = c.responsibilities.map((x) => `- ${x}`).join("\n") || "- (none yet)";
      return `## ${c.name || "Unnamed class"}\n\n**Responsibilities**\n${r}\n\n**Collaborators:** ${c.collaborators.join(", ") || "(none)"}\n`;
    }).join("\n");
    try { await navigator.clipboard.writeText(`# CRC cards\n\nClassification: internal\n\n${md}`); showToast("Copied as Markdown"); }
    catch (e) { showToast("Copy failed — try again"); }
  };

  const [exportText, setExportText] = useState(null);

  const exportJson = () => {
    setExportText(JSON.stringify({ format: "crc-board", version: 1, exported: new Date().toISOString(), classification: "internal", cards }, null, 2));
  };

  const downloadExport = () => {
    try {
      const blob = new Blob([exportText], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "crc-board.json";
      document.body.appendChild(a);
      a.click();
      a.remove();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      showToast("Download started — check your Downloads folder");
    } catch (e) {
      showToast("Download blocked here — use Copy instead");
    }
  };

  const copyExport = async () => {
    try { await navigator.clipboard.writeText(exportText); showToast("JSON copied — paste into a .json file"); }
    catch (e) { showToast("Copy failed — select the text manually"); }
  };

  const importJson = (e) => {
    const file = e.target.files && e.target.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      try {
        const parsed = JSON.parse(reader.result);
        const list = Array.isArray(parsed) ? parsed : parsed.cards;
        if (!Array.isArray(list)) throw new Error("no cards array");
        const clean = list.map((c) => ({
          id: typeof c.id === "string" ? c.id : uid(),
          name: typeof c.name === "string" ? c.name : "",
          responsibilities: Array.isArray(c.responsibilities) ? c.responsibilities.filter((x) => typeof x === "string") : [],
          collaborators: Array.isArray(c.collaborators) ? c.collaborators.filter((x) => typeof x === "string") : [],
          color: typeof c.color === "string" ? c.color : undefined,
          x: typeof c.x === "number" ? c.x : undefined,
          y: typeof c.y === "number" ? c.y : undefined,
        }));
        const final = clean.some((c) => typeof c.x !== "number") ? autoLayout(clean) : clean;
        if (cards.length && !window.confirm(`Replace the current ${cards.length} card(s) with ${final.length} imported card(s)?`)) { e.target.value = ""; return; }
        setCards(final);
        setSelected([]);
        showToast(`Imported ${final.length} card${final.length === 1 ? "" : "s"}`);
      } catch (err) { showToast("Import failed — not a valid CRC board file"); }
      e.target.value = "";
    };
    reader.readAsText(file);
  };

  const clearBoard = () => { if (cards.length && !window.confirm("Remove all cards?")) return; setCards([]); setSelected([]); };

  const names = cards.map((c) => c.name.trim()).filter(Boolean);
  const byName = {};
  for (const c of cards) if (c.name.trim()) byName[c.name.trim().toLowerCase()] = c;

  // ---- connections ----
  const center = (c) => ({ cx: c.x + CARD_W / 2, cy: c.y + (sizes[c.id] || 260) / 2 });
  const edgePoint = (from, to) => {
    const w = CARD_W / 2, h = (sizes[from.id] || 260) / 2;
    const { cx, cy } = center(from);
    const t = center(to);
    const dx = t.cx - cx, dy = t.cy - cy;
    if (dx === 0 && dy === 0) return { x: cx, y: cy };
    const s = Math.min(Math.abs(w / (dx || 1e-6)), Math.abs(h / (dy || 1e-6)));
    return { x: cx + dx * s, y: cy + dy * s };
  };
  const links = [];
  for (const c of cards) for (const n of c.collaborators) {
    const t = byName[n.trim().toLowerCase()];
    if (t && t.id !== c.id) links.push({ from: c, to: t, hot: selected.includes(c.id) || selected.includes(t.id) });
  }
  const boardW = Math.max(1200, ...cards.map((c) => c.x + CARD_W + 120));
  const boardH = Math.max(700, ...cards.map((c) => c.y + (sizes[c.id] || 260) + 120));

  return (
    <div style={{ height: "100vh", display: "flex", flexDirection: "column", background: `linear-gradient(180deg, ${T.desk}, ${T.deskEdge})`, fontFamily: "system-ui, -apple-system, 'Segoe UI', sans-serif", color: T.ink }}>
      <div style={{ display: "flex", flexWrap: "wrap", alignItems: "baseline", gap: 12, padding: "18px 24px 12px", color: "#E8ECF1", flexShrink: 0 }}>
        <h1 style={{ margin: 0, fontFamily: "Georgia, 'Times New Roman', serif", fontWeight: 400, fontSize: 24 }}>CRC cards</h1>
        <span style={{ fontFamily: "ui-monospace, Menlo, monospace", fontSize: 12, opacity: 0.75 }}>
          {cards.length} {cards.length === 1 ? "class" : "classes"} · {links.length} {links.length === 1 ? "connection" : "connections"}{selected.length > 1 ? ` · ${selected.length} selected` : ""} · drag empty space to select several
        </span>
        <div style={{ marginLeft: "auto", display: "flex", gap: 8 }}>
          <input ref={fileInput} type="file" accept=".json,application/json" onChange={importJson} style={{ display: "none" }} aria-label="Import board file" />
          <HeaderBtn onClick={() => fileInput.current && fileInput.current.click()} label="Import" />
          <HeaderBtn onClick={exportJson} label="Export" />
          <HeaderBtn onClick={exportMarkdown} label="Copy as Markdown" />
          <HeaderBtn onClick={() => setCards((p) => autoLayout(p))} label="Auto-arrange" />
          <HeaderBtn onClick={clearBoard} label="Clear board" subtle />
          <button onClick={addCard} style={{ background: T.card, color: T.ink, border: "none", borderRadius: 6, padding: "8px 16px", fontSize: 13, fontWeight: 600, cursor: "pointer", boxShadow: "0 2px 6px rgba(0,0,0,0.3)" }}>+ New card</button>
        </div>
      </div>

      <div ref={boardRef} style={{ flex: 1, overflow: "auto", position: "relative" }} onPointerDown={startMarquee}>
        {cards.length === 0 ? (
          <div style={{ textAlign: "center", color: "#B9C2CC", marginTop: 90 }}>
            <p style={{ fontFamily: "Georgia, serif", fontSize: 20, margin: 0 }}>The desk is empty.</p>
            <p style={{ fontSize: 13, marginTop: 8 }}>Add a card per candidate class. Collaborator lines appear once names match.</p>
            <button onClick={addCard} style={{ marginTop: 16, background: T.card, color: T.ink, border: "none", borderRadius: 6, padding: "10px 20px", fontSize: 14, fontWeight: 600, cursor: "pointer" }}>+ First card</button>
          </div>
        ) : (
          <div style={{ width: boardW * zoom, height: boardH * zoom }}>
            <div data-boardbg="1" style={{ position: "relative", width: boardW, height: boardH, transform: `scale(${zoom})`, transformOrigin: "0 0" }}>
              <svg width={boardW} height={boardH} style={{ position: "absolute", inset: 0, pointerEvents: "none" }}>
                <defs>
                  <marker id="arr" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
                    <path d="M 0 0 L 10 5 L 0 10 z" fill={T.line} />
                  </marker>
                  <marker id="arrHot" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
                    <path d="M 0 0 L 10 5 L 0 10 z" fill={T.lineHot} />
                  </marker>
                </defs>
                {links.map((l, i) => {
                  const a = edgePoint(l.from, l.to);
                  const b = edgePoint(l.to, l.from);
                  const mx = (a.x + b.x) / 2, my = (a.y + b.y) / 2 - 30;
                  return (
                    <path key={i} d={`M ${a.x} ${a.y} Q ${mx} ${my} ${b.x} ${b.y}`} fill="none"
                      stroke={l.hot ? T.lineHot : T.line} strokeWidth={l.hot ? 2.5 : 1.5}
                      strokeDasharray={l.hot ? "none" : "6 5"} markerEnd={l.hot ? "url(#arrHot)" : "url(#arr)"} />
                  );
                })}
              </svg>

              {cards.map((c) => (
                <IndexCard key={c.id} card={c}
                  selected={selected.includes(c.id)}
                  names={names}
                  refFn={(el) => (cardRefs.current[c.id] = el)}
                  onPointerDown={(e) => startDrag(e, c.id)}
                  onChange={(patch) => update(c.id, patch)}
                  onDelete={() => removeCard(c.id)}
                  onJump={jumpTo}
                />
              ))}

              {marquee && (
                <div style={{
                  position: "absolute",
                  left: Math.min(marquee.x1, marquee.x2), top: Math.min(marquee.y1, marquee.y2),
                  width: Math.abs(marquee.x2 - marquee.x1), height: Math.abs(marquee.y2 - marquee.y1),
                  border: `1.5px dashed ${T.redRule}`, background: "rgba(204,81,72,0.08)", pointerEvents: "none",
                }} />
              )}
            </div>
          </div>
        )}
      </div>

      <div style={{ position: "fixed", bottom: 20, right: 20, display: "flex", alignItems: "center", gap: 2, background: "#1E242C", borderRadius: 8, boxShadow: "0 4px 14px rgba(0,0,0,0.4)", padding: 3 }}>
        <ZoomBtn onClick={() => zoomBy(1 / 1.15)} label="−" title="Zoom out" />
        <button onClick={() => setZoom(1)} title="Reset to 100%" style={{ border: "none", background: "transparent", color: "#EDF1F5", fontSize: 12, fontFamily: "ui-monospace, Menlo, monospace", width: 48, cursor: "pointer" }}>
          {Math.round(zoom * 100)}%
        </button>
        <ZoomBtn onClick={() => zoomBy(1.15)} label="+" title="Zoom in" />
        <ZoomBtn onClick={fitToView} label="Fit" title="Fit all cards in view" wide />
      </div>

      {exportText !== null && (
        <div onClick={() => setExportText(null)}
          style={{ position: "fixed", inset: 0, background: "rgba(15,20,26,0.6)", display: "flex", alignItems: "center", justifyContent: "center", zIndex: 50 }}>
          <div onClick={(e) => e.stopPropagation()}
            style={{ background: T.card, borderRadius: 8, padding: 20, width: "min(560px, 92vw)", boxShadow: "0 12px 40px rgba(0,0,0,0.5)" }}>
            <h2 style={{ margin: "0 0 4px", fontFamily: "Georgia, serif", fontWeight: 400, fontSize: 19 }}>Export board</h2>
            <p style={{ margin: "0 0 10px", fontSize: 12.5, color: T.inkSoft }}>
              Download as a file, or copy the JSON and save it as crc-board.json yourself.
            </p>
            <textarea readOnly value={exportText} onFocus={(e) => e.target.select()}
              style={{ width: "100%", height: 180, resize: "vertical", fontFamily: "ui-monospace, Menlo, monospace", fontSize: 11.5, border: `1px solid ${T.rule}`, borderRadius: 6, padding: 8, background: "#fff", color: T.ink, boxSizing: "border-box" }} />
            <div style={{ display: "flex", gap: 8, marginTop: 12, justifyContent: "flex-end" }}>
              <button onClick={() => setExportText(null)} style={{ background: "transparent", color: T.inkSoft, border: `1px solid ${T.rule}`, borderRadius: 6, padding: "8px 14px", fontSize: 13, cursor: "pointer" }}>Close</button>
              <button onClick={copyExport} style={{ background: "transparent", color: T.btn, border: `1px solid ${T.btn}`, borderRadius: 6, padding: "8px 14px", fontSize: 13, cursor: "pointer" }}>Copy JSON</button>
              <button onClick={downloadExport} style={{ background: T.btn, color: T.btnText, border: "none", borderRadius: 6, padding: "8px 16px", fontSize: 13, fontWeight: 600, cursor: "pointer" }}>Download .json</button>
            </div>
          </div>
        </div>
      )}

      {toast && <div style={{ position: "fixed", bottom: 22, left: "50%", transform: "translateX(-50%)", background: "#1E242C", color: "#EDF1F5", padding: "10px 18px", borderRadius: 8, fontSize: 13, boxShadow: "0 4px 14px rgba(0,0,0,0.4)" }}>{toast}</div>}
    </div>
  );
}

function HeaderBtn({ onClick, label, subtle }) {
  return (
    <button onClick={onClick} style={{ background: "transparent", color: subtle ? "#9AA6B2" : "#DDE5EC", border: `1px solid ${subtle ? "#4A555F" : "#6B7885"}`, borderRadius: 6, padding: "8px 14px", fontSize: 13, cursor: "pointer" }}>
      {label}
    </button>
  );
}

function ZoomBtn({ onClick, label, title, wide }) {
  return (
    <button onClick={onClick} title={title} aria-label={title}
      style={{ border: "none", background: "transparent", color: "#EDF1F5", fontSize: wide ? 12 : 16, width: wide ? 38 : 30, height: 30, borderRadius: 6, cursor: "pointer" }}
      onMouseEnter={(e) => (e.currentTarget.style.background = "#333B45")}
      onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}>
      {label}
    </button>
  );
}

function IndexCard({ card, selected, names, refFn, onPointerDown, onChange, onDelete, onJump }) {
  const [respDraft, setRespDraft] = useState("");
  const [collDraft, setCollDraft] = useState("");
  const [showPalette, setShowPalette] = useState(false);
  const crowded = card.responsibilities.length > 4;
  const col = card.color || COLORS[0];

  const addResp = () => { const v = respDraft.trim(); if (!v) return; onChange({ responsibilities: [...card.responsibilities, v] }); setRespDraft(""); };
  const addColl = (raw) => { const v = (raw ?? collDraft).trim(); if (!v || card.collaborators.includes(v)) { setCollDraft(""); return; } onChange({ collaborators: [...card.collaborators, v] }); setCollDraft(""); };

  const ruled = { backgroundImage: `repeating-linear-gradient(180deg, transparent 0, transparent 25px, ${T.rule} 25px, ${T.rule} 26px)`, lineHeight: "26px" };
  const suggestions = names.filter((n) => n.toLowerCase() !== card.name.trim().toLowerCase() && !card.collaborators.includes(n));

  return (
    <div ref={refFn} onPointerDown={onPointerDown}
      style={{
        position: "absolute", left: card.x, top: card.y, width: CARD_W,
        background: T.card, borderRadius: 3,
        boxShadow: selected ? `0 0 0 3px rgba(255,255,255,0.9), 0 8px 20px ${T.cardShadow}` : `0 6px 16px ${T.cardShadow}`,
        display: "flex", flexDirection: "column", minHeight: 230,
        cursor: "grab", userSelect: "none", touchAction: "none",
      }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "10px 14px 6px", borderBottom: `2px solid ${col}` }}>
        <button onClick={() => setShowPalette((s) => !s)} title="Card color" aria-label="Change card color"
          style={{ width: 14, height: 14, borderRadius: "50%", background: col, border: "1.5px solid rgba(0,0,0,0.2)", cursor: "pointer", padding: 0, flexShrink: 0 }} />
        <input value={card.name} onChange={(e) => onChange({ name: e.target.value })} placeholder="Class name" aria-label="Class name"
          style={{ flex: 1, border: "none", outline: "none", background: "transparent", fontFamily: "Georgia, serif", fontSize: 19, color: T.ink, cursor: "text" }} />
        <button onClick={onDelete} title="Remove card" aria-label={`Remove card ${card.name || "unnamed"}`}
          style={{ border: "none", background: "transparent", color: T.inkSoft, fontSize: 15, cursor: "pointer", padding: "2px 6px", lineHeight: 1 }}>✕</button>
      </div>
      {showPalette && (
        <div style={{ display: "flex", gap: 8, padding: "8px 14px", borderBottom: `1px solid ${T.rule}`, background: "rgba(0,0,0,0.03)" }}>
          {COLORS.map((c) => (
            <button key={c} onClick={() => { onChange({ color: c }); setShowPalette(false); }}
              title="Set color" aria-label={`Set card color ${c}`}
              style={{
                width: 18, height: 18, borderRadius: "50%", background: c, cursor: "pointer", padding: 0,
                border: c === col ? "2px solid #26241F" : "1.5px solid rgba(0,0,0,0.15)",
              }} />
          ))}
        </div>
      )}

      <div style={{ display: "flex", flex: 1 }}>
        <div style={{ flex: 1.4, padding: "6px 12px 12px", ...ruled }}>
          <Eyebrow>Responsibilities</Eyebrow>
          {card.responsibilities.map((r, i) => (
            <div key={i} style={{ display: "flex", gap: 6, fontSize: 13.5, alignItems: "baseline" }}>
              <span style={{ flex: 1 }}>{r}</span>
              <button onClick={() => onChange({ responsibilities: card.responsibilities.filter((_, j) => j !== i) })} aria-label={`Remove responsibility: ${r}`}
                style={{ border: "none", background: "transparent", color: T.inkSoft, cursor: "pointer", fontSize: 11 }}>✕</button>
            </div>
          ))}
          <input value={respDraft} onChange={(e) => setRespDraft(e.target.value)} onKeyDown={(e) => e.key === "Enter" && addResp()} onBlur={addResp}
            placeholder="Add… (Enter)" aria-label="Add responsibility"
            style={{ width: "100%", border: "none", outline: "none", background: "transparent", fontSize: 13.5, color: T.ink, lineHeight: "26px", cursor: "text" }} />
          {crowded && <p style={{ fontSize: 11.5, color: T.redRule, margin: "4px 0 0", lineHeight: 1.4 }}>Getting crowded — a card this full often hides two classes.</p>}
        </div>

        <div style={{ width: 1, background: T.rule }} />

        <div style={{ flex: 1, padding: "6px 12px 12px", ...ruled }}>
          <Eyebrow>Collaborators</Eyebrow>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 5, paddingTop: 4 }}>
            {card.collaborators.map((n) => {
              const linked = names.some((x) => x.toLowerCase() === n.toLowerCase());
              return (
                <span key={n} style={{ display: "inline-flex", alignItems: "center", gap: 4, background: T.chipBg, border: `1px solid ${T.chipBorder}`, borderRadius: 4, padding: "1px 7px", fontSize: 12, lineHeight: "18px" }}>
                  <button onClick={() => linked && onJump(n)} title={linked ? "Go to card" : "No card with this name yet — the line appears once one exists"}
                    style={{ border: "none", background: "transparent", padding: 0, cursor: linked ? "pointer" : "default", color: linked ? T.btn : T.inkSoft, textDecoration: linked ? "underline dotted" : "none", fontSize: 12 }}>
                    {n}
                  </button>
                  <button onClick={() => onChange({ collaborators: card.collaborators.filter((x) => x !== n) })} aria-label={`Remove collaborator ${n}`}
                    style={{ border: "none", background: "transparent", color: T.inkSoft, cursor: "pointer", fontSize: 10, padding: 0 }}>✕</button>
                </span>
              );
            })}
          </div>
          <input value={collDraft} onChange={(e) => setCollDraft(e.target.value)} onKeyDown={(e) => e.key === "Enter" && addColl()}
            placeholder="Add… (Enter)" aria-label="Add collaborator"
            style={{ width: "100%", border: "none", outline: "none", background: "transparent", fontSize: 13, color: T.ink, lineHeight: "26px", cursor: "text" }} />
          {collDraft && suggestions.filter((n) => n.toLowerCase().startsWith(collDraft.toLowerCase())).length > 0 && (
            <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
              {suggestions.filter((n) => n.toLowerCase().startsWith(collDraft.toLowerCase())).slice(0, 4).map((n) => (
                <button key={n} onClick={() => addColl(n)}
                  style={{ border: `1px dashed ${T.chipBorder}`, background: "transparent", borderRadius: 4, padding: "0 7px", fontSize: 11.5, color: T.btn, cursor: "pointer", lineHeight: "18px" }}>
                  {n}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function Eyebrow({ children }) {
  return (
    <div style={{ fontFamily: "ui-monospace, Menlo, monospace", fontSize: 10, letterSpacing: "0.12em", textTransform: "uppercase", color: T.inkSoft, lineHeight: "26px" }}>
      {children}
    </div>
  );
}
