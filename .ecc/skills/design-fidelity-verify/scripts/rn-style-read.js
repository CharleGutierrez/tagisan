/* =============================================================================
 * rn-style-read.js — screenshot-free rendered-style reader for React Native
 * (the PRIMARY numeric channel for design-fidelity-verify step B5)
 * =============================================================================
 *
 * WHAT IT DOES
 *   Walks the live React fiber tree in the app's JS runtime over Metro CDP and
 *   returns, per host View/Text node: the *resolved* style object (the concrete
 *   values NativeWind or StyleSheet produced from `className` / `style`), the
 *   source `className`, the text content, and an absolute layout rect. This is
 *   the rendered truth — read off the running app, identical on iOS and Android,
 *   no screenshot, no native devtools injection required.
 *
 * WHY THIS IS THE PRIMARY CHANNEL (not native view props)
 *   NativeWind resolves `className` -> a concrete `style` object in the JS layer
 *   (cssInterop); plain StyleSheet does the same. That object lands on the *host*
 *   fiber's `memoizedProps`. Reading it over CDP works on every dev-client.
 *   iOS native paint props (`native-find-views`) frequently won't inject
 *   (`connected:false`), and even when they do, on the New Architecture (Fabric)
 *   they surface less than this read — background + frame, but no cornerRadius
 *   and no font. Treat native paint as a bonus cross-check, never the pass.
 *   On Android there is no native paint channel at all; this is the only source.
 *
 * HOW TO RUN IT  (argent debugger-evaluate  OR  metro-mcp evaluate_js — same CDP)
 *   1) Install once (paste this whole file as the expression). The JS runtime
 *      persists between evals, so you install once per app session:
 *         argent     debugger-evaluate { expression: <contents of this file> }
 *         metro-mcp  evaluate_js       { code:       <contents of this file> }
 *   2) Then call it (separate eval). **Wrap the call in JSON.stringify.** Many
 *      MCP eval wrappers (Argent, metro-mcp) marshal only primitives back, so an
 *      object result is silently dropped with no error. (Raw CDP Runtime.evaluate
 *      returns a RemoteObject and doesn't need this — but stringify anyway.)
 *         JSON.stringify(globalThis.__rnStyleRead({ targets: ["rounded-lg",
 *           "text-primary-500"], maxNodes: 250 }))
 *
 * ANDROID
 *   Run `adb -s <serial> reverse tcp:8081 tcp:8081` BEFORE connecting to Metro.
 *   Without it the app has no route to Metro and there is no CDP target at all.
 *
 * OPTIONS  __rnStyleRead(opts)
 *   targets   string[]  match a node when its className | componentName | text
 *                       contains ANY of these substrings. Omit/[] + includeAll
 *                       returns everything (bounded by maxNodes).
 *   includeAll boolean  return every host node (default false unless no targets).
 *   maxNodes  number    hard cap; the count dropped by the cap is reported in
 *                       meta.dropped (never a silent truncation).
 *   measure   boolean   measure layout rects (default true). Set false for a
 *                       faster style-only pass — skips a native measureInWindow
 *                       per returned node (the dominant cost of a large read).
 *   styleKeys string[]  if set, each node's `style` is trimmed to just these keys
 *                       (shrinks the CDP payload + parse). Omit = the full style.
 *   rendererId number   override the auto-detected app renderer.
 *
 * OUTPUT  { meta:{ rendererId, rootCount, hostTotal, returned, dropped, measured,
 *                  channel, ts }, nodes:[ { seq, label, tag, hostType,
 *                  componentName, className, classNameHops, style, text, rect } ] }
 *   - style  = flattened + normalized resolved style (colors -> lowercase hex,
 *              fontWeight -> string, "fully round" borderRadius -> 9999).
 *   - rect   = { x, y, w, h } absolute window points (dp on Android), or null.
 *   - ts     = null on purpose. Date.now() is omitted so two reads of the same
 *              screen are byte-comparable; stamp it from the caller if you need it.
 *
 * GOTCHAS THIS FILE ALREADY HANDLES (each one silently corrupts a hand-rolled walk)
 *   - The host fiber carries no `className` — it lives 2-7 hops up (`.return`).
 *   - `componentName` is the nearest *named ancestor*, not your file's component:
 *     a ProductCard rendering through a shared Card reports "Card".
 *   - Style may be a nested array — flatten, last-wins.
 *   - `fontWeight` may be the number 600 or the string "600".
 *   - `rounded-full` surfaces as a huge sentinel radius, not 9999.
 *   - `stateNode.canonical.publicInstance` is lazily created and usually absent
 *     on Fabric; `nativeFabricUIManager.measureInWindow` fires SYNCHRONOUSLY
 *     inside the eval, so no Promise plumbing is needed.
 *   - The fiber walk includes other mounted tabs — disambiguate by text/rect/route.
 *   - Several devtools renderers may be registered; we pick the one with hosts.
 *   - An animated element's rect changes between reads. Scope targets to STATIC
 *     classNames if you need determinism.
 *
 * SINGLE-ELEMENT FALLBACK (to spot-check one on-screen element by coordinate):
 *   argent     debugger-inspect-element { x, y }   -> file:line + props
 *   metro-mcp  inspect_at_point { x, y } / inspect_component
 *
 * Validated live on iOS (simulator) and Android (emulator) with RN 0.85
 * (Fabric / New Architecture), React 19, Hermes, NativeWind v5 (preview), NO
 * platform-specific changes: `measureInWindow` fires synchronously in-eval on
 * both, and output is byte-identical across repeated reads of a static screen.
 * ===========================================================================*/

(function installRnStyleRead() {
  // React fiber WorkTags. Stable across React 16-19; if a future React renumbers
  // them the walk finds no hosts (meta.hostTotal === 0) — a loud, obvious failure.
  var hostComponent = 5; // RCTView / RCTText / RCTImage ... (fiber.type is a string)
  var hostText = 6; // raw text node (memoizedProps is the string)
  // Composite names that are RN internals, not app components — skip when labeling.
  var internalNames = {
    View: 1, Text: 1, TextImpl: 1, RCTView: 1, RCTText: 1, RCTScrollView: 1,
    ScrollView: 1, RCTSinglelineTextInputView: 1, AnimatedComponent: 1,
    ForwardRef: 1, Memo: 1, Unknown: 1, anonymous: 1, anon: 1,
  };
  // style keys whose value is a color and should be normalized to hex.
  var colorKeys = {
    color: 1, backgroundColor: 1, borderColor: 1, borderTopColor: 1,
    borderRightColor: 1, borderBottomColor: 1, borderLeftColor: 1,
    borderStartColor: 1, borderEndColor: 1, shadowColor: 1, tintColor: 1,
    textDecorationColor: 1, textShadowColor: 1, overlayColor: 1,
  };
  var radiusKeys = {
    borderRadius: 1, borderTopLeftRadius: 1, borderTopRightRadius: 1,
    borderBottomLeftRadius: 1, borderBottomRightRadius: 1,
    borderTopStartRadius: 1, borderTopEndRadius: 1,
    borderBottomStartRadius: 1, borderBottomEndRadius: 1,
  };

  function hexByte(n) {
    var s = (n & 0xff).toString(16);
    return s.length === 1 ? "0" + s : s;
  }

  // Normalize any RN color value to a lowercase hex string. Handles hex strings
  // (3/4/6/8 digit), rgb()/rgba(), and processColor integers (safety net —
  // NativeWind emits hex strings, but a theme hook or processColor can yield ints).
  function normalizeColor(v) {
    if (v == null) return v;
    if (typeof v === "number") {
      var a = (v >>> 24) & 0xff, r = (v >>> 16) & 0xff, g = (v >>> 8) & 0xff, b = v & 0xff;
      return "#" + hexByte(r) + hexByte(g) + hexByte(b) + (a < 255 ? hexByte(a) : "");
    }
    if (typeof v !== "string") return v; // PlatformColor / OpaqueColorValue object — leave raw
    var s = v.trim().toLowerCase();
    if (s[0] === "#") {
      if (s.length === 4) return "#" + s[1] + s[1] + s[2] + s[2] + s[3] + s[3];
      if (s.length === 5) return "#" + s[1] + s[1] + s[2] + s[2] + s[3] + s[3] + s[4] + s[4];
      return s;
    }
    var m = s.match(/^rgba?\(([^)]+)\)$/);
    if (m) {
      var p = m[1].split(",").map(function (x) { return x.trim(); });
      var r2 = parseInt(p[0], 10), g2 = parseInt(p[1], 10), b2 = parseInt(p[2], 10);
      var hex = "#" + hexByte(r2) + hexByte(g2) + hexByte(b2);
      if (p.length === 4) {
        var af = parseFloat(p[3]);
        if (af < 1) hex += hexByte(Math.round(af * 255));
      }
      return hex;
    }
    return s; // named color or unrecognized — leave as-is
  }

  // Recursively flatten a RN style (object | array | nested arrays | false/null),
  // last value wins — matches StyleSheet.flatten semantics.
  function flattenStyle(style, out) {
    out = out || {};
    if (!style) return out;
    if (Array.isArray(style)) {
      for (var i = 0; i < style.length; i++) flattenStyle(style[i], out);
      return out;
    }
    if (typeof style === "object") {
      for (var k in style) {
        if (Object.prototype.hasOwnProperty.call(style, k)) out[k] = style[k];
      }
    }
    return out;
  }

  function normalizeStyle(flat) {
    var o = {};
    for (var k in flat) {
      if (!Object.prototype.hasOwnProperty.call(flat, k)) continue;
      var v = flat[k];
      if (colorKeys[k]) o[k] = normalizeColor(v);
      else if (radiusKeys[k] && typeof v === "number" && v > 9999) o[k] = 9999; // "fully round" sentinel
      else if (k === "fontWeight" && v != null) o[k] = String(v);
      else o[k] = v;
    }
    return o;
  }

  function nearestClassName(fiber) {
    var c = fiber, hops = 0;
    while (c && hops < 10) {
      if (c.memoizedProps && typeof c.memoizedProps.className === "string") {
        return { className: c.memoizedProps.className, hops: hops };
      }
      c = c.return; hops++;
    }
    return null;
  }

  function componentName(fiber) {
    var c = fiber.return, hops = 0, firstAny = null;
    while (c && hops < 12) {
      var t = c.type;
      var nm = null;
      if (typeof t === "function") nm = t.displayName || t.name;
      else if (t && typeof t === "object") nm = t.displayName || (t.render && (t.render.displayName || t.render.name));
      if (nm) {
        if (!firstAny) firstAny = nm;
        if (!internalNames[nm]) return nm; // prefer first app-level component
      }
      c = c.return; hops++;
    }
    return firstAny;
  }

  function textOf(fiber) {
    if (fiber.tag === hostText) return typeof fiber.memoizedProps === "string" ? fiber.memoizedProps : "";
    // Text host (tag 5 RCTText): gather direct child raw-text nodes
    var c = fiber.child, s = "", n = 0;
    while (c && n < 8) {
      if (c.tag === hostText && typeof c.memoizedProps === "string") s += c.memoizedProps;
      c = c.sibling; n++;
    }
    return s;
  }

  function round2(n) { return Math.round(n * 100) / 100; }

  // Absolute window rect via the Fabric shadow node — fires synchronously in-eval.
  function measureRect(fiber) {
    var sn = fiber.stateNode;
    if (!sn) return null;
    var rect = null;
    var fab = globalThis.nativeFabricUIManager;
    try {
      if (fab && sn.node && fab.measureInWindow) {
        fab.measureInWindow(sn.node, function (x, y, w, h) {
          rect = { x: round2(x), y: round2(y), w: round2(w), h: round2(h) };
        });
        if (rect) return rect;
      }
    } catch (e) { /* fall through */ }
    // Fallback: lazily-created public instance (older arch / when present)
    try {
      var pi = sn.canonical && sn.canonical.publicInstance;
      if (pi && pi.measureInWindow) {
        pi.measureInWindow(function (x, y, w, h) {
          rect = { x: round2(x), y: round2(y), w: round2(w), h: round2(h) };
        });
        if (rect) return rect;
      }
      if (pi && typeof pi.offsetWidth === "number") {
        return { x: pi.offsetLeft, y: pi.offsetTop, w: pi.offsetWidth, h: pi.offsetHeight, _rel: true };
      }
    } catch (e2) { /* give up */ }
    return rect;
  }

  function pickRoots(hook, override) {
    // Return [{ rendererId, root }] for every renderer that has fiber roots.
    var ids = Array.from(hook.renderers ? hook.renderers.keys() : []);
    if (override != null) ids = [override];
    var picks = [];
    for (var i = 0; i < ids.length; i++) {
      var id = ids[i], roots;
      try { roots = Array.from(hook.getFiberRoots(id)); } catch (e) { continue; }
      for (var r = 0; r < roots.length; r++) {
        if (roots[r] && roots[r].current) picks.push({ rendererId: id, root: roots[r] });
      }
    }
    return picks;
  }

  globalThis.__rnStyleRead = function (opts) {
    opts = opts || {};
    var targets = opts.targets || [];
    var includeAll = opts.includeAll || targets.length === 0;
    var maxNodes = opts.maxNodes || 400;
    var doMeasure = opts.measure !== false;
    var styleKeys = opts.styleKeys && opts.styleKeys.length ? opts.styleKeys : null;

    var hook = globalThis.__REACT_DEVTOOLS_GLOBAL_HOOK__;
    if (!hook) return { meta: { error: "no __REACT_DEVTOOLS_GLOBAL_HOOK__" }, nodes: [] };

    var picks = pickRoots(hook, opts.rendererId);
    var nodes = [], hostTotal = 0, dropped = 0, seq = 0, chosenRenderer = null;

    function matches(label) {
      for (var t = 0; t < targets.length; t++) if (label.indexOf(targets[t]) !== -1) return true;
      return false;
    }

    for (var p = 0; p < picks.length; p++) {
      var stack = [picks[p].root.current], n = 0, localHosts = 0;
      while (stack.length && n < 60000) {
        var f = stack.pop(); n++;
        var isHost = f.tag === hostComponent || f.tag === hostText;
        if (isHost && f.memoizedProps != null) {
          localHosts++; hostTotal++;
          var ncn = nearestClassName(f);
          var cn = ncn ? ncn.className : "";
          var comp = componentName(f);
          var txt = textOf(f);
          var label = (comp || "") + " " + cn + " " + txt;
          var take = includeAll || matches(label);
          if (take) {
            if (nodes.length >= maxNodes) { dropped++; }
            else {
              var flat = f.tag === hostComponent ? normalizeStyle(flattenStyle(f.memoizedProps.style)) : null;
              if (flat && styleKeys) {
                var picked = {};
                for (var qk = 0; qk < styleKeys.length; qk++) if (styleKeys[qk] in flat) picked[styleKeys[qk]] = flat[styleKeys[qk]];
                flat = picked;
              }
              nodes.push({
                seq: seq++,
                label: ((comp || (f.tag === hostText ? "#text" : "host")) +
                        (cn ? " ." + cn.split(/\s+/).slice(0, 3).join(".") : "") +
                        (txt ? ' "' + txt.slice(0, 24) + '"' : "")).slice(0, 90),
                tag: f.tag,
                hostType: typeof f.type === "string" ? f.type : null,
                componentName: comp,
                className: cn || null,
                classNameHops: ncn ? ncn.hops : null,
                style: flat,
                text: txt || null,
                rect: doMeasure && f.tag === hostComponent ? measureRect(f) : null,
              });
            }
          }
        }
        if (f.child) stack.push(f.child);
        if (f.sibling) stack.push(f.sibling);
      }
      if (localHosts > 0 && chosenRenderer == null) chosenRenderer = picks[p].rendererId;
    }

    return {
      meta: {
        rendererId: chosenRenderer,
        rootCount: picks.length,
        hostTotal: hostTotal,
        returned: nodes.length,
        dropped: dropped,
        measured: doMeasure,
        channel: "cdp-fiber",
        ts: null, // stamp from the caller side; Date.now() avoided for determinism
      },
      nodes: nodes,
    };
  };

  return "rn-style-read installed: call JSON.stringify(globalThis.__rnStyleRead({targets:[...]}))";
})();
